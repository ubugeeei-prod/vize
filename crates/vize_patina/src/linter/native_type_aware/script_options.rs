use vize_croquis::script_parser::{self, ScriptParseResult, ScriptParserOptions};

pub(super) fn parse_plain_script_for_type_aware(source: &str) -> ScriptParseResult {
    script_parser::parse_script_with_options(
        source,
        ScriptParserOptions {
            options_api: true,
            legacy_vue2: is_likely_legacy_vue2_script(source),
        },
    )
}

fn is_likely_legacy_vue2_script(source: &str) -> bool {
    source.contains("@nuxtjs/composition-api")
        || source.contains("nuxt-property-decorator")
        || source.contains("Vue.extend")
        || source.contains("_Vue.extend")
}
