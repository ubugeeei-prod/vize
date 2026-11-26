//! Shared parses are lent across reads, kept past their scope and freed in
//! drop order. The Miri lane runs these tests, so the lifetime-erased parse,
//! the owned source pointer and the drop order are also checked under the
//! aliasing model.

use super::*;

fn tree_address(text: &str, source_type: SourceType) -> usize {
    with_shared_parse(text, source_type, |parsed| {
        std::ptr::from_ref(&parsed.program) as usize
    })
}

#[test]
fn a_scope_parses_equal_text_and_dialect_once() {
    let _scope = SharedParseScope::enter();
    let text: Box<str> = Box::from("const value = 1;\n");
    let copy: Box<str> = Box::from(&*text);
    assert_eq!(
        tree_address(&text, SourceType::ts()),
        tree_address(&copy, SourceType::ts())
    );
    assert_ne!(
        tree_address(&text, SourceType::ts()),
        tree_address(&text, SourceType::ts().with_module(true))
    );
    assert_ne!(
        tree_address(&text, SourceType::ts()),
        tree_address("const value = 2;\n", SourceType::ts())
    );
}

#[test]
fn a_shared_parse_matches_a_private_one() {
    let text = "import { ref } from 'vue'\nconst count = ref(\n";
    let private = with_shared_parse(text, SourceType::ts(), |parsed| {
        (
            parsed.panicked,
            parsed.diagnostics.len(),
            parsed.program.body.len(),
        )
    });
    let _scope = SharedParseScope::enter();
    for _ in 0..2 {
        let shared = with_shared_parse(text, SourceType::ts(), |parsed| {
            (
                parsed.panicked,
                parsed.diagnostics.len(),
                parsed.program.body.len(),
            )
        });
        assert_eq!(shared, private);
    }
}

#[test]
fn nested_reads_share_and_a_nested_scope_joins_the_outer_one() {
    let _scope = SharedParseScope::enter();
    let outer = tree_address("let a = 1;", SourceType::ts());
    let nested = with_shared_parse("let b = 2;", SourceType::ts(), |_| {
        let _inner = SharedParseScope::enter();
        (
            tree_address("let a = 1;", SourceType::ts()),
            tree_address("let b = 2;", SourceType::ts()),
        )
    });
    assert_eq!(nested.0, outer);
    assert_eq!(nested.1, tree_address("let b = 2;", SourceType::ts()));
    assert_eq!(tree_address("let a = 1;", SourceType::ts()), outer);
}

#[test]
fn a_ts_or_tsx_extension_shares_the_dialect_parse_and_others_stay_apart() {
    let text = "const value = <T,>(input: T) => input;\n";
    let from_path = |path| SourceType::from_path(path).unwrap_or_default();
    let private = with_shared_parse(text, from_path("script.ts"), |parsed| {
        (
            parsed.panicked,
            parsed.diagnostics.len(),
            parsed.program.body.len(),
        )
    });
    let _scope = SharedParseScope::enter();
    let ts = tree_address(text, SourceType::ts());
    assert_eq!(tree_address(text, from_path("script.ts")), ts);
    assert_eq!(
        tree_address(text, from_path("script.tsx")),
        tree_address(text, SourceType::tsx())
    );
    assert_ne!(tree_address(text, from_path("script.tsx")), ts);
    assert_eq!(
        with_shared_parse(text, from_path("script.ts"), |parsed| {
            (
                parsed.panicked,
                parsed.diagnostics.len(),
                parsed.program.body.len(),
            )
        }),
        private
    );
    for separate in [
        "script.mts",
        "script.cts",
        "script.d.ts",
        "script.js",
        "script.jsx",
    ] {
        let source_type = from_path(separate);
        assert_eq!(program_source_type(source_type), source_type, "{separate}");
        assert_ne!(tree_address(text, source_type), ts, "{separate}");
    }
    assert_eq!(
        program_source_type(from_path("script.ts").with_module(true)),
        SourceType::ts().with_module(true)
    );
}

fn parse_debug(text: &str, source_type: SourceType) -> vize_carton::String {
    use vize_carton::ToCompactString;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, text, source_type).parse();
    let diagnostics: Vec<vize_carton::String> = parsed
        .diagnostics
        .iter()
        .map(ToCompactString::to_compact_string)
        .collect();
    vize_carton::cstr!(
        "{:?}\n{:?}\n{}",
        parsed.program,
        diagnostics,
        parsed.panicked
    )
}

#[test]
fn a_module_goal_reuses_the_unambiguous_parse_only_where_they_are_identical() {
    let reused = [
        "import { ref } from 'vue'\nconst count = ref(0)\nawait load()\n",
        "// leading comment\nimport type { A } from './a'\nlet a: A\nfor await (const x of y) {}\n",
        "import x from 'x'\nconst f = (a) => a <\n  b\nconst g = async () => { await x }\n",
        "import { h } from 'vue'\nexport default { render: () => h('div') }\n",
    ];
    let separate = [
        "const value = await load()\nimport x from 'x'\n",
        "import await from 'x'\n",
        "const ready = true\n",
        "console.log(import.meta.url)\n",
        "import x from 'x'\nconst y = 1 <!-- 2\n",
        "import x from 'x'\n--> comment\n",
        "await x\n",
        "export const value = await fetch(url)\n",
        "import x from 'x'\nconst broken = (\n",
    ];
    for (text, expect) in reused
        .iter()
        .map(|text| (text, true))
        .chain(separate.iter().map(|text| (text, false)))
    {
        for dialect in [SourceType::ts(), SourceType::tsx()] {
            let _scope = SharedParseScope::enter();
            let unambiguous = tree_address(text, dialect);
            let module = tree_address(text, dialect.with_module(true));
            assert_eq!(module == unambiguous, expect, "{text:?} {dialect:?}");
            if expect {
                assert_eq!(
                    parse_debug(text, dialect.with_module(true)),
                    parse_debug(text, dialect),
                    "{text:?} {dialect:?}"
                );
            }
        }
    }
    let _scope = SharedParseScope::enter();
    let text = "import x from 'x'\n";
    let module = tree_address(text, SourceType::ts().with_module(true));
    assert_ne!(tree_address(text, SourceType::ts()), module);
}

#[test]
fn dropping_the_scope_stops_sharing() {
    {
        let _scope = SharedParseScope::enter();
        tree_address("let a = 1;", SourceType::ts());
    }
    PARSES.with_borrow(|parses| assert!(parses.is_none()));
}

#[test]
fn a_lent_parse_stays_readable_while_the_scope_grows() {
    let _scope = SharedParseScope::enter();
    let text = "import { ref } from 'vue'\nconst count = ref(0)\n";
    let kept = shared_program(text, SourceType::ts()).expect("inside a scope");
    let parsed = kept.get();
    let body = parsed.program.body.len();
    // Enough new parses to reallocate the scope's list several times while
    // `parsed` still borrows the first one.
    let mut addresses = std::vec::Vec::new();
    for index in 0..40u32 {
        let other = vize_carton::cstr!("let value{} = {};\n", index, index);
        addresses.push(tree_address(&other, SourceType::ts()));
    }
    assert_eq!(parsed.program.body.len(), body);
    assert_eq!(parsed.program.source_text, text);
    assert_eq!(
        std::ptr::from_ref(&parsed.program) as usize,
        tree_address(text, SourceType::ts())
    );
    assert_eq!(addresses.len(), 40);
}

#[test]
fn a_kept_parse_outlives_its_scope_and_frees_once() {
    let text = "export const answer = 42\n";
    let kept = {
        let _scope = SharedParseScope::enter();
        let kept =
            shared_program(text, SourceType::ts().with_module(true)).expect("inside a scope");
        let clone = kept.clone();
        drop(kept);
        clone
    };
    PARSES.with_borrow(|parses| assert!(parses.is_none()));
    let parsed = kept.get();
    assert_eq!(parsed.program.source_text, text);
    assert_eq!(parsed.program.body.len(), 1);
    assert!(!parsed.panicked);
    let source = kept.0.source();
    assert_eq!(&*source.text, text);
    drop(kept);
    assert!(shared_program(text, SourceType::ts()).is_none());
}

#[test]
fn a_reader_of_one_parse_can_start_another_inside_its_callback() {
    let _scope = SharedParseScope::enter();
    let outer_text = "const outer = 1;\n";
    let inner_text = "const inner = 2;\n";
    let (outer_body, inner_body, inner_address) =
        with_shared_parse(outer_text, SourceType::ts(), |outer| {
            let (inner_body, inner_address) =
                with_shared_parse(inner_text, SourceType::ts(), |inner| {
                    (
                        inner.program.body.len(),
                        std::ptr::from_ref(&inner.program) as usize,
                    )
                });
            (outer.program.body.len(), inner_body, inner_address)
        });
    assert_eq!((outer_body, inner_body), (1, 1));
    assert_eq!(tree_address(inner_text, SourceType::ts()), inner_address);
}
