//! Shared parses are lent across reads, kept past their handle and read
//! back. The Miri lane runs these tests as a check of safe code.

use super::*;

fn tree_address(parses: Option<&ScriptParses<'_>>, text: &str, source_type: SourceType) -> usize {
    with_shared_parse(parses, text, source_type, |parsed| {
        std::ptr::from_ref(&parsed.program) as usize
    })
}

#[test]
fn a_handle_parses_equal_text_and_dialect_once() {
    let arena = Allocator::default();
    let parses = ScriptParses::new(&arena);
    let parses = Some(&parses);
    let text: Box<str> = Box::from("const value = 1;\n");
    let copy: Box<str> = Box::from(&*text);
    assert_eq!(
        tree_address(parses, &text, SourceType::ts()),
        tree_address(parses, &copy, SourceType::ts())
    );
    assert_ne!(
        tree_address(parses, &text, SourceType::ts()),
        tree_address(parses, &text, SourceType::ts().with_module(true))
    );
    assert_ne!(
        tree_address(parses, &text, SourceType::ts()),
        tree_address(parses, "const value = 2;\n", SourceType::ts())
    );
}

#[test]
fn a_shared_parse_matches_a_private_one() {
    let text = "import { ref } from 'vue'\nconst count = ref(\n";
    let summary = |parsed: &ParserReturn<'_>| {
        (
            parsed.panicked,
            parsed.diagnostics.len(),
            parsed.program.body.len(),
        )
    };
    let private = with_shared_parse(None, text, SourceType::ts(), summary);
    let arena = Allocator::default();
    let parses = ScriptParses::new(&arena);
    for _ in 0..2 {
        let shared = with_shared_parse(Some(&parses), text, SourceType::ts(), summary);
        assert_eq!(shared, private);
    }
}

#[test]
fn without_a_handle_every_read_parses_privately_and_handles_share_nothing() {
    let text = "let a = 1;";
    let arena = Allocator::default();
    let first = ScriptParses::new(&arena);
    let second = ScriptParses::new(&arena);
    let shared = tree_address(Some(&first), text, SourceType::ts());
    assert_eq!(tree_address(Some(&first), text, SourceType::ts()), shared);
    assert_ne!(tree_address(Some(&second), text, SourceType::ts()), shared);
    assert_ne!(tree_address(None, text, SourceType::ts()), shared);
}

#[test]
fn a_ts_or_tsx_extension_shares_the_dialect_parse_and_others_stay_apart() {
    let text = "const value = <T,>(input: T) => input;\n";
    let from_path = |path| SourceType::from_path(path).unwrap_or_default();
    let summary = |parsed: &ParserReturn<'_>| {
        (
            parsed.panicked,
            parsed.diagnostics.len(),
            parsed.program.body.len(),
        )
    };
    let private = with_shared_parse(None, text, from_path("script.ts"), summary);
    let arena = Allocator::default();
    let parses = ScriptParses::new(&arena);
    let parses = Some(&parses);
    let ts = tree_address(parses, text, SourceType::ts());
    assert_eq!(tree_address(parses, text, from_path("script.ts")), ts);
    assert_eq!(
        tree_address(parses, text, from_path("script.tsx")),
        tree_address(parses, text, SourceType::tsx())
    );
    assert_ne!(tree_address(parses, text, from_path("script.tsx")), ts);
    assert_eq!(
        with_shared_parse(parses, text, from_path("script.ts"), summary),
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
        assert_ne!(tree_address(parses, text, source_type), ts, "{separate}");
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
            let arena = Allocator::default();
            let parses = ScriptParses::new(&arena);
            let parses = Some(&parses);
            let unambiguous = tree_address(parses, text, dialect);
            let module = tree_address(parses, text, dialect.with_module(true));
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
    let arena = Allocator::default();
    let parses = ScriptParses::new(&arena);
    let parses = Some(&parses);
    let text = "import x from 'x'\n";
    let module = tree_address(parses, text, SourceType::ts().with_module(true));
    assert_ne!(tree_address(parses, text, SourceType::ts()), module);
}

#[test]
fn a_lent_parse_stays_readable_while_the_handle_grows() {
    let arena = Allocator::default();
    let parses = ScriptParses::new(&arena);
    let text = "import { ref } from 'vue'\nconst count = ref(0)\n";
    let kept = parses.program(text, SourceType::ts());
    let parsed = kept.get();
    let body = parsed.program.body.len();
    // Enough new parses to reallocate the handle's list several times while
    // `parsed` still borrows the first one.
    let mut addresses = std::vec::Vec::new();
    for index in 0..40u32 {
        let other = vize_carton::cstr!("let value{} = {};\n", index, index);
        addresses.push(tree_address(Some(&parses), &other, SourceType::ts()));
    }
    assert_eq!(parsed.program.body.len(), body);
    assert_eq!(parsed.program.source_text, text);
    assert_eq!(
        std::ptr::from_ref(&parsed.program) as usize,
        tree_address(Some(&parses), text, SourceType::ts())
    );
    assert_eq!(addresses.len(), 40);
}

#[test]
fn a_kept_parse_outlives_its_handle() {
    let arena = Allocator::default();
    let text = "export const answer = 42\n";
    let kept = {
        let parses = ScriptParses::new(&arena);
        let kept = parses.program(text, SourceType::ts().with_module(true));
        let clone = kept.clone();
        drop(kept);
        clone
    };
    let parsed = kept.get();
    assert_eq!(parsed.program.source_text, text);
    assert_eq!(parsed.program.body.len(), 1);
    assert!(!parsed.panicked);
    assert_eq!(kept.0.text, text);
    drop(kept);
}

#[test]
fn a_reader_of_one_parse_can_start_another_inside_its_callback() {
    let arena = Allocator::default();
    let parses = ScriptParses::new(&arena);
    let parses = Some(&parses);
    let outer_text = "const outer = 1;\n";
    let inner_text = "const inner = 2;\n";
    let (outer_body, inner_body, inner_address) =
        with_shared_parse(parses, outer_text, SourceType::ts(), |outer| {
            let (inner_body, inner_address) =
                with_shared_parse(parses, inner_text, SourceType::ts(), |inner| {
                    (
                        inner.program.body.len(),
                        std::ptr::from_ref(&inner.program) as usize,
                    )
                });
            (outer.program.body.len(), inner_body, inner_address)
        });
    assert_eq!((outer_body, inner_body), (1, 1));
    assert_eq!(
        tree_address(parses, inner_text, SourceType::ts()),
        inner_address
    );
}
