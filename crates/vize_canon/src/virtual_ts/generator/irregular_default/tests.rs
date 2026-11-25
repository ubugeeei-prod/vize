use super::{IrregularDefault, TextBuf, VizeMapping};
use crate::virtual_ts::generator::options_api::analyze_options_api_script;
use crate::virtual_ts::generator::template_ref_keys::TemplateRefKeyChecks;
use crate::virtual_ts::mapping::{map_generated_offset_to_source, mapping_for_generated_offset};

#[test]
fn preserves_trivia_and_exact_body_mappings_across_keyword_lines() {
    for (header, expected) in [
        (
            "export /* 日本 */ default ",
            "const before = 1; const __default__ = __vizeDefineComponent(       /* 日本 */         { name: 'A' }); const after: number = 'bad';\n",
        ),
        (
            "export\r\ndefault ",
            "const before = 1; const __default__ = __vizeDefineComponent(      \r\n        { name: 'A' }); const after: number = 'bad';\n",
        ),
        (
            "export // @ts-ignore\ndefault ",
            "const before = 1; const __default__ = __vizeDefineComponent(       // @ts-ignore\n        { name: 'A' }); const after: number = 'bad';\n",
        ),
    ] {
        let source = vize_carton::cstr!(
            "const before = 1; {header}{{ name: 'A' }}; const after: number = 'bad';"
        );
        let facts = analyze_options_api_script(&source, true, false);
        let rewrite = IrregularDefault::new(&source, &facts).unwrap();
        let mut generated = TextBuf::default();
        let mut mappings = Vec::new();
        let mut start = 0;
        for line in source.split('\n') {
            let gen_start = generated.len();
            if let Some(output) = rewrite.project_line(line, start, &[]) {
                output.emit(
                    &mut generated,
                    &mut mappings,
                    &TemplateRefKeyChecks::default(),
                    start,
                    &|x| x,
                );
            } else {
                mappings.push(VizeMapping::new(
                    gen_start..gen_start + line.len(),
                    start..start + line.len(),
                ));
                generated.push_str(line);
                generated.push('\n');
            }
            start += line.len() + 1;
        }
        assert_eq!(generated.as_str(), expected);
        for token in ["before", "name", "after", "'bad'"] {
            let offset = generated.find(token).unwrap();
            let mapping = mapping_for_generated_offset(&mappings, offset).unwrap();
            assert_eq!(
                map_generated_offset_to_source(mapping, offset),
                source.find(token).unwrap()
            );
        }
    }
}

#[test]
fn named_class_keeps_decorators_and_defers_alias() {
    let source = "export /* gap */ default @Component() class Foo {}";
    let facts = analyze_options_api_script(source, true, false);
    let rewrite = IrregularDefault::new(source, &facts).unwrap();
    let output = rewrite.project_line(source, 0, &[]).unwrap();
    assert_eq!(
        output.text.as_str(),
        "       /* gap */         @Component() class Foo {}"
    );
    assert_eq!(rewrite.class_alias(0, source.len()), Some("Foo"));
}
