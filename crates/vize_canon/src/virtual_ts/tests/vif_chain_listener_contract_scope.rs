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

fn matching_brace(code: &str, opening_brace: usize) -> Option<usize> {
    let mut depth = 0;
    for (relative, ch) in code[opening_brace..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(opening_brace + relative);
                }
            }
            _ => {}
        }
    }
    None
}

#[test]
fn scoped_slot_alias_narrowing_wraps_an_arrow_listener_in_its_guard() {
    let script = r#"import Child from './Child.vue'
import List from './List.vue'
type File = { kind: 'file'; size: number }
type Folder = { kind: 'folder' }
const items: (File | Folder)[] = []
const isFile = (item: File | Folder): item is File => item.kind === 'file'
const select = (file: File) => file.size
"#;
    let template = r#"<List :items="items">
  <template #item="{ item }">
    <Child v-if="isFile(item)" @click="() => select(item)" />
  </template>
</List>"#;
    let allocator = vize_carton::Allocator::new();
    let (root, errors) = vize_armature::parse(&allocator, template);
    assert!(errors.is_empty(), "{errors:?}");
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);
    let code = output.code.as_str();

    let slot = code
        .find("void function _slot_item_")
        .expect("scoped slot function");
    let handler = code[slot..]
        .find("// @click handler")
        .map(|offset| slot + offset)
        .expect("listener callback");
    let callback = code[handler..]
        .find("() => select(item)")
        .map(|offset| handler + offset)
        .expect("authored callback");
    let guard = code[..handler]
        .rfind("if ((isFile(item))) {")
        .unwrap_or_else(|| panic!("callback guard:\n{code}"));
    let opening_brace = code[guard..handler]
        .find('{')
        .map(|offset| guard + offset)
        .expect("callback guard opening brace");
    let closing_brace = matching_brace(code, opening_brace)
        .unwrap_or_else(|| panic!("callback guard closing brace:\n{code}"));

    assert!(
        handler < callback && callback < closing_brace,
        "the stable slot parameter's guard must enclose the authored callback:\n{code}"
    );
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
