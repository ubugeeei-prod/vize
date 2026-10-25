use vize_carton::Allocator;
use vize_croquis::sfc::__internal::{
    extract_and_transform_v_bind_with_context, extract_and_transform_v_bind_with_edits,
};

#[test]
fn actual_replacement_writes_preserve_all_untouched_bytes_and_existing_variables() {
    let cases: &[(&str, &[&str])] = &[
        (".a{color:v-bind(theme)}.b{color:blue}", &["theme"]),
        (
            "/* v-bind(no) */.a{content:'v-bind(no)';color:v-bind('主题😀')}.b{opacity:v-bind(nested(')'))}",
            &["主题😀", "nested(')')"],
        ),
        (
            " .a{color:v-bind(\r\n '雪😀'\r\n)}\r\n.b{opacity:v-bind(next)}",
            &["雪😀", "next"],
        ),
        (".a{color:v-bind(foo)v-bind(bar)}", &["foo", "bar"]),
        (".a{color:v-bind('oops'}", &[]),
        (".a{content:'v-bind(no)'}/* v-bind(no) */", &[]),
        ("", &[]),
    ];
    for &(source, expected_vars) in cases {
        for scope in [None, Some("data-v-test"), Some("data-v-雪😀")] {
            for filename in [None, Some("/src/组件雪😀.vue")] {
                let plain_arena = Allocator::default();
                let mapped_arena = Allocator::default();
                let plain = extract_and_transform_v_bind_with_context(
                    &plain_arena,
                    source,
                    scope,
                    filename,
                );
                let mut edits = Vec::new();
                let mapped = extract_and_transform_v_bind_with_edits(
                    &mapped_arena,
                    source,
                    scope,
                    filename,
                    |edit| edits.push(edit),
                );
                assert_eq!(mapped, plain);
                assert_eq!(
                    mapped
                        .1
                        .iter()
                        .map(|value| value.as_str())
                        .collect::<Vec<_>>(),
                    expected_vars
                );
                assert_eq!(edits.len(), expected_vars.len());
                let (mut original, mut generated) = (0, 0);
                for edit in &edits {
                    assert_eq!(
                        source.get(original..edit.source.start),
                        mapped.0.get(generated..edit.generated.start)
                    );
                    assert!(
                        source
                            .get(edit.source.clone())
                            .unwrap()
                            .starts_with("v-bind(")
                    );
                    let replacement = mapped.0.get(edit.generated.clone()).unwrap();
                    assert!(replacement.starts_with("var(--"));
                    assert!(replacement.ends_with(')'));
                    original = edit.source.end;
                    generated = edit.generated.end;
                }
                assert_eq!(source.get(original..), mapped.0.get(generated..));
                if expected_vars.is_empty() {
                    assert_eq!(edits.capacity(), 0);
                }
            }
        }
    }
}
