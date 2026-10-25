use super::super::{
    VizeMapping, VizeSubSpan, map_generated_end_offset_to_source, map_generated_offset_to_source,
};

#[test]
fn declared_endpoints_preserve_whole_tokens_without_changing_start_affinity() {
    let mapping = VizeMapping {
        gen_range: 10..30,
        src_range: 100..140,
        sub_spans: vec![VizeSubSpan {
            gen_range: 12..18,
            src_range: 102..109,
        }],
    };
    assert_eq!(
        [10, 12, 14, 17, 18, 19, 29, 30, 31].map(|at| (
            map_generated_offset_to_source(&mapping, at),
            map_generated_end_offset_to_source(&mapping, at)
        )),
        [
            (100, 100),
            (102, 102),
            (104, 104),
            (107, 107),
            (108, 109),
            (109, 109),
            (119, 119),
            (120, 140),
            (121, 140)
        ],
    );
    let shorter = VizeMapping {
        gen_range: 0..20,
        src_range: 50..60,
        sub_spans: vec![VizeSubSpan {
            gen_range: 3..11,
            src_range: 52..57,
        }],
    };
    assert_eq!(
        [0, 3, 7, 10, 11, 12, 19, 20, 21].map(|at| (
            map_generated_offset_to_source(&shorter, at),
            map_generated_end_offset_to_source(&shorter, at)
        )),
        [
            (50, 50),
            (52, 52),
            (56, 56),
            (57, 57),
            (57, 57),
            (60, 60),
            (60, 60),
            (60, 60),
            (60, 60)
        ],
    );
}
