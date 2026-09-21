//! A generic `<script setup>` is compiled into a function, and the component
//! is exported out of it. A class, enum or interface the block declares is
//! then a private name of that export, which a project that emits declarations
//! reports as `TS4025` where the block begins. Type aliases are serialized
//! structurally and stay silent.
//!
//! The virtual module keeps such declarations at module scope so other files
//! can import the component's types, which makes TypeScript itself silent; the
//! diagnostic is restored here, once per file, for the first private name the
//! component's macros expose.

use crate::batch::{Diagnostic, SfcBlockType, VirtualProject};
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{cstr, line_index::LineIndex};
use vize_croquis::script_parser::parse_program_for_analysis;

pub(super) const PRIVATE_NAME_CODE: u32 = 4025;
const MACROS: [&str; 4] = ["defineProps", "defineEmits", "defineSlots", "defineModel"];

pub(super) fn apply(diagnostics: &mut Vec<Diagnostic>, project: &VirtualProject) {
    if !project.authored_config_emits_declarations() {
        return;
    }
    for file in project.virtual_files_sorted() {
        if file
            .original_path
            .extension()
            .is_none_or(|extension| extension != "vue")
        {
            continue;
        }
        let Some(source) = project.original_content_for_virtual(&file.virtual_path) else {
            continue;
        };
        let Some((offset, name)) = first_private_name(source) else {
            continue;
        };
        let (line, column) = LineIndex::new(source).line_col(offset);
        diagnostics.push(Diagnostic {
            file: file.original_path.clone(),
            line,
            column,
            message: cstr!(
                "Exported variable '__default__' has or is using private name '{name}'."
            ),
            code: Some(PRIVATE_NAME_CODE),
            severity: 1,
            block_type: Some(SfcBlockType::ScriptSetup),
        });
    }
}

/// The start of the generic `<script setup>` content and the first locally
/// declared class, enum or interface a macro's type arguments name.
fn first_private_name(source: &str) -> Option<(usize, &str)> {
    if !source.contains("generic") {
        return None;
    }
    let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
    let setup = descriptor.script_setup.as_ref()?;
    setup.attrs.get("generic")?;
    let content: &str = &setup.content;
    let start = setup.loc.start;

    let allocator = Allocator::default();
    let parsed = parse_program_for_analysis(&allocator, content, SourceType::ts());
    if parsed.panicked {
        return None;
    }
    let private_names: Vec<&str> = parsed
        .program
        .body
        .iter()
        .filter_map(|statement| match statement {
            Statement::TSInterfaceDeclaration(declaration) => Some(declaration.id.name.as_str()),
            Statement::TSEnumDeclaration(declaration) => Some(declaration.id.name.as_str()),
            Statement::ClassDeclaration(declaration) => {
                declaration.id.as_ref().map(|id| id.name.as_str())
            }
            // An exported declaration is lifted to the module, and an alias is
            // serialized structurally.
            _ => None,
        })
        .collect();
    if private_names.is_empty() {
        return None;
    }
    let name = MACROS
        .iter()
        .flat_map(|name| content.match_indices(name))
        .filter_map(|(at, name)| type_arguments(&content[at + name.len()..]))
        .flat_map(identifiers)
        .find(|identifier| private_names.contains(identifier))?;
    // The name borrows from `content`, which borrows from `source`.
    let at = source.find(name)?;
    Some((start, &source[at..at + name.len()]))
}

/// The text between the angle brackets that directly follow a macro name.
fn type_arguments(after_macro: &str) -> Option<&str> {
    let rest = after_macro.strip_prefix('<')?;
    let mut depth = 1usize;
    for (index, character) in rest.char_indices() {
        match character {
            '<' => depth += 1,
            '>' if depth == 1 => return Some(&rest[..index]),
            '>' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn identifiers(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| !(character.is_alphanumeric() || matches!(character, '_' | '$')))
        .filter(|word| !word.is_empty())
}

#[cfg(test)]
mod tests {
    use super::first_private_name;

    #[test]
    fn a_local_interface_in_a_generic_block_is_a_private_name() {
        let source = "<script lang=\"ts\" setup generic>\ninterface Props {\n\titem: any;\n}\n\ndefineProps<Props>();\n</script>\n";
        assert_eq!(first_private_name(source), Some((32, "Props")));
    }

    #[test]
    fn aliases_exports_and_non_generic_blocks_are_not() {
        for source in [
            "<script lang=\"ts\" setup generic=\"T\">\ntype Props = { item: T };\ndefineProps<Props>();\n</script>\n",
            "<script lang=\"ts\" setup generic=\"T\">\nexport interface Props { item: T }\ndefineProps<Props>();\n</script>\n",
            "<script lang=\"ts\" setup>\ninterface Props { item: any }\ndefineProps<Props>();\n</script>\n",
            "<script lang=\"ts\" setup generic=\"T\">\ninterface Unused { item: T }\ndefineProps<{ item: T }>();\n</script>\n",
        ] {
            assert_eq!(first_private_name(source), None, "{source}");
        }
    }
}
