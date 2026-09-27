//! #6846: tag formatting preserves authored closing forms without resolution.

use serde::Deserialize;
use vize_glyph::{FormatOptions, format_sfc, format_template};
use vize_l0::String;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Corpus {
    schema_version: u32,
    native: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    expected: String,
}

#[test]
fn public_formatter_preserves_authored_tag_forms() {
    let corpus: Corpus = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/formatter-tag-forms/cases.json"
    ))
    .expect("strict authored tag corpus");
    assert_eq!(corpus.schema_version, 1);
    assert_eq!(corpus.native, "unsupported");
    assert_eq!(corpus.cases.len(), 11, "every audited shape must execute");
    let options = FormatOptions::default();
    for case in corpus.cases {
        let template = format_template(&case.source, &options).expect("valid template");
        assert_eq!(template.as_bytes(), case.expected.as_bytes(), "{}", case.id);
        assert_eq!(
            format_template(&template, &options)
                .expect("template repeat")
                .as_bytes(),
            template.as_bytes(),
            "{}",
            case.id
        );

        let mut source = String::from("<template>");
        source.push_str(&case.source);
        source.push_str("</template>");
        let mut expected = String::from("<template>\n  ");
        expected.push_str(&case.expected);
        expected.push_str("\n</template>\n");
        let sfc = format_sfc(&source, &options).expect("valid SFC");
        assert_eq!(sfc.code.as_bytes(), expected.as_bytes(), "{}", case.id);
        let repeat = format_sfc(&sfc.code, &options).expect("SFC repeat");
        assert_eq!(repeat.code.as_bytes(), sfc.code.as_bytes(), "{}", case.id);
        assert!(!repeat.changed, "{}", case.id);
    }
}
