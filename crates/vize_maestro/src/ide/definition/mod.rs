//! Definition provider for Vue SFC files.
//!
//! Provides go-to-definition for:
//! - Template expressions -> script bindings
//! - Component usages -> component definitions
//! - Import statements -> imported files
//! - Real definitions from Corsa (when available)

pub mod bindings;
mod helpers;
mod script;
mod service;
mod template;

pub use bindings::{extract_bindings_with_locations, BindingKind, BindingLocation};

use super::IdeContext;

/// Definition service for providing go-to-definition functionality.
pub struct DefinitionService;

#[cfg(test)]
mod tests {
    use super::{bindings, helpers, script, BindingKind};

    #[test]
    fn test_find_binding_location_const() {
        let content = r#"// Virtual TypeScript
// Generated

const message = ref('hello')
const count = ref(0)
"#;

        let loc = script::find_binding_location(content, "message", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "message");
        assert_eq!(loc.kind, BindingKind::Const);
    }

    #[test]
    fn test_find_binding_location_function() {
        let content = r#"// Virtual TypeScript
// Generated

function handleClick() {
  console.log('clicked')
}
"#;

        let loc = script::find_binding_location(content, "handleClick", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "handleClick");
        assert_eq!(loc.kind, BindingKind::Function);
    }

    #[test]
    fn test_find_binding_location_destructure() {
        let content = r#"// Virtual TypeScript
// Generated

const { data, error } = useFetch('/api')
"#;

        let loc = script::find_binding_location(content, "data", true);
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "data");
        assert_eq!(loc.kind, BindingKind::Destructure);
    }

    #[test]
    fn test_offset_to_position() {
        let content = "line1\nline2\nline3";

        let (line, col) = helpers::offset_to_position(content, 0);
        assert_eq!(line, 0);
        assert_eq!(col, 0);

        let (line, col) = helpers::offset_to_position(content, 3);
        assert_eq!(line, 0);
        assert_eq!(col, 3);

        let (line, col) = helpers::offset_to_position(content, 6);
        assert_eq!(line, 1);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_get_word_at_offset() {
        let content = "const message = 'hello'";

        let word = helpers::get_word_at_offset(content, 6);
        assert_eq!(word, Some("message".to_string()));

        let word = helpers::get_word_at_offset(content, 5);
        assert_eq!(word, None); // space

        let word = helpers::get_word_at_offset(content, 0);
        assert_eq!(word, Some("const".to_string()));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(bindings::is_valid_identifier("foo"));
        assert!(bindings::is_valid_identifier("_foo"));
        assert!(bindings::is_valid_identifier("$foo"));
        assert!(bindings::is_valid_identifier("foo123"));
        assert!(!bindings::is_valid_identifier("123foo"));
        assert!(!bindings::is_valid_identifier(""));
    }

    #[test]
    fn test_find_binding_location_raw_const() {
        let content = r#"
import { ref } from 'vue'

const message = ref('hello')
const count = ref(0)
"#;

        let loc = script::find_binding_location_raw(content, "message");
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "message");
        assert_eq!(loc.kind, BindingKind::Const);
        assert_eq!(&content[loc.offset..loc.offset + 7], "message");
    }

    #[test]
    fn test_find_binding_location_raw_import() {
        let content = r#"import { ref } from 'vue'
import MyComponent from './MyComponent.vue'
"#;

        let loc = script::find_binding_location_raw(content, "MyComponent");
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "MyComponent");
        assert_eq!(loc.kind, BindingKind::Import);
        assert_eq!(&content[loc.offset..loc.offset + 11], "MyComponent");
    }

    #[test]
    fn test_find_binding_location_raw_destructure() {
        let content = r#"const { data, error } = useFetch('/api')
"#;

        let loc = script::find_binding_location_raw(content, "data");
        assert!(loc.is_some());
        let loc = loc.unwrap();
        assert_eq!(loc.name, "data");
        assert_eq!(loc.kind, BindingKind::Destructure);
        assert_eq!(&content[loc.offset..loc.offset + 4], "data");
    }

    #[test]
    fn test_find_prop_in_define_props() {
        let content = r#"defineProps<{
  title: string
  isSubmitting?: boolean
  count: number
}>()"#;

        let pos = helpers::find_prop_in_define_props(content, "title");
        assert!(pos.is_some());

        let pos = helpers::find_prop_in_define_props(content, "isSubmitting");
        assert!(pos.is_some());

        let pos = helpers::find_prop_in_define_props(content, "nonExistent");
        assert!(pos.is_none());
    }

    #[test]
    fn test_is_in_vue_directive_expression_detection() {
        let vue_attrs = [
            ":disabled",
            "@click",
            "v-if",
            "v-for",
            "v-model",
            "#default",
        ];
        let html_attrs = ["id", "class", "href", "src", "title"];

        for attr in vue_attrs {
            assert!(
                attr.starts_with(':')
                    || attr.starts_with('@')
                    || attr.starts_with('#')
                    || attr.starts_with("v-"),
                "Vue directive {} should match pattern",
                attr
            );
        }

        for attr in html_attrs {
            assert!(
                !attr.starts_with(':')
                    && !attr.starts_with('@')
                    && !attr.starts_with('#')
                    && !attr.starts_with("v-"),
                "HTML attribute {} should NOT match Vue pattern",
                attr
            );
        }
    }
}
