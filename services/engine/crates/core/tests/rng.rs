use mosaicforge_core::rng::{map_u32_to_range, philox4x32_10, sample_u32};

// Random123 tests/kat_vectors, Philox4x32 with ten rounds, consulted 2026-10-14.
#[test]
fn philox_matches_reference_vectors() {
    for (counter, key, expected) in [
        (
            [0; 4],
            [0; 2],
            [0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8],
        ),
        (
            [u32::MAX; 4],
            [u32::MAX; 2],
            [0x408f276d, 0x41c83b0e, 0xa20bc7c6, 0x6d5451fd],
        ),
        (
            [0x243f6a88, 0x85a308d3, 0x13198a2e, 0x03707344],
            [0xa4093822, 0x299f31d0],
            [0xd16cfe09, 0x94fdcceb, 0x5001e420, 0x24126ea1],
        ),
    ] {
        assert_eq!(philox4x32_10(counter, key), expected);
    }
}

#[test]
fn samples_are_index_local() {
    let expected = sample_u32(u64::MAX, u64::MAX, u16::MAX, 7, u32::MAX).unwrap();
    for index in (0..200).rev() {
        for attempt in 0..8 {
            sample_u32(u64::MAX, index, 2, attempt, 4).unwrap();
        }
    }
    assert_eq!(
        sample_u32(u64::MAX, u64::MAX, u16::MAX, 7, u32::MAX).unwrap(),
        expected
    );
    let full: Vec<_> = (0..200)
        .map(|i| sample_u32(42, i, 3, 0, 9).unwrap())
        .collect();
    let partial: Vec<_> = (100..200)
        .map(|i| sample_u32(42, i, 3, 0, 9).unwrap())
        .collect();
    assert_eq!(full[100..], partial);
}

#[test]
fn sample_mapping_matches_counter_and_lane() {
    for (seed, index, rule, attempt, draw) in [
        (0, 0, 0, 0, 0),
        (u64::MAX, u64::MAX, u16::MAX, 7, u32::MAX),
        (0x0123456789abcdef, 0xfedcba9876543210, 19, 3, 4),
        (42, 200, 65535, 6, 5),
    ] {
        let key = [seed as u32, (seed >> 32) as u32];
        let counter = [
            index as u32,
            (index >> 32) as u32,
            draw / 4,
            u32::from(rule) | (u32::from(attempt) << 16),
        ];
        let expected = philox4x32_10(counter, key)[(draw % 4) as usize];
        assert_eq!(
            sample_u32(seed, index, rule, attempt, draw).unwrap(),
            expected
        );
    }
}

#[test]
fn all_four_lanes_and_distinct_domains_are_used() {
    let expected = philox4x32_10([0; 4], [0; 2]);
    for lane in 0..4 {
        assert_eq!(
            sample_u32(0, 0, 0, 0, lane).unwrap(),
            expected[lane as usize]
        );
    }
    let baseline = sample_u32(42, 5, 2, 0, 0).unwrap();
    for (seed, index, rule, attempt, draw) in [
        (43, 5, 2, 0, 0),
        (42, 6, 2, 0, 0),
        (42, 5, 3, 0, 0),
        (42, 5, 2, 1, 0),
        (42, 5, 2, 0, 4),
    ] {
        assert_ne!(
            sample_u32(seed, index, rule, attempt, draw).unwrap(),
            baseline
        );
    }
}

#[test]
fn attempts_outside_domain_are_rejected() {
    for attempt in 8..=u8::MAX {
        assert!(sample_u32(0, 0, 0, attempt, 0).is_err());
    }
}

#[test]
fn schedules_and_batches_do_not_change_samples() {
    let sequential: Vec<_> = (0..201)
        .map(|i| sample_u32(123, i, 4, 2, 15).unwrap())
        .collect();
    for size in [1, 7, 100] {
        let mut grouped = Vec::new();
        for group in sequential.chunks(size).enumerate().rev() {
            let start = group.0 * size;
            for i in (start..start + group.1.len()).rev() {
                grouped.push((i, sample_u32(123, i as u64, 4, 2, 15).unwrap()));
            }
        }
        grouped.sort_unstable_by_key(|(i, _)| *i);
        assert_eq!(
            grouped
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>(),
            sequential
        );
    }
    let threaded = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            (0..100)
                .map(|i| sample_u32(123, i, 4, 2, 15).unwrap())
                .collect::<Vec<_>>()
        });
        let second = scope.spawn(|| {
            (100..201)
                .map(|i| sample_u32(123, i, 4, 2, 15).unwrap())
                .collect::<Vec<_>>()
        });
        [first.join().unwrap(), second.join().unwrap()].concat()
    });
    assert_eq!(threaded, sequential);
}

#[test]
fn range_mapping_is_finite_bounded_and_defined() {
    assert_eq!(map_u32_to_range(0, -2.0, 6.0).unwrap(), -2.0);
    assert_eq!(map_u32_to_range(1 << 31, -2.0, 6.0).unwrap(), 2.0);
    assert_eq!(map_u32_to_range(1 << 31, -f64::MAX, f64::MAX).unwrap(), 0.0);
    assert!(map_u32_to_range(u32::MAX, 0.0, 1.0).unwrap() < 1.0);
    for value in [0, 1, 1 << 31, u32::MAX] {
        assert_eq!(map_u32_to_range(value, 3.0, 3.0).unwrap(), 3.0);
        let mapped = map_u32_to_range(value, -f64::MAX, f64::MAX).unwrap();
        assert!(mapped.is_finite());
        assert!((-f64::MAX..=f64::MAX).contains(&mapped));
    }
    for (min, max) in [
        (1.0, 0.0),
        (f64::NAN, 1.0),
        (0.0, f64::INFINITY),
        (f64::NEG_INFINITY, 0.0),
    ] {
        assert!(map_u32_to_range(0, min, max).is_err());
    }
}
