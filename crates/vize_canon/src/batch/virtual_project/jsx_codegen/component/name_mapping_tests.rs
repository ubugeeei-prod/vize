use super::super::{JsxLang, generate_jsx_virtual_ts};
use crate::virtual_ts::{
    VizeSubSpan,
    mapping::{map_generated_end_offset_to_source, map_generated_offset_to_source},
};
use std::path::Path;

const STATEMENT: &str = "export const view = __vize_jsx_expr__(__vize_jsx_component__(Widget)({\"count\": count, \"fooBar\": \"ok\", \"active\": true, \"名称\": name}));";

#[test]
fn jsx_and_tsx_preserve_complete_quoted_and_unquoted_prop_name_coordinates() {
    for (filename, lang) in [("Comp.tsx", JsxLang::Tsx), ("Comp.jsx", JsxLang::Jsx)] {
        for newline in ["\n", "\r\n"] {
            let source = "// 🧭 authored bytes\nexport const view = <Widget count={count} foo-bar=\"ok\" active 名称={name} />;\n".replace('\n', newline);
            let generated = generate_jsx_virtual_ts(Path::new(filename), &source, lang).unwrap();
            assert!(generated.diagnostics.is_empty());
            let statement = generated
                .code
                .lines()
                .find(|line| line.starts_with("export const view ="))
                .unwrap();
            assert_eq!(
                statement, STATEMENT,
                "generated TypeScript bytes stay unchanged"
            );
            let start = generated.code.find("export const view =").unwrap();
            let actual: Vec<_> = generated
                .mappings
                .iter()
                .flat_map(|mapping| &mapping.sub_spans)
                .map(|span| VizeSubSpan {
                    gen_range: span.gen_range.start - start..span.gen_range.end - start,
                    src_range: span.src_range.clone(),
                })
                .collect();
            let mut expected = Vec::new();
            for (generated_name, authored_name) in [
                ("count", "count"),
                ("fooBar", "foo-bar"),
                ("active", "active"),
                ("名称", "名称"),
            ] {
                let quoted = vize_l0::cstr!("\"{generated_name}\"");
                let key = STATEMENT.find(quoted.as_str()).unwrap();
                let authored = source.find(authored_name).unwrap();
                expected.push(VizeSubSpan {
                    gen_range: key + 1..key + quoted.len() - 1,
                    src_range: authored..authored + authored_name.len(),
                });
                expected.push(VizeSubSpan {
                    gen_range: key..key + quoted.len(),
                    src_range: authored..authored + authored_name.len(),
                });
            }
            assert_eq!(actual, expected, "complete name mapping inventory");
            for mapping in &generated.mappings {
                for span in &mapping.sub_spans {
                    assert_eq!(
                        (
                            map_generated_offset_to_source(mapping, span.gen_range.start),
                            map_generated_end_offset_to_source(mapping, span.gen_range.end)
                        ),
                        (span.src_range.start, span.src_range.end)
                    );
                }
            }
        }
    }
}
