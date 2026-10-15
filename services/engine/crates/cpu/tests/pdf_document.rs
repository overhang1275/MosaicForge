use mosaicforge_core::{contract::PdfJobSpec, template::import_svg};
use mosaicforge_cpu::pdf::generate_pdf;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn setup(count: u64) -> (mosaicforge_core::template::Template, PdfJobSpec) {
    let t=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100" fill="#123456"/></g></svg>"##).unwrap();
    let mut g: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    g["templateHash"] = json!(t.hash);
    g["count"] = json!(count);
    (t,serde_json::from_value(json!({"schemaVersion":"1","generation":g,"pdf":{"exporterVersion":"1","trimWidthMm":100,"trimHeightMm":100,"bleedMm":0}})).unwrap())
}
#[test]
fn groups_share_one_pdf_hash_and_order_with_partial_final_group() {
    let (t, j) = setup(201);
    let mut expected = None;
    for n in [1, 7, 100] {
        let mut bytes = Vec::new();
        let mut map = Vec::new();
        let result = generate_pdf(&t, &j, n, &mut bytes, |r| {
            map.push(serde_json::to_value(r).unwrap());
            Ok(())
        })
        .unwrap();
        let result = serde_json::to_value(result).unwrap();
        assert_eq!(result["pageCount"], 201);
        assert_eq!(
            result["documentHash"],
            format!("{:x}", Sha256::digest(&bytes))
        );
        assert!(bytes.starts_with(b"%PDF-1.7"));
        assert!(bytes.ends_with(b"%%EOF\n"));
        for (i, r) in map.iter().enumerate() {
            assert_eq!(r["variant"]["index"], i.to_string());
            assert_eq!(r["pageNumber"], i + 1);
        }
        if let Some((ref b, ref m)) = expected {
            assert_eq!(&bytes, b);
            assert_eq!(&map, m);
        } else {
            expected = Some((bytes, map));
        }
    }
}
#[test]
fn no_accepted_pages_and_failures_do_not_return_completed() {
    let (t, mut j) = setup(2);
    j.generation.rules.opacity.min = 0.0;
    j.generation.rules.opacity.max = 0.0;
    let mut bytes = Vec::new();
    let mut map = Vec::new();
    let r = generate_pdf(&t, &j, 1, &mut bytes, |r| {
        map.push(serde_json::to_value(r).unwrap());
        Ok(())
    })
    .unwrap();
    let r = serde_json::to_value(r).unwrap();
    assert_eq!(r["status"], "no_pages");
    assert!(r.get("documentHash").is_none_or(Value::is_null));
    assert!(bytes.is_empty());
    assert!(
        map.iter()
            .all(|r| r.get("pageNumber").is_none_or(Value::is_null))
    );
    assert!(generate_pdf(&t, &j, 0, &mut bytes, |_| Ok(())).is_err());
    j.generation.rules.opacity.min = 1.0;
    j.generation.rules.opacity.max = 1.0;
    assert!(
        generate_pdf(&t, &j, 1, &mut bytes, |_| Err(
            mosaicforge_core::EngineError::RenderFailed("callback")
        ))
        .is_err()
    );
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("injected"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(generate_pdf(&t, &j, 1, &mut Broken, |_| Ok(())).is_err());
}

#[test]
fn rejected_indices_leave_no_page_gaps_and_u64_max_is_preserved() {
    let (t, mut j) = setup(24);
    j.generation.rules.rotation_degrees.min = 0.0;
    j.generation.rules.rotation_degrees.max = 90.0;
    j.generation.rules.scale_x.max = 1.3;
    j.generation.rules.scale_y.max = 1.3;
    let mut bytes = Vec::new();
    let mut map = Vec::new();
    let r = generate_pdf(&t, &j, 7, &mut bytes, |r| {
        map.push(r);
        Ok(())
    })
    .unwrap();
    assert!(r.accepted_count > 0 && r.rejected_count > 0);
    let mut page = 0;
    for (i, record) in map.iter().enumerate() {
        assert_eq!(record.variant.index.as_str(), i.to_string());
        if record.variant.status.to_string() == "accepted" {
            page += 1;
            assert_eq!(record.page_number.unwrap().get(), page);
        } else {
            assert!(record.page_number.is_none());
        }
    }
    let (t, mut j) = setup(1);
    j.generation.start_index = u64::MAX.to_string().parse().unwrap();
    let mut last = None;
    generate_pdf(&t, &j, 100, &mut Vec::new(), |r| {
        last = Some(r);
        Ok(())
    })
    .unwrap();
    assert_eq!(last.unwrap().variant.index.as_str(), u64::MAX.to_string());
}
