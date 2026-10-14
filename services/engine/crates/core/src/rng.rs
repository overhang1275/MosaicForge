use crate::EngineError;

pub fn philox4x32_10(mut counter: [u32; 4], mut key: [u32; 2]) -> [u32; 4] {
    for _ in 0..10 {
        let first = u64::from(counter[0]) * 0xd251_1f53;
        let second = u64::from(counter[2]) * 0xcd9e_8d57;
        counter = [
            (second >> 32) as u32 ^ counter[1] ^ key[0],
            second as u32,
            (first >> 32) as u32 ^ counter[3] ^ key[1],
            first as u32,
        ];
        key[0] = key[0].wrapping_add(0x9e37_79b9);
        key[1] = key[1].wrapping_add(0xbb67_ae85);
    }
    counter
}

pub fn sample_u32(
    seed: u64,
    index: u64,
    rule: u16,
    attempt: u8,
    draw: u32,
) -> Result<u32, EngineError> {
    if attempt > 7 {
        return Err(EngineError::InvalidSpec(
            "attempt must be between zero and seven",
        ));
    }
    let counter = [
        index as u32,
        (index >> 32) as u32,
        draw / 4,
        u32::from(rule) | (u32::from(attempt) << 16),
    ];
    let key = [seed as u32, (seed >> 32) as u32];
    Ok(philox4x32_10(counter, key)[(draw % 4) as usize])
}

pub fn map_u32_to_range(value: u32, min: f64, max: f64) -> Result<f64, EngineError> {
    if !min.is_finite() || !max.is_finite() || min > max {
        return Err(EngineError::InvalidSpec(
            "sample interval must be finite and ordered",
        ));
    }
    if min == max || value == 0 {
        return Ok(min);
    }
    let unit = f64::from(value) / 4_294_967_296.0;
    // Weighted endpoints avoid overflowing max - min for opposite finite extremes.
    let mapped = (1.0 - unit) * min + unit * max;
    if !mapped.is_finite() {
        return Err(EngineError::InvalidSpec(
            "sample interval cannot produce a finite value",
        ));
    }
    Ok(mapped.clamp(min, max))
}
