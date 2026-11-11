use super::*;

fn tree_address(text: &str, source_type: SourceType) -> usize {
    with_shared_parse(text, source_type, |parsed| {
        std::ptr::from_ref(&parsed.program) as usize
    })
}

fn expression_address(text: &str) -> Option<usize> {
    with_shared_expression(text, SourceType::ts(), |expression| {
        expression.map(|expression| std::ptr::from_ref(expression) as usize)
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
fn expressions_share_apart_from_programs_and_keep_failures() {
    let private = with_shared_expression("(a) && b.c", SourceType::ts(), |expression| {
        expression.map(GetSpan::span)
    });
    let _scope = SharedParseScope::enter();
    let first = expression_address("(a) && b.c");
    assert!(first.is_some());
    assert_eq!(expression_address("(a) && b.c"), first);
    assert_ne!(expression_address("(a) || b.c"), first);
    assert_eq!(
        with_shared_expression("(a) && b.c", SourceType::ts(), |expression| {
            expression.map(GetSpan::span)
        }),
        private
    );
    assert_eq!(expression_address("a +"), None);
    assert_eq!(expression_address("a +"), None);
    tree_address("(a) && b.c", SourceType::ts());
    PARSES.with_borrow(|parses| {
        let parses = parses.as_ref().map(|parses| {
            (
                parses.programs.0.values().map(Vec::len).sum::<usize>(),
                parses.expressions.0.values().map(Vec::len).sum::<usize>(),
            )
        });
        assert_eq!(parses, Some((1, 3)));
    });
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
    // Without the unambiguous parse in the scope, the module goal parses itself.
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
