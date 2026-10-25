use super::Case;

fn location(source: &str, needle: &str, offset: usize) -> (usize, usize, String) {
    assert_eq!(
        source.matches(needle).count(),
        1,
        "ambiguous authored diagnostic anchor: {needle}"
    );
    let position = source.find(needle).unwrap() + offset;
    let before = source.get(..position).unwrap();
    let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
    let text = source.lines().nth(line - 1).unwrap().to_owned();
    (line, column, text)
}
pub(super) fn diagnostics(
    case: &Case,
    source: &str,
) -> Vec<(usize, usize, u32, &'static str, String)> {
    let (needle, offset, errors): (&str, usize, &[(u32, &'static str)]) =
        match case.expected.as_str() {
            "clean" => return Vec::new(),
            "tdz" => (
                "const before = undefined;",
                "const before = ".len(),
                &[
                    (
                        2448,
                        "Block-scoped variable 'undefined' used before its declaration.",
                    ),
                    (2454, "Variable 'undefined' is used before being assigned."),
                ],
            ),
            "type" => (
                "const mustBeNumber: number = undefined;",
                "const ".len(),
                &[(2322, "Type 'string' is not assignable to type 'number'.")],
            ),
            "member" => (
                "undefined.notARealMember",
                "undefined.".len(),
                &[(
                    2339,
                    "Property 'notARealMember' does not exist on type 'string'.",
                )],
            ),
            other => panic!("unregistered diagnostic contract {other}"),
        };
    let (line, column, text) = location(source, needle, offset);
    errors
        .iter()
        .map(|(code, message)| (line, column, *code, *message, text.clone()))
        .collect()
}
pub(super) fn raw_expected(case: &Case, source: &str, file: &str) -> String {
    diagnostics(case, source)
        .into_iter()
        .map(|(line, column, code, message, _)| {
            format!("{file}({line},{column}): error TS{code}: {message}\n")
        })
        .collect()
}
pub(super) fn assert_mapper_geometry(
    case: &Case,
    source: &str,
    transform: &vize_canon::ContentMapperTransform,
) {
    assert!(
        transform.diagnostics.is_empty(),
        "{}: {:?}",
        case.id,
        transform.diagnostics
    );
    for span in &transform.mappings {
        let [generated, length, authored, original, _, _] = span.0;
        assert!(
            transform.text.get(generated..generated + length).is_some(),
            "{}: generated {:?}",
            case.id,
            span
        );
        assert!(
            source.get(authored..authored + original).is_some(),
            "{}: authored {:?}",
            case.id,
            span
        );
    }
    let placeholder = transform
        .text
        .find("const __ctx = (void 0)")
        .expect("operator context");
    let placeholder = placeholder + "const __ctx = ".len();
    assert!(
        transform
            .mappings
            .iter()
            .all(|span| span.0[0] + span.0[1] <= placeholder || span.0[0] > placeholder),
        "synthetic context cannot gain authored diagnostic ownership"
    );
    if let Some(start) = source.find("{{ undefined") {
        let start = start + "{{ ".len();
        assert!(
            transform.mappings.iter().any(|span| {
                let [generated, length, authored, original, _, features] = span.0;
                if authored > start
                    || start + "undefined".len() > authored + original
                    || length != original
                    || features == 0
                {
                    return false;
                }
                // The producer may retain the whole member expression as one
                // copied span. Keep that tuple; prove this token's exact offset.
                let token = generated + (start - authored);
                transform.text.get(generated..generated + length)
                    == source.get(authored..authored + original)
                    && transform.text.get(token..token + "undefined".len()) == Some("undefined")
            }),
            "{}: original template token stays navigable: {:?}",
            case.id,
            transform.mappings
        );
    }
    if case.id == "prop-used" {
        let bindings: Vec<_> = transform
            .text
            .lines()
            .filter_map(|line| line.trim_start().strip_prefix("const undefined = "))
            .collect();
        assert_eq!(
            bindings,
            ["props[\"undefined\"];"],
            "the complete original prop binding is retained exactly once"
        );
    }
}
