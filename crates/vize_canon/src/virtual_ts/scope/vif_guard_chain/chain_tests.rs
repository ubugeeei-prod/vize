use super::{
    ActiveGuardPrefix, SuspendedChain, append_bare_chained_vif_guard_open,
    append_chained_vif_guard_close, append_chained_vif_guard_open,
};
use vize_carton::{String, append};

fn emit(guards: &[&str]) -> String {
    let mut ts = String::from("");
    for (index, guard) in guards.iter().enumerate() {
        append_chained_vif_guard_open(&mut ts, "  ", guard, "Inference-only guard");
        append!(ts, "    void check{index};\n");
        append_chained_vif_guard_close(&mut ts, "  ");
    }
    ts
}

#[test]
fn consecutive_branches_become_one_else_if_chain() {
    let ts = emit(&[
        "(a)",
        "(a)",
        "!(a) && (b)",
        "!(a) && !(b) && (c)",
        "!(a) && !(b) && !(c)",
    ]);
    let conditions: Vec<&str> = ts
        .lines()
        .filter(|line| line.contains("if (") || line.trim() == "else {")
        .map(str::trim)
        .collect();
    assert_eq!(
        conditions,
        ["if ((a)) {", "else if ((b)) {", "else if ((c)) {", "else {"]
    );
    assert_eq!(ts.matches("void check").count(), 5);
    assert_eq!(ts.matches('{').count(), ts.matches('}').count());
}

#[test]
fn an_enclosing_guard_is_factored_outside_the_sibling_chain() {
    let ts = emit(&[
        "(p) && (a)",
        "(p) && (a)",
        "(p) && !(a) && (b)",
        "(p) && !(a) && (b)",
        "(p) && !(a) && !(b)",
    ]);
    assert_eq!(
        ts.as_str(),
        concat!(
            "  // @ts-ignore Inference-only guard; authored v-if checks own diagnostics.\n",
            "  if ((p)) {\n",
            "  // @ts-ignore Inference-only guard; authored v-if checks own diagnostics.\n",
            "  if ((a)) {\n",
            "    void check0;\n",
            "    void check1;\n",
            "  }\n",
            "  // @ts-ignore Inference-only guard; authored v-if checks own diagnostics.\n",
            "  else if ((b)) {\n",
            "    void check2;\n",
            "    void check3;\n",
            "  }\n",
            "  // @ts-ignore Inference-only guard; authored v-if checks own diagnostics.\n",
            "  else {\n",
            "    void check4;\n",
            "  }\n",
            "  }\n",
        )
    );
}

#[test]
fn quoted_delimiters_do_not_split_an_authored_condition() {
    assert_eq!(
        emit(&["(ready) && (item.kind === ') && (')"]).as_str(),
        concat!(
            "  // @ts-ignore Inference-only guard; authored v-if checks own diagnostics.\n",
            "  if ((ready)) {\n",
            "  // @ts-ignore Inference-only guard; authored v-if checks own diagnostics.\n",
            "  if ((item.kind === ') && (')) {\n",
            "    void check0;\n",
            "  }\n",
            "  }\n",
        )
    );
}

#[test]
fn reopening_nested_branches_preserves_every_prior_body_offset() {
    let mut ts = String::from("");
    let mut mappings = Vec::new();
    for guard in [
        "!(hidden) && (ready && enabled) && (a)",
        "!(hidden) && (ready && enabled) && (a)",
        "!(hidden) && (ready && enabled) && !(a) && (b)",
        "!(hidden) && (ready && enabled) && !(a) && !(b)",
    ] {
        ts.push_str("  // @vize-map: prop -> 1:2\n");
        append_chained_vif_guard_open(&mut ts, "  ", guard, "Inference-only guard");
        let start = ts.len();
        ts.push_str("    void mapped;\n");
        mappings.push(start..ts.len());
        append_chained_vif_guard_close(&mut ts, "  ");
    }
    for mapping in mappings {
        assert_eq!(ts.get(mapping), Some("    void mapped;\n"));
    }
    let conditions: Vec<_> = ts
        .lines()
        .filter(|line| !line.contains("//") && !line.contains("void"))
        .collect();
    assert_eq!(
        conditions,
        [
            "  if (!(hidden) && (ready && enabled)) {",
            "  if ((a)) {",
            "  }",
            "  else if ((b)) {",
            "  }",
            "  else {",
            "  }",
            "  }",
        ]
    );
}

#[test]
fn one_hundred_eighty_nested_siblings_emit_each_condition_once() {
    let mut prefix = String::from("(parent)");
    let mut guards = Vec::new();
    let mut expected = vec![String::from("if ((parent)) {")];
    for index in 0..180 {
        let mut guard = prefix.clone();
        append!(guard, " && (branch{index})");
        guards.push(guard);
        append!(prefix, " && !(branch{index})");
        let mut condition = String::from(if index == 0 { "if" } else { "else if" });
        append!(condition, " ((branch{index})) {{");
        expected.push(condition);
    }
    guards.push(prefix);
    expected.push(String::from("else {"));
    let ts = emit(&guards.iter().map(String::as_str).collect::<Vec<_>>());
    let conditions: Vec<_> = ts
        .lines()
        .filter(|line| line.contains("if (") || line.trim() == "else {")
        .map(str::trim)
        .collect();
    assert_eq!(
        conditions,
        expected.iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert_eq!(ts.matches("void check").count(), 181);
    assert_eq!(ts.matches('{').count(), 182);
    assert_eq!(ts.matches('}').count(), 182);
}

#[test]
fn unrelated_guards_open_separate_blocks() {
    let ts = emit(&["(a)", "(b)"]);
    assert_eq!(ts.matches("else").count(), 0, "{ts}");
    assert_eq!(ts.matches("if (").count(), 2, "{ts}");
}

#[test]
fn comment_lines_between_blocks_do_not_stop_folding() {
    let mut ts = String::from("");
    for guard in ["(a)", "(a)", "!(a) && (b)"] {
        ts.push_str("  // @vize-map: prop -> 1:2\n");
        append_chained_vif_guard_open(&mut ts, "  ", guard, "Inference-only guard");
        ts.push_str("    void check;\n");
        append_chained_vif_guard_close(&mut ts, "  ");
    }
    assert_eq!(ts.matches("if (").count(), 2, "{ts}");
    assert_eq!(ts.matches("else if ((b))").count(), 1, "{ts}");
    assert_eq!(ts.matches("@vize-map").count(), 3, "{ts}");
    assert_eq!(ts.matches('{').count(), ts.matches('}').count(), "{ts}");
}

#[test]
fn a_block_after_other_output_is_not_folded() {
    let mut ts = String::from("");
    append_chained_vif_guard_open(&mut ts, "  ", "(a)", "Inference-only guard");
    append_chained_vif_guard_close(&mut ts, "  ");
    ts.push_str("  const between = 1;\n");
    append_chained_vif_guard_open(&mut ts, "  ", "!(a) && (b)", "Inference-only guard");
    append_chained_vif_guard_close(&mut ts, "  ");
    assert_eq!(ts.matches("else").count(), 0, "{ts}");
}

#[test]
fn an_active_prefix_drops_covered_terms_and_keeps_the_remainder() {
    let prefix = ActiveGuardPrefix::push("(ready) && (open)");
    let mut ts = String::from("");
    append_chained_vif_guard_open(&mut ts, "  ", "(ready) && (open)", "Inference-only guard");
    append!(ts, "    void covered;\n");
    append_chained_vif_guard_close(&mut ts, "  ");
    append_chained_vif_guard_open(
        &mut ts,
        "  ",
        "(ready) && (open) && (extra)",
        "Inference-only guard",
    );
    append!(ts, "    void extra;\n");
    append_chained_vif_guard_close(&mut ts, "  ");
    assert_eq!(
        ts.as_str(),
        concat!(
            "    void covered;\n",
            "  // @ts-ignore Inference-only guard; authored v-if checks own diagnostics.\n",
            "  if ((extra)) {\n",
            "    void extra;\n",
            "  }\n",
        )
    );
    drop(prefix);
}

#[test]
fn a_bare_chain_omits_the_ignore_comment() {
    let mut ts = String::from("");
    for guard in ["(a)", "!(a) && (b)"] {
        append_bare_chained_vif_guard_open(&mut ts, "  ", guard);
        append!(ts, "    void check;\n");
        append_chained_vif_guard_close(&mut ts, "  ");
    }
    assert_eq!(
        ts.as_str(),
        concat!(
            "  if ((a)) {\n",
            "    void check;\n",
            "  }\n",
            "  else if ((b)) {\n",
            "    void check;\n",
            "  }\n",
        )
    );
}

#[test]
fn a_suspended_body_does_not_steal_the_outer_chain() {
    let mut ts = String::from("");
    append_chained_vif_guard_open(&mut ts, "  ", "(a)", "Inference-only guard");
    let suspended = SuspendedChain::suspend();
    append_chained_vif_guard_open(&mut ts, "    ", "(inner)", "Inference-only guard");
    append!(ts, "      void inner;\n");
    append_chained_vif_guard_close(&mut ts, "    ");
    drop(suspended);
    append!(ts, "    void outer;\n");
    append_chained_vif_guard_close(&mut ts, "  ");
    append_chained_vif_guard_open(&mut ts, "  ", "!(a) && (b)", "Inference-only guard");
    append!(ts, "    void next;\n");
    append_chained_vif_guard_close(&mut ts, "  ");
    assert_eq!(ts.matches("else if ((b))").count(), 1, "{ts}");
    assert!(ts.contains("void outer"), "{ts}");
    assert_eq!(ts.matches('{').count(), ts.matches('}').count(), "{ts}");
}
