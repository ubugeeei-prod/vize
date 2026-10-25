//! The real host qualification additionally checks every raw native finding.

use std::path::Path;

use vize_canon::generate_vue_content_mapper_transform;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/narrowed-slot-host-guard-10660/src/Status.vue.txt"
));

#[test]
fn both_slot_host_passes_keep_each_inference_guard_on_its_own_ignored_line() {
    for newline in ["\n", "\r\n"] {
        let source = SOURCE.replace('\n', newline);
        let result = generate_vue_content_mapper_transform(Path::new("Status.vue"), &source)
            .expect("complete mapper projection");
        let lines = result.text.lines().collect::<Vec<_>>();
        let guarded_copies = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains("(!(status === 'pending')) ? {"))
            .collect::<Vec<_>>();
        assert_eq!(
            guarded_copies.len(),
            4,
            "both constructor/factory calls in both slot passes"
        );
        for (index, _) in guarded_copies {
            assert_eq!(
                index
                    .checked_sub(1)
                    .and_then(|index| lines.get(index))
                    .map(|line| line.trim()),
                Some("// @ts-ignore Inference-only guard; authored v-if checks own diagnostics."),
                "an ignore on a previous props-object line cannot protect this replay"
            );
        }
    }
}
