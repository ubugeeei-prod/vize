//! JSDoc on runtime `defineProps({ ... })` keys.
//!
//! The projection republishes runtime props as a `Props` type literal. Copying
//! each key's JSDoc onto its member lets quick info on a parent's attribute or
//! a `props.x` read show it. Comments are trivia: types, diagnostics and check
//! cost are unchanged.

use vize_croquis::macros::{MacroKind, MacroTracker};

/// `(key, JSDoc block)` for every top-level key directly preceded by `/** */`.
pub(super) fn runtime_prop_docs(object: &str) -> Vec<(&str, &str)> {
    let bytes = object.as_bytes();
    let mut docs = Vec::new();
    let mut depth = 0_usize;
    let mut expects_key = false;
    let mut doc = None;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let end = object
                    .get(index + 2..)
                    .and_then(|rest| rest.find("*/"))
                    .map_or(bytes.len(), |end| index + 2 + end + 2);
                let block = object.get(index..end).unwrap_or_default();
                doc = (depth == 1 && expects_key && block.starts_with("/**") && block.len() > 4)
                    .then_some(block);
                index = end;
                continue;
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = object
                    .get(index..)
                    .and_then(|rest| rest.find('\n'))
                    .map_or(bytes.len(), |end| index + end);
                doc = None;
                continue;
            }
            _ if byte.is_ascii_whitespace() => {
                index += 1;
                continue;
            }
            b'\'' | b'"' | b'`' => {
                let end = string_end(bytes, index);
                if depth == 1 && expects_key {
                    if let (Some(doc), Some(key)) = (doc, object.get(index + 1..end - 1)) {
                        docs.push((key, doc));
                    }
                    expects_key = false;
                }
                index = end;
                doc = None;
                continue;
            }
            b'{' | b'(' | b'[' => {
                depth += 1;
                expects_key = depth == 1 && byte == b'{';
            }
            b'}' | b')' | b']' => depth = depth.saturating_sub(1),
            b',' if depth == 1 => expects_key = true,
            _ if depth == 1 && expects_key && is_identifier_start(byte) => {
                let end = bytes
                    .get(index..)
                    .and_then(|rest| rest.iter().position(|byte| !is_identifier_part(*byte)))
                    .map_or(bytes.len(), |length| index + length);
                if let (Some(doc), Some(key)) = (doc, object.get(index..end)) {
                    docs.push((key, doc));
                }
                expects_key = false;
                index = end;
                doc = None;
                continue;
            }
            _ if depth == 1 => expects_key = false,
            _ => {}
        }
        doc = None;
        index += 1;
    }
    docs
}

/// A runtime key names the prop Vue camelizes it to (`item-kind` → `itemKind`).
pub(super) fn names_prop(key: &str, prop: &str) -> bool {
    let mut expected = prop.chars();
    let mut upper = false;
    for character in key.chars() {
        if character == '-' {
            upper = true;
            continue;
        }
        let character = if upper {
            character.to_ascii_uppercase()
        } else {
            character
        };
        upper = false;
        if expected.next() != Some(character) {
            return false;
        }
    }
    expected.next().is_none()
}

/// JSDoc directly attached to the variable declaration that owns a
/// `defineModel` call.
///
/// The retained model declaration points at the explicit name literal (or the
/// macro identifier for an implicit `modelValue`), so recover the containing
/// call first. Only a direct `const x = defineModel(...)` initializer inherits
/// the preceding block; wrappers and intervening comments deliberately do not.
pub(super) fn model_doc<'a>(script: &'a str, macros: &MacroTracker, name: &str) -> Option<&'a str> {
    let (declaration_start, declaration_end) = macros.model_declaration(name)?;
    let call = macros.all_calls().iter().find(|call| {
        call.kind == MacroKind::DefineModel
            && call.start <= declaration_start
            && declaration_end <= call.end
    })?;
    let before_call = script.get(..call.start as usize)?;
    let doc_start = before_call.rfind("/**")?;
    let rest = script.get(doc_start..call.start as usize)?;
    let doc_end = rest.find("*/")?.checked_add(2)?;
    let doc = rest.get(..doc_end)?;
    let declaration_prefix = rest.get(doc_end..)?.trim();
    let declaration_prefix = declaration_prefix
        .strip_prefix("const ")
        .or_else(|| declaration_prefix.strip_prefix("let "))
        .or_else(|| declaration_prefix.strip_prefix("var "))?;
    let (_, initializer) = declaration_prefix.split_once('=')?;
    initializer.trim().is_empty().then_some(doc)
}

fn string_end(bytes: &[u8], start: usize) -> usize {
    let quote = bytes.get(start).copied();
    let mut index = start + 1;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'\\' {
            index += 2;
            continue;
        }
        index += 1;
        if Some(byte) == quote {
            return index;
        }
    }
    bytes.len()
}

fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte == b'$' || byte.is_ascii_alphabetic()
}

fn is_identifier_part(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::{model_doc, runtime_prop_docs};
    use vize_croquis::script_parser::parse_script_setup;

    #[test]
    fn runtime_keys_name_their_camelized_props() {
        assert!(super::names_prop("title", "title"));
        assert!(super::names_prop("item-kind", "itemKind"));
        assert!(!super::names_prop("item-kind", "item"));
        assert!(!super::names_prop("title", "titles"));
    }

    #[test]
    fn only_top_level_keys_directly_after_a_jsdoc_block_carry_it() {
        let object = r#"{
  /** The title shown above. */
  title: { type: String, required: true },
  count: Number,
  /**
   * Kebab key.
   */
  'item-kind': { type: String, default: () => /** inner */ 'a' },
  /* plain */ plain: String,
  /** detached */
  // separated
  detached: String,
  nested: { /** not a prop */ type: Object },
}"#;
        assert_eq!(
            runtime_prop_docs(object),
            vec![
                ("title", "/** The title shown above. */"),
                ("item-kind", "/**\n   * Kebab key.\n   */"),
            ]
        );
    }

    #[test]
    fn direct_model_declarations_inherit_their_jsdoc() {
        let source = r#"/** The edited name. */
const name = defineModel<string>('name')
/** Detached. */
const wrapped = use(defineModel<number>('count'))
"#;
        let summary = parse_script_setup(source).into_croquis();
        assert_eq!(
            model_doc(source, &summary.macros, "name"),
            Some("/** The edited name. */")
        );
        assert_eq!(model_doc(source, &summary.macros, "count"), None);
    }
}
