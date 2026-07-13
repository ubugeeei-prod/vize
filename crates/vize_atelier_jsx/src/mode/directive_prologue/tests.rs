use super::{ModeDirectiveSet, classify_source_directives};

#[test]
fn recognizes_comments_asi_and_conflicts_in_function_prologues() {
    let modes = classify_source_directives(
        r#"const App = () => { /* lead */ "use vue:vdom"
        'use vue:vapor'; return <main/>; };"#,
    );
    assert_eq!(
        modes,
        ModeDirectiveSet {
            vdom: true,
            vapor: true
        }
    );
}

#[test]
fn ignores_comments_literals_escapes_and_non_prologue_strings() {
    let modes = classify_source_directives(
        r#"
        // const Fake = () => { "use vue:vapor"; return <i/>; };
        const text = "const Fake = () => { 'use vue:vapor'; }";
        const template = `() => { "use vue:vapor"; }`;
        const regex = /=>\{"use vue:vapor";\}/;
        const escaped = () => { "use vue:\x76apor"; return <i/>; };
        const late = () => { const marker = 1; "use vue:vapor"; return <b/>; };
        const stable = () => { "use vue:vdom"; return <main/>; };
    "#,
    );
    assert_eq!(
        modes,
        ModeDirectiveSet {
            vdom: true,
            vapor: false
        }
    );
}

#[test]
fn ordinary_block_string_is_not_a_function_directive() {
    let modes =
        classify_source_directives(r#"if (ready) { "use vue:vapor"; } const App = () => <main/>;"#);
    assert_eq!(modes, ModeDirectiveSet::default());
}
