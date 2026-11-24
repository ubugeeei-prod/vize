//! Check the resident visitor's actual binary partition against direct containment.
extern crate core;
mod window;

#[kani::proof]
#[kani::unwind(9)]
fn window_matches_each_requested_cursor() {
    let storage: [u32; 7] = kani::any();
    let length: usize = kani::any();
    kani::assume(length <= storage.len());
    let points = &storage[..length];
    for adjacent in points.windows(2) {
        kani::assume(adjacent[0] <= adjacent[1]);
    }
    let start: u32 = kani::any();
    let end: u32 = kani::any();
    kani::assume(start <= end);

    let hits = window::matching_positions(points, start, end);
    if length <= 4 {
        assert_eq!(window::matching_short(points, start, end), hits);
    }
    if let [cursor] = *points {
        assert_eq!(window::matching_position(cursor, start, end), hits);
    }
    assert!(hits.start <= hits.end && hits.end <= length);
    for (index, &point) in points.iter().enumerate() {
        let direct = (start <= point && point < end) || (start == end && point == start);
        assert_eq!(hits.contains(&index), direct);
        assert_eq!(index < hits.start, point < start);
    }
    kani::cover!(length == 0, "no requested positions");
    kani::cover!(
        start == u32::MAX && hits.len() >= 2,
        "repeated full-width EOF anchor"
    );
    kani::cover!(
        start < end && !hits.is_empty() && points.last() == Some(&end),
        "nonempty right endpoint excluded"
    );
    kani::cover!(
        hits.start > 0 && hits.end < length && hits.len() > 1,
        "interior match with a prefix and suffix"
    );
    kani::cover!(length == 3 && hits.len() == 2, "short group with a nonmatching cursor");
    kani::cover!(length == 7 && hits.start == 0 && hits.end == 5, "matching prefix continues into the tail");
}
