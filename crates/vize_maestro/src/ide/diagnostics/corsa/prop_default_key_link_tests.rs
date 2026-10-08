use crate::DiagnosticService;
use tower_lsp::lsp_types::Url;
use vize_canon::virtual_ts::VizeSemanticLinkKind;

#[test]
fn auxiliary_default_key_links_survive_import_and_tsx_prefix_rewrites() {
    let uri = Url::parse("file:///workspace/App.vue").unwrap();
    for language in ["ts", "tsx"] {
        for newline in ["\n", "\r\n"] {
            let source = vize_l0::cstr!(
                r#"<!-- 😀 -->
<script setup lang="{language}">
import Child from './Child.vue';
type Props = {{ tone?: 'light' | 'dark' }};
withDefaults(defineProps<Props>(), {{ tone: 'light' }});
void Child;
</script>
<template><div>{{{{ tone }}}}</div></template>"#
            )
            .replace('\n', newline);
            let result =
                DiagnosticService::generate_virtual_ts(&uri, &source, false, false).unwrap();
            assert!(result.code.contains("./Child.vue.ts"));
            let core: Vec<_> = result
                .semantic_links
                .iter()
                .filter(|link| link.kind == VizeSemanticLinkKind::VueTemplatePropBinding)
                .collect();
            assert_eq!(core.len(), 1);
            assert_eq!(result.prop_default_key_links.len(), 1);
            let key = &result.prop_default_key_links[0];
            assert_eq!(key.source_range, core[0].source_range);
            assert_eq!(result.code.get(key.source_range.clone()), Some("tone"));
            assert_eq!(result.code.get(key.target_range.clone()), Some("tone"));
            assert_ne!(key.target_range, core[0].target_range);
            for endpoint in [key.source_range.start, key.target_range.start] {
                let (line, character) = crate::ide::offset_to_position(&result.code, endpoint);
                assert_eq!(
                    crate::ide::position_to_offset(&result.code, line, character),
                    Some(endpoint)
                );
            }
        }
    }
}
