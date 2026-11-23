//! Expanded CSS for a complete original SCSS program.
use vize_l0::String;
use vize_l2::lang::scss::ScssProgram;

pub fn emit(program: &ScssProgram<'_>) -> String {
    let mut css = String::default();
    let mut previous_group = None;
    for rule in program.runs() {
        if let Some(previous) = previous_group {
            css.push('\n');
            if previous != rule.group {
                css.push('\n');
            }
        }
        css.push_str(&rule.selector);
        css.push_str(" {\n");
        for property in &rule.declarations {
            css.push_str("  ");
            css.push_str(property.name);
            css.push_str(": ");
            css.push_str(&property.value);
            css.push_str(";\n");
        }
        css.push('}');
        previous_group = Some(rule.group);
    }
    css
}
