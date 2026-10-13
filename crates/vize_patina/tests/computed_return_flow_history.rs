//! Issue #8828: every reachable computed getter completion must return a value.
use vize_patina::{LintDiagnostic, LintPreset, LintResult, Linter};

const RULE: &str = "script/return-in-computed-property";
fn linter() -> Linter {
    Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![RULE.into()]))
}

fn offset(source: &str, line: usize, column: usize) -> u32 {
    let before: usize = source
        .split_inclusive('\n')
        .take(line - 1)
        .map(str::len)
        .sum();
    let prefix: usize = source
        .lines()
        .nth(line - 1)
        .unwrap()
        .chars()
        .take(column - 1)
        .map(char::len_utf8)
        .sum();
    (before + prefix) as u32
}

#[test]
fn three_original_n8n_getters_keep_complete_authored_reports() {
    for (filename, source, start_line, start_column, end_line, end_column) in [
        (
            "RunStatusPill.vue",
            include_str!(
                "../../../tests/_fixtures/differential/linter/computed-return-flow-8828/RunStatusPill.vue.fixture"
            ),
            36,
            27,
            47,
            2,
        ),
        (
            "PreferenceScopeBadge.vue",
            include_str!(
                "../../../tests/_fixtures/differential/linter/computed-return-flow-8828/PreferenceScopeBadge.vue.fixture"
            ),
            33,
            24,
            54,
            2,
        ),
        (
            "WorkflowReviewActivityEventEntry.vue",
            include_str!(
                "../../../tests/_fixtures/differential/linter/computed-return-flow-8828/WorkflowReviewActivityEventEntry.vue.fixture"
            ),
            112,
            11,
            191,
            2,
        ),
    ] {
        let start = offset(source, start_line, start_column);
        let end = offset(source, end_line, end_column);
        let diagnostic = LintDiagnostic::error(RULE,
            "Expected to return a value in this computed property.", start, end)
            .with_label("computed getter without a return value", start, end)
            .with_help("A computed getter must return the derived value. Add a `return` that yields a value on every path through the getter.");
        let expected = LintResult {
            filename: filename.into(),
            warning_count: 0,
            error_count: 1,
            diagnostics: vec![diagnostic],
        };
        let actual = linter().lint_sfc(source, filename);
        assert_eq!(
            format!("{actual:#?}"),
            format!("{expected:#?}"),
            "{filename}"
        );
    }
}

#[test]
fn reachable_completion_controls() {
    for (body, errors) in [
        ("if(ok)return 1", 1),
        ("if(ok)return 1;return 2", 0),
        ("if(ok)return 1;else return 2", 0),
        ("if(ok)return;else return 1", 1),
        ("switch(kind){case 'a':return 1;case 'b':return 2}", 1),
        ("switch(kind){case 'a':return 1;default:return 2}", 0),
        ("switch(kind){case 'a':break;default:return 2}", 1),
        (
            "switch(kind){case 'a':case 'b':return 1;default:return 2}",
            0,
        ),
        ("switch(kind){case 'a':break;default:return 2}return 3", 0),
        ("switch(kind){default:return 1;case 'a':break}return 2", 0),
        ("const nested=()=>{return 1}", 1),
        ("function nested(){return 1}", 1),
        ("while(ok){return 1}", 1),
        ("while(true){if(ok)return 1}", 0),
        ("while(true){if(ok)break;return 1}", 1),
        ("while(false){return;}return 1", 0),
        ("do{return 1}while(ok)", 0),
        ("do{if(ok)return 1}while(again)", 1),
        ("for(;;){return 1}", 0),
        ("for(const item of items){return item}", 1),
        (
            "for(const item of items){if(ok)continue;return item}return 2",
            0,
        ),
        (
            "outer:while(true){switch(kind){case 'a':continue outer;default:return 1}}",
            0,
        ),
        ("outer:while(true){if(ok)break outer;return 1}", 1),
        ("block:{break block}return 1", 0),
        ("try{return 1}catch{}", 0),
        ("try{return call()}catch{}", 1),
        ("try{return call()}catch{return 2}", 0),
        ("try{return 1}finally{}", 0),
        ("try{return 1}finally{return;}", 1),
        ("try{return;}finally{return 1}", 0),
        ("try{do{return 1}while(call())}catch{}", 0),
        ("throw Error()", 1),
    ] {
        let source = format!("import {{computed}} from 'vue';computed(()=>{{{body}}})");
        let actual = linter().lint_script(&source, "flow.ts");
        assert_eq!(actual.error_count, errors, "{body}: {actual:#?}");
        assert_eq!(actual.warning_count, 0, "{body}: {actual:#?}");
        assert_eq!(actual.diagnostics.len(), errors, "{body}: {actual:#?}");
    }
}

#[test]
fn options_and_writable_getters_follow_the_same_paths() {
    for (source, errors) in [
        ("export default{computed:{value(){if(this.ok)return 1}}}", 1),
        (
            "export default{computed:{value(){if(this.ok)return 1;return 2}}}",
            0,
        ),
        ("const value=computed({get(){if(ok)return 1},set(v){}})", 1),
        (
            "const value=computed({get(){if(ok)return 1;return 2},set(v){}})",
            0,
        ),
        ("const value=computed(()=>1)", 0),
    ] {
        let actual = linter().lint_script(source, "options.ts");
        assert_eq!(actual.error_count, errors, "{source}: {actual:#?}");
        assert_eq!(actual.diagnostics.len(), errors, "{source}: {actual:#?}");
    }
}
