//! Script names the template side of a component reads at runtime.
//!
//! The relation is demand-only. The drawer fills it during the template walk
//! that already resolves each expression identifier against the template's
//! own scopes, so a `v-for` or slot alias never reaches it. Component tags,
//! custom directives and template refs join through the same resolution the
//! unused-binding relation uses, and `<style>` `v-bind()` expressions join
//! after the walk. A read that cannot be established marks the relation
//! incomplete: a consumer then cannot rule any name out.

use vize_carton::{CompactString, FxHashSet};

/// Runtime reads the template and `<style>` `v-bind()` make of script names.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TemplateBindingReads {
    /// Free identifiers of template expressions.
    expressions: FxHashSet<CompactString>,
    /// Free identifiers of `<style>` `v-bind()` values.
    style_binds: FxHashSet<CompactString>,
    /// Static `ref` names. A script binding of that name receives the ref.
    refs: FxHashSet<CompactString>,
    /// Component tags and directives, as setup resolution compares them.
    assets: FxHashSet<CompactString>,
    incomplete: bool,
}

impl TemplateBindingReads {
    /// Whether the template side may read the script binding `name`.
    ///
    /// An incomplete relation answers `true` for every name.
    #[must_use]
    pub fn may_read(&self, name: &str) -> bool {
        self.incomplete
            || self.expressions.contains(name)
            || self.style_binds.contains(name)
            || self.refs.contains(name)
            || (!self.assets.is_empty() && self.assets.contains(asset_key(name).as_str()))
    }

    /// Whether a template expression evaluates the free identifier `name`.
    ///
    /// Unlike a tag, a directive or a `ref` name, an expression identifier has
    /// no registration to fall back on: the render function reads the name.
    #[must_use]
    pub fn expression_reads(&self, name: &str) -> bool {
        self.expressions.contains(name)
    }

    /// Whether a `<style>` `v-bind()` value evaluates the free identifier
    /// `name`. Like a template expression, it has no registration to fall
    /// back on.
    #[must_use]
    pub fn style_bind_reads(&self, name: &str) -> bool {
        self.style_binds.contains(name)
    }

    /// Whether every block that can read a script name was walked.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.incomplete
    }

    /// Record the free identifiers of one expression evaluated with `locals`
    /// in template scope. Direct `eval` can read any name.
    ///
    /// Out of line: the expression walk calls this per expression only under
    /// a read demand. Without any binding demand the walk makes main's one
    /// test at that site.
    #[inline(never)]
    pub fn note_expression(&mut self, identifiers: &[CompactString], locals: &[CompactString]) {
        for identifier in identifiers {
            if identifier == "eval" {
                self.incomplete = true;
            }
            if !locals.contains(identifier) && !self.expressions.contains(identifier) {
                self.expressions.insert(identifier.clone());
            }
        }
    }

    /// Record the free identifiers of one `<style>` `v-bind()` value.
    pub fn note_style_bind(&mut self, identifiers: &[CompactString]) {
        for identifier in identifiers {
            if identifier == "eval" {
                self.incomplete = true;
            }
            if !self.style_binds.contains(identifier) {
                self.style_binds.insert(identifier.clone());
            }
        }
    }

    /// Record a static `ref="name"`.
    pub fn note_ref(&mut self, name: &str) {
        self.refs.insert(CompactString::new(name));
    }

    /// Record a component tag or `v-` directive spelling resolved in setup scope.
    pub fn note_asset(&mut self, spelling: &str) {
        self.assets.insert(asset_key(spelling));
    }

    /// Record that some read could not be established.
    pub fn mark_incomplete(&mut self) {
        self.incomplete = true;
    }

    /// The form in which a setup binding and a component tag or directive
    /// spelling compare. Vue resolves them without regard to hyphens and
    /// case: `<foo-bar>` and `<FooBar>` both read `FooBar`.
    pub(crate) fn asset_key(name: &str) -> CompactString {
        asset_key(name)
    }
}

impl super::Croquis {
    /// The demanded template-side reads; `None` when no consumer demanded them.
    ///
    /// Held with the other demand-only diagnostic facts of `setup_context`,
    /// not as a `Croquis` field: the struct stays the size it was, and it
    /// stays out of semantic snapshots and serialized Croquis surfaces.
    #[must_use]
    pub fn template_binding_reads(&self) -> Option<&TemplateBindingReads> {
        self.setup_context.template_binding_reads()
    }

    /// Producer access for the blocks a template walk does not cover.
    #[doc(hidden)]
    pub fn template_binding_reads_mut(&mut self) -> Option<&mut TemplateBindingReads> {
        self.setup_context.template_binding_reads_mut()
    }
}

fn asset_key(name: &str) -> CompactString {
    name.chars()
        .filter(|ch| *ch != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::TemplateBindingReads;
    use vize_carton::CompactString;

    fn names(values: &[&str]) -> Vec<CompactString> {
        values
            .iter()
            .map(|value| CompactString::new(value))
            .collect()
    }

    #[test]
    fn template_locals_shadow_and_assets_ignore_case_and_hyphens() {
        let mut reads = TemplateBindingReads::default();
        reads.note_expression(&names(&["item", "Foo", "format"]), &names(&["item", "Foo"]));
        reads.note_asset("foo-bar");
        reads.note_asset("v-focus");
        reads.note_ref("panel");
        reads.note_style_bind(&names(&["theme"]));
        assert_eq!(
            [
                "format", "FooBar", "fooBar", "vFocus", "panel", "theme", "item", "Foo", "Bar"
            ]
            .map(|name| (
                reads.may_read(name),
                reads.expression_reads(name),
                reads.style_bind_reads(name)
            )),
            [
                (true, true, false),
                (true, false, false),
                (true, false, false),
                (true, false, false),
                (true, false, false),
                (true, false, true),
                (false, false, false),
                (false, false, false),
                (false, false, false),
            ]
        );
        assert!(reads.is_complete());
    }

    #[test]
    fn eval_and_refusal_leave_no_name_ruled_out() {
        let mut reads = TemplateBindingReads::default();
        reads.note_expression(&names(&["eval"]), &[]);
        assert_eq!(
            (
                reads.is_complete(),
                reads.may_read("Foo"),
                reads.expression_reads("Foo")
            ),
            (false, true, false)
        );
        let mut refused = TemplateBindingReads::default();
        refused.mark_incomplete();
        assert_eq!(
            (refused.is_complete(), refused.may_read("Foo")),
            (false, true)
        );
    }
}
