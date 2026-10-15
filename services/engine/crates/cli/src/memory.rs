use mosaicforge_core::EngineError;
use std::sync::{Condvar, Mutex};

pub struct BufferBudget {
    limit: u64,
    used: Mutex<u64>,
    changed: Condvar,
}
pub struct Reservation<'a> {
    budget: &'a BufferBudget,
    bytes: u64,
}
impl BufferBudget {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            used: Mutex::new(0),
            changed: Condvar::new(),
        }
    }
    pub fn reserve(&self, bytes: u64) -> Result<Reservation<'_>, EngineError> {
        if bytes > self.limit {
            return Err(EngineError::InvalidSpec(
                "PNG reservation exceeds shared budget",
            ));
        }
        let mut used = self
            .used
            .lock()
            .map_err(|_| EngineError::RenderFailed("shared PNG budget poisoned"))?;
        while bytes > self.limit - *used {
            used = self
                .changed
                .wait(used)
                .map_err(|_| EngineError::RenderFailed("shared PNG budget poisoned"))?;
        }
        *used += bytes;
        Ok(Reservation {
            budget: self,
            bytes,
        })
    }
}
impl Drop for Reservation<'_> {
    fn drop(&mut self) {
        let mut used = self.budget.used.lock().unwrap_or_else(|e| e.into_inner());
        *used -= self.bytes;
        self.budget.changed.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, time::Duration};
    #[test]
    fn reservations_wait_without_rejecting_an_individually_valid_large_image() {
        let budget = BufferBudget::new(100);
        let first = budget.reserve(70).unwrap();
        std::thread::scope(|scope| {
            let (started_tx, started_rx) = mpsc::channel();
            let (done_tx, done_rx) = mpsc::channel();
            let budget = &budget;
            scope.spawn(move || {
                started_tx.send(()).unwrap();
                let _second = budget.reserve(90).unwrap();
                assert_eq!(*budget.used.lock().unwrap(), 90);
                done_tx.send(()).unwrap();
            });
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(matches!(done_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            assert_eq!(*budget.used.lock().unwrap(), 70);
            drop(first);
            done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        });
        assert_eq!(*budget.used.lock().unwrap(), 0);
    }
    #[test]
    fn capacity_is_shared_and_released_on_error() {
        let budget = BufferBudget::new(100);
        let first = budget.reserve(70).unwrap();
        let fail = || -> Result<(), EngineError> {
            let _second = budget.reserve(30)?;
            assert_eq!(*budget.used.lock().unwrap(), 100);
            Err(EngineError::RenderFailed("injected renderer error"))
        };
        assert!(fail().is_err());
        assert_eq!(*budget.used.lock().unwrap(), 70);
        assert!(budget.reserve(101).is_err());
        drop(first);
        assert_eq!(*budget.used.lock().unwrap(), 0);
        assert!(budget.reserve(100).is_ok());
    }
    #[test]
    fn maximum_canvas_is_not_rejected_by_dividing_budget_between_workers() {
        use mosaicforge_cpu::render::{MAX_RENDER_BUFFER_BYTES, plan_png_render};
        let output = serde_json::from_value(serde_json::json!({"format":"png","width":4096,"height":4096,"background":"transparent"})).unwrap();
        let plan = plan_png_render(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="#123456"/></svg>"##, &output, MAX_RENDER_BUFFER_BYTES).unwrap();
        assert!(plan.reserved_bytes > MAX_RENDER_BUFFER_BYTES / 2);
        let budget = BufferBudget::new(MAX_RENDER_BUFFER_BYTES);
        assert!(budget.reserve(plan.reserved_bytes).is_ok());
    }
}
