//! The source of compiler-generated Vue imports, separate from authored imports.
use vize_carton::String;
use vize_l0::js_string::push_js_string_literal;

pub(crate) struct RuntimeModule<'a> {
    pub(crate) name: &'a str,
    literal: String,
}

impl<'a> RuntimeModule<'a> {
    pub(crate) fn new(name: &'a str) -> Self {
        // Keep default output byte-identical, including its quote spelling.
        let literal = if name == "vue" {
            String::from("'vue'")
        } else {
            let mut literal = String::default();
            push_js_string_literal(&mut literal, name);
            literal
        };
        Self { name, literal }
    }

    pub(crate) fn push_import(&self, output: &mut vize_carton::Vec<u8>, specifiers: &str) {
        output.extend_from_slice(b"import { ");
        output.extend_from_slice(specifiers.as_bytes());
        output.extend_from_slice(b" } from ");
        output.extend_from_slice(self.literal.as_bytes());
        output.push(b'\n');
    }

    pub(crate) fn append_import(&self, output: &mut String, specifiers: &str) {
        output.push_str("import { ");
        output.push_str(specifiers);
        output.push_str(" } from ");
        output.push_str(&self.literal);
        output.push('\n');
    }
}
