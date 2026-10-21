use super::generate_virtual_ts;
use vize_croquis::{Analyzer, AnalyzerOptions};

/// Generated names of `type` aliases, each with the braces open at its
/// declaration (as offsets of their `{`).
fn type_alias_blocks(code: &str) -> Vec<(&str, Vec<usize>)> {
    let mut open = Vec::new();
    let mut aliases = Vec::new();
    for (offset, ch) in code.char_indices() {
        match ch {
            '{' => open.push(offset),
            '}' => {
                open.pop();
            }
            't' if code[offset..].starts_with("type __") => {
                let name = &code[offset + "type ".len()..];
                let end = name
                    .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                    .unwrap_or(name.len());
                aliases.push((&name[..end], open.clone()));
            }
            _ => {}
        }
    }
    aliases
}

fn alias_is_visible(aliases: &[(&str, Vec<usize>)], name: &str, open: &[usize]) -> bool {
    aliases
        .iter()
        .any(|(alias, blocks)| *alias == name && open.starts_with(blocks))
}

#[test]
fn listener_contracts_are_not_shared_across_separate_guard_blocks() {
    // The `v-else` child closes the first listener's `if (enabled)` block,
    // so the second `<Toggle>` is checked in a new block with the same guard.
    let script = r#"import Toggle from './Toggle.vue'
import Radio from './Radio.vue'
const enabled = true
const flag = false
const onToggle = (value: unknown) => value
const onPick = (value: unknown) => value
"#;
    let template = r#"<div v-if="enabled">
  <Toggle @change="onToggle">
    <span v-if="flag">x</span>
    <Radio v-else @pick="onPick" />
  </Toggle>
  <Toggle @change="onToggle" />
</div>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let code = output.code.as_str();

    let aliases = type_alias_blocks(code);
    let mut open = Vec::new();
    let mut checked = 0;
    let mut outside = Vec::new();
    for (offset, ch) in code.char_indices() {
        match ch {
            '{' => open.push(offset),
            '}' => {
                open.pop();
            }
            '_' if code[offset..].starts_with("__Toggle_")
                && !code[..offset].ends_with("type ")
                && !code[..offset].ends_with('_') =>
            {
                let name = &code[offset..];
                let end = name
                    .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                    .unwrap_or(name.len());
                let name = &name[..end];
                let Some(_) = aliases.iter().find(|(alias, _)| *alias == name) else {
                    continue;
                };
                if !alias_is_visible(&aliases, name, &open) {
                    outside.push(name);
                }
                checked += 1;
            }
            _ => {}
        }
    }
    assert_eq!(
        outside,
        Vec::<&str>::new(),
        "listener aliases referenced outside their declaration blocks:\n{code}"
    );
    assert!(
        checked > 0,
        "no Toggle listener aliases were referenced:\n{code}"
    );
}
