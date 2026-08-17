//! Whether a mirrored `.d.ts` has to keep CommonJS spelling in the virtual
//! project.
//!
//! The virtual root carries its own `package.json` with `"type": "module"`
//! (#2679), so every mirrored file is an ES module unless its extension says
//! otherwise. Two TypeScript forms are then rejected outright: `export =`
//! (`TS1203`) and `import x = require(...)` (`TS1202`). A declaration file that
//! uses either is mirrored as `.d.cts` so it keeps parsing.
//!
//! Every other declaration file keeps its `.d.ts` spelling, and that is
//! load-bearing rather than cosmetic. A `.d.cts` resolves its module specifiers
//! under the `require` condition, so `declare module "vue"` inside it binds to
//! `vue/dist/vue.d.ts` while the generated `.vue.ts` modules import
//! `vue/dist/vue.d.mts`. Two module identities means two
//! `ComponentCustomProperties` interfaces: a plugin augmentation authored in a
//! `.d.ts` (vue-i18n's `$t`, for one) never reaches the
//! `ComponentPublicInstance` the template context reads, and the global
//! collapses to `unknown`.

use oxc_allocator::Allocator;
use oxc_ast::ast::{TSExportAssignment, TSExternalModuleReference};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Whether `content` uses a CommonJS-only declaration form.
///
/// A file the parser cannot read keeps the historical `.d.cts` spelling: its
/// forms are unknown, and the mirrored copy must not start failing to parse.
pub(super) fn declaration_requires_commonjs_spelling(content: &str) -> bool {
    if !content.contains("export") && !content.contains("require") {
        return false;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, content, SourceType::d_ts()).parse();
    if parsed.panicked {
        return true;
    }
    let mut finder = CommonJsFormFinder { found: false };
    finder.visit_program(&parsed.program);
    finder.found
}

struct CommonJsFormFinder {
    found: bool,
}

impl<'a> Visit<'a> for CommonJsFormFinder {
    fn visit_ts_export_assignment(&mut self, assignment: &TSExportAssignment<'a>) {
        self.found = true;
        walk::walk_ts_export_assignment(self, assignment);
    }

    /// The `require(...)` half of `import x = require(...)`. The
    /// `import x = A.B` form is an entity-name alias that ES modules accept, so
    /// only the external-module reference is matched.
    fn visit_ts_external_module_reference(&mut self, reference: &TSExternalModuleReference<'a>) {
        self.found = true;
        walk::walk_ts_external_module_reference(self, reference);
    }
}

#[cfg(test)]
mod tests {
    use super::declaration_requires_commonjs_spelling;

    #[test]
    fn plain_module_augmentation_keeps_declaration_spelling() {
        assert!(!declaration_requires_commonjs_spelling(
            "declare module \"vue\" {\n  interface ComponentCustomProperties {\n    $t: (key: string) => string;\n  }\n}\n\nexport {};\n"
        ));
    }

    #[test]
    fn global_script_declaration_keeps_declaration_spelling() {
        assert!(!declaration_requires_commonjs_spelling(
            "declare global {\n  const answer: number;\n}\n"
        ));
    }

    #[test]
    fn top_level_export_assignment_requires_commonjs_spelling() {
        assert!(declaration_requires_commonjs_spelling(
            "declare function f(): void;\nexport = f;\n"
        ));
    }

    #[test]
    fn export_assignment_inside_ambient_module_requires_commonjs_spelling() {
        assert!(declaration_requires_commonjs_spelling(
            "declare module \"buffer-from\" {\n  function bufferFrom(): void;\n  export = bufferFrom;\n}\n"
        ));
    }

    #[test]
    fn import_require_requires_commonjs_spelling() {
        assert!(declaration_requires_commonjs_spelling(
            "import fs = require(\"fs\");\nexport declare const f: typeof fs;\n"
        ));
    }

    #[test]
    fn import_entity_name_alias_keeps_declaration_spelling() {
        assert!(!declaration_requires_commonjs_spelling(
            "declare namespace A {\n  const b: number;\n}\nimport C = A.b;\nexport { C };\n"
        ));
    }

    #[test]
    fn export_assignment_in_a_comment_keeps_declaration_spelling() {
        assert!(!declaration_requires_commonjs_spelling(
            "// legacy shape used `export = value`\nexport declare const value: number;\n"
        ));
    }
}
