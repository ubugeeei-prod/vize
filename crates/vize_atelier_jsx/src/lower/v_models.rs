//! `v-models` — the `@vue/babel-plugin-jsx` built-in that binds several models
//! on one component at once.
//!
//! `v-models={[[value], [value, "arg"], [value, ["mod"]], [value, "arg", ["mod"]]]}`
//! is exactly a list of the `v-model={[…]}` array form, so every entry is
//! lowered through the same [`Lowerer::lower_model_array`] `v-model` uses and the
//! attribute expands into one `model` directive per entry.
//!
//! # Why this is a built-in and not a custom directive
//!
//! `v-models` is a plugin built-in, not a user directive. Before #3418 any
//! unrecognized `v-*` attribute fell through to the generic custom-directive
//! path, so `v-models` compiled to `resolveDirective("models")` — a lookup for a
//! directive that does not exist. Vue resolves it to nothing at runtime, so the
//! component rendered with **no model bindings at all**, with no error and no
//! warning. Every shape below therefore either lowers or is diagnosed; none may
//! reach the custom-directive path again.
//!
//! # Shapes that are diagnosed rather than lowered
//!
//! - `v-models:arg={…}` — babel reads the JSX namespace as a default prop name
//!   but then ignores each entry's own arg: `v-models:x={[[a], [b, "b"]]}` binds
//!   `x` and `modelValue`, never `b`. The spelling is undocumented and its babel
//!   behavior is inconsistent, so Vize rejects it and points at the entry form.
//! - `v-models_lazy={…}` — the `_`-suffixed modifier spelling. Same reasoning:
//!   undocumented, and the per-entry modifier array expresses it exactly.
//! - a missing value, a non-array value, an empty array, a non-array entry, or
//!   an entry that cannot be destructured. Babel rejects all of these too
//!   ("You should pass a Two-dimensional Arrays to v-models").
//! - an entry whose target cannot be assigned to, for the same reason `v-model`
//!   rejects one (#3420): the write-back compiles to `target = $event`.
//! - an entry with a non-string argument. Dynamic argument support needs
//!   computed prop and update-listener codegen, so it is diagnosed (#3466).
//! - `v-models` on a plain element. Babel: "v-models can only use in custom
//!   components".
//!
//! # Known divergence, deliberately not diagnosed
//!
//! Two entries that resolve to the same prop name (`v-models={[[a], [b]]}`, both
//! `modelValue`) emit a duplicate key. Babel keeps the first value and merges the
//! update handlers into an array; Vize emits both pairs, so the last one wins.
//! The input is a user error either way and both outputs parse, so this is
//! recorded here rather than turned into a diagnostic.

use oxc_ast::ast::{ArrayExpression, Expression, JSXAttribute, JSXAttributeName};
use oxc_span::{GetSpan, Span};
use vize_relief::PropNode;
use vize_s0::Vec;

use super::Lowerer;
use super::v_model::{ModelArrayLowering, is_assignable_target};

/// How a `v-models` attribute was spelled.
enum Form {
    /// `v-models={…}` — the only supported spelling.
    Plain,
    /// `v-models:arg={…}` — a JSX namespace argument.
    Argument,
    /// `v-models_lazy={…}` — `_`-suffixed modifiers.
    UnderscoreModifiers,
}

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Lower a `v-models` attribute into one `model` directive per entry,
    /// appending them to `props`.
    ///
    /// `on_component` is whether the owning tag renders as a component; `false`
    /// rejects the attribute the way babel does.
    ///
    /// Returns `true` when `attr` was a `v-models` attribute and has been fully
    /// consumed here — **including when it was rejected with a diagnostic**,
    /// because a rejected built-in must not fall back to the custom-directive
    /// path that caused #3418.
    pub(crate) fn try_lower_v_models(
        &mut self,
        attr: &JSXAttribute<'_>,
        on_component: bool,
        props: &mut Vec<'a, PropNode<'a>>,
    ) -> bool {
        let Some(form) = self.v_models_form(attr) else {
            return false;
        };
        let span = attr.span;

        match form {
            Form::Argument => {
                self.reject(
                    span,
                    "v-models does not take an argument; name the prop inside each entry \
                     instead, e.g. v-models={[[value, \"name\"]]}.",
                );
                return true;
            }
            Form::UnderscoreModifiers => {
                self.reject(
                    span,
                    "v-models does not take `_`-suffixed modifiers; list them inside each \
                     entry instead, e.g. v-models={[[value, [\"lazy\"]]]}.",
                );
                return true;
            }
            Form::Plain => {}
        }

        if !on_component {
            self.reject(
                span,
                "v-models can only be used on a component; a plain element binds one \
                 model with v-model.",
            );
            return true;
        }

        let Some(entries) = self.array_literal_value(attr.value.as_ref()) else {
            self.reject(
                span,
                "v-models expects a two-dimensional array of `[value, arg?, modifiers?]` \
                 entries, e.g. v-models={[[foo], [bar, \"bar\"]]}.",
            );
            return true;
        };
        if entries.elements.is_empty() {
            self.reject(
                span,
                "v-models was given an empty array; it needs at least one \
                 `[value, arg?, modifiers?]` entry.",
            );
            return true;
        }

        self.lower_v_models_entries(entries, span, props);
        true
    }

    /// Lower every entry of a validated `v-models` array.
    ///
    /// All entries are lowered before anything is appended: a rejected entry
    /// drops the whole attribute, because emitting the entries that happened to
    /// parse would pair a build error with a partial set of model bindings.
    fn lower_v_models_entries(
        &mut self,
        entries: &ArrayExpression<'_>,
        attr_span: Span,
        props: &mut Vec<'a, PropNode<'a>>,
    ) {
        let loc = self.mapper().location(attr_span);
        let mut lowered = std::vec::Vec::with_capacity(entries.elements.len());
        let mut rejected = false;

        for element in &entries.elements {
            let element_span = element.span();
            let Some(Expression::ArrayExpression(entry)) = element.as_expression() else {
                let source = self.mapper().slice(element_span);
                self.reject_at(
                    element_span,
                    format_args!(
                        "v-models entry `{source}` is not an array; each entry must be \
                         `[value, arg?, modifiers?]`."
                    ),
                );
                rejected = true;
                continue;
            };

            match entry
                .elements
                .first()
                .and_then(|first| first.as_expression())
            {
                Some(target) if is_assignable_target(target) => {}
                Some(target) => {
                    let target_span = target.span();
                    let source = self.mapper().slice(target_span);
                    self.reject_at(
                        target_span,
                        format_args!(
                            "v-models target `{source}` cannot be assigned to; v-model needs \
                             a variable or property reference."
                        ),
                    );
                    rejected = true;
                    continue;
                }
                None => {
                    let source = self.mapper().slice(element_span);
                    self.reject_at(
                        element_span,
                        format_args!(
                            "v-models entry `{source}` has no bound value; each entry must be \
                             `[value, arg?, modifiers?]`."
                        ),
                    );
                    rejected = true;
                    continue;
                }
            }

            match self.lower_model_array(entry, &loc, self.uses_babel_vdom_compat()) {
                ModelArrayLowering::Lowered(prop) => lowered.push(prop),
                ModelArrayLowering::Rejected => rejected = true,
                ModelArrayLowering::Unrecognized => {
                    let source = self.mapper().slice(element_span);
                    self.reject_at(
                        element_span,
                        format_args!(
                            "v-models entry `{source}` cannot be destructured into \
                             `[value, arg?, modifiers?]`."
                        ),
                    );
                    rejected = true;
                }
            }
        }

        if !rejected {
            props.extend(lowered);
        }
    }

    /// Classify an attribute as one of the `v-models` spellings, or `None` when
    /// it is not a `v-models` attribute at all.
    fn v_models_form(&self, attr: &JSXAttribute<'_>) -> Option<Form> {
        match &attr.name {
            JSXAttributeName::NamespacedName(named) => {
                let raw = self
                    .mapper()
                    .slice(named.namespace.span())
                    .strip_prefix("v-")?;
                // A namespace is an argument, so even bare `v-models:x` is the
                // argument form rather than the plain one.
                match classify(raw)? {
                    Form::Plain => Some(Form::Argument),
                    other => Some(other),
                }
            }
            JSXAttributeName::Identifier(id) => {
                classify(self.mapper().slice(id.span()).strip_prefix("v-")?)
            }
        }
    }
}

/// Classify a `v-`-stripped attribute name as a `v-models` spelling.
fn classify(name: &str) -> Option<Form> {
    if name == "models" {
        return Some(Form::Plain);
    }
    // `v-models_lazy` — JSX attribute names cannot contain `.`, so
    // `@vue/babel-plugin-jsx` also accepts `_`-joined modifier suffixes.
    if name
        .strip_prefix("models_")
        .is_some_and(|rest| !rest.is_empty())
    {
        return Some(Form::UnderscoreModifiers);
    }
    None
}
