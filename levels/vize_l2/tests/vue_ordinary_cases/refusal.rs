use super::*;

#[test]
fn complete_neutral_facts_never_certify_other_statements_before_or_after_the_default()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for invalid in [
        "42;",
        "const value=1;",
        "let value=1;",
        "var value=1;",
        "function hidden(){}",
        "import 'dep';",
        "export {};",
    ] {
        for body in [
            format!("{invalid}export default {{}};"),
            format!("export default {{}};{invalid}"),
            format!(";{invalid};export default {{}};;"),
        ] {
            let source = format!("<script>{body}</script>");
            let original = Observed::new(&arena, &source)?;
            check(
                original
                    .syntax
                    .admitted_program()
                    .ok_or("actual syntax")?
                    .sole_default_export()
                    .is_some(),
            )?;
            let producer = original.producer(&arena)?;
            check(producer.ordinary_empty_script().is_none())?;
            let file = producer.finish().map_err(|_| "File")?;
            check(file.is_complete())?;
            equal(
                kind(original.checked(&file)?)?,
                OrdinaryIssueKind::UnsupportedSyntax,
            )?;
            equal(original.syntax.diagnostics().count(), 0)?;
        }
    }
    Ok(())
}

#[test]
fn actual_empty_direct_object_excludes_nonempty_options_and_all_expression_wrappers()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (lang, body) in [
        ("", "export default {name:'options'};"),
        ("", "export default {method(){}};"),
        ("", "export default {get value(){return 1}};"),
        ("", "export default {__proto__:null};"),
        ("", "export default ({ });"),
        ("", "export default [];"),
        ("", "export default null;"),
        ("", "export default function(){}"),
        ("", "export default class {}"),
        ("", "const value={};export default value;"),
        ("", "export default {}\n.value;"),
        ("", "export default {}\n(1);"),
        (" lang='ts'", "export default {} as const;"),
        (" lang='ts'", "export default {} satisfies {};"),
        (" lang='ts'", "export default <{}>{};"),
        (" lang='ts'", "export default {}!;"),
        (" lang='ts'", "export default {};export default {};"),
        (" lang='ts'", "export default interface Value {}"),
        (" lang='ts'", "type Value=number;export default {};"),
        (
            " lang='ts'",
            "declare const value:number;export default {};",
        ),
        (" lang='ts'", "namespace Value {}export default {};"),
        (" lang='ts'", "export type {};export default {};"),
    ] {
        let source = format!("<script{lang}>{body}</script>");
        let original = Observed::new(&arena, &source)?;
        check(original.syntax.admitted_program().is_some())?;
        let producer = original.producer(&arena)?;
        check(producer.ordinary_empty_script().is_none())?;
        let file = producer.finish().map_err(|_| "neutral File")?;
        check(original.checked(&file)?.is_err())?;
        equal(original.syntax.diagnostics().count(), 0)?;
    }
    Ok(())
}

#[test]
fn strict_original_directive_receipts_and_nonprologue_strings_remain_fail_closed()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for body in [
        "'\\1';export default {};",
        "'\\8';export default {};",
        "'\\9';export default {};",
        ";'use strict';export default {};",
        "export default {};'use strict';",
        "#!/usr/bin/env node\nexport default {};",
    ] {
        let source = format!("<script>{body}</script>");
        let original = Observed::new(&arena, &source)?;
        let admitted = original.syntax.admitted_program().ok_or("actual syntax")?;
        if body.starts_with("'\\") {
            check(admitted.has_legacy_literals())?;
        }
        let file = original.file(&arena)?;
        check(file.is_complete())?;
        equal(
            kind(original.checked(&file)?)?,
            OrdinaryIssueKind::UnsupportedSyntax,
        )?;
        equal(original.syntax.diagnostics().count(), 0)?;
    }
    for directive in ["'\\0'", "'\\x01'", "'\\u0001'", "'\\\\1'", "'雪'"] {
        let source = format!("<script>{directive};export default {{}};</script>");
        let original = Observed::new(&arena, &source)?;
        check(
            !original
                .syntax
                .admitted_program()
                .ok_or("original")?
                .has_legacy_literals(),
        )?;
        let file = original.file(&arena)?;
        check(original.checked(&file)?.is_ok())?;
    }
    for body in ["", ";;;", "'use strict';"] {
        let source = format!("<script>{body}</script>");
        let original = Observed::new(&arena, &source)?;
        let file = original.file(&arena)?;
        check(file.is_complete())?;
        equal(
            kind(original.checked(&file)?)?,
            OrdinaryIssueKind::UnsupportedSyntax,
        )?;
    }
    Ok(())
}
