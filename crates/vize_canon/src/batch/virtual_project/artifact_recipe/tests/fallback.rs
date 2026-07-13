use vize_atelier_sfc::{
    authored_script_parse_invocations, reset_authored_script_parse_invocations,
};
use vize_carton::config::VueVersion;

use super::*;

#[test]
fn every_vue_dialect_and_script_shape_uses_the_parse_once_canon_projection() {
    let shapes = [
        (
            "normal",
            "<script lang=\"ts\">export default { data() { return { value: 1 } } }</script><template>{{ value }}</template>",
            1,
        ),
        (
            "setup",
            "<script setup lang=\"ts\">const value = 1</script><template>{{ value }}</template>",
            1,
        ),
        (
            "dual",
            "<script lang=\"ts\">export interface Props { value: number }; export default {}</script><script setup lang=\"ts\">const props = defineProps<Props>()</script><template>{{ props.value }}</template>",
            2,
        ),
    ];

    for dialect in [
        VueVersion::V1,
        VueVersion::V2,
        VueVersion::V2_7,
        VueVersion::V3,
    ] {
        for (shape, content, expected_parses) in shapes {
            reset_authored_script_parse_invocations();
            crate::virtual_ts::reset_authored_script_fallback_parse_invocations();
            let case_name = format!("atlas-canon-{}-{shape}", dialect.as_str());
            let (root, src) = case(&case_name);
            let path = src.join("Fixture.vue");
            let mut project = VirtualProject::new(&root).unwrap();
            project.set_dialect(dialect);
            let (compilation, sources) =
                prepare_compilation(&project, &[source(path, content)]).unwrap();
            let snapshot = compilation.snapshot();
            let mut session = snapshot.query_session();
            let outcome = session
                .query::<CanonTypedDocumentProduct>(sources[0])
                .unwrap();

            outcome.value().to_corsa_result().unwrap();
            assert_eq!(
                authored_script_parse_invocations(),
                expected_parses,
                "{} {shape} must parse each authored block once",
                dialect.as_str()
            );
            assert_eq!(
                crate::virtual_ts::authored_script_fallback_parse_invocations(),
                0,
                "{} {shape} must not enter Canon's compatibility parser",
                dialect.as_str()
            );
            let _ = fs::remove_dir_all(root);
        }
    }
}
