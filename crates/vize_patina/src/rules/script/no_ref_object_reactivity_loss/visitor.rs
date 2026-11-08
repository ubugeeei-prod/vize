use super::{MESSAGE, REF_FACTORIES, RULE};
use crate::diagnostic::LintDiagnostic;
use crate::rules::script::ScriptLintResult;
use oxc_ast::ast::{
    ArrayPattern, BindingPattern, Expression, FormalParameters, ImportDeclaration, ObjectPattern,
    Program,
};
use oxc_span::Span;
use vize_l0::{CompactString, FxHashMap};

mod visit;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Factory,
    ToRefs,
    DefineModel,
    Namespace,
    Ref(u32),
    Bag(u32),
    Other,
}

pub(super) struct Visitor<'result, 'ast> {
    pub(super) offset: usize,
    pub(super) result: &'result mut ScriptLintResult,
    pub(super) fn_depth: u32,
    pub(super) writing: bool,
    pub(super) suppress_reads: bool,
    pub(super) frames: Vec<FxHashMap<CompactString, Kind>>,
    pub(super) sibling_program: Option<&'ast Program<'ast>>,
}

impl Visitor<'_, '_> {
    pub(super) fn insert(&mut self, name: &str, kind: Kind) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(CompactString::from(name), kind);
        }
    }

    pub(super) fn assign(&mut self, name: &str, kind: Kind) {
        if let Some(frame) = self
            .frames
            .iter_mut()
            .rev()
            .find(|frame| frame.contains_key(name))
        {
            frame.insert(CompactString::from(name), kind);
        }
    }

    pub(super) fn assigned_kind(&self, expression: &Expression<'_>) -> Option<Kind> {
        self.bag_depth(expression)
            .map(Kind::Bag)
            .or_else(|| self.ref_depth(expression).map(Kind::Ref))
    }

    fn lookup(&self, name: &str) -> Option<Kind> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.get(name).copied())
    }

    fn peel<'b>(expr: &'b Expression<'b>) -> &'b Expression<'b> {
        match expr {
            Expression::ParenthesizedExpression(expr) => Self::peel(&expr.expression),
            Expression::TSAsExpression(expr) => Self::peel(&expr.expression),
            Expression::TSSatisfiesExpression(expr) => Self::peel(&expr.expression),
            Expression::TSNonNullExpression(expr) => Self::peel(&expr.expression),
            other => other,
        }
    }

    fn ref_depth(&self, expr: &Expression<'_>) -> Option<u32> {
        match Self::peel(expr) {
            Expression::Identifier(id) => match self.lookup(id.name.as_str()) {
                Some(Kind::Ref(depth)) => Some(depth),
                _ => None,
            },
            Expression::StaticMemberExpression(member) => {
                self.bag_member(&member.object, member.property.name.as_str())
            }
            Expression::CallExpression(call)
                if self.is_ref_factory(&call.callee) || self.is_define_model(&call.callee) =>
            {
                Some(self.fn_depth)
            }
            _ => None,
        }
    }

    /// A ref factory only defines a tracked binding when the call is the
    /// declarator's direct initializer. Type assertions sit between the call
    /// and the declarator in ESTree, so eslint-plugin-vue deliberately does
    /// not propagate ref identity through them.
    fn binding_ref_depth(&self, expr: &Expression<'_>) -> Option<u32> {
        match expr {
            Expression::ParenthesizedExpression(parenthesized) => {
                self.binding_ref_depth(&parenthesized.expression)
            }
            Expression::Identifier(_) | Expression::StaticMemberExpression(_) => {
                self.ref_depth(expr)
            }
            Expression::CallExpression(call)
                if self.is_ref_factory(&call.callee) || self.is_define_model(&call.callee) =>
            {
                Some(self.fn_depth)
            }
            _ => None,
        }
    }

    fn bag_member(&self, object: &Expression<'_>, property: &str) -> Option<u32> {
        if property == "value" {
            return None;
        }
        let Expression::Identifier(id) = Self::peel(object) else {
            return None;
        };
        match self.lookup(id.name.as_str()) {
            Some(Kind::Bag(depth)) => Some(depth),
            _ => None,
        }
    }

    fn bag_depth(&self, expr: &Expression<'_>) -> Option<u32> {
        match Self::peel(expr) {
            Expression::Identifier(id) => match self.lookup(id.name.as_str()) {
                Some(Kind::Bag(depth)) => Some(depth),
                _ => None,
            },
            Expression::CallExpression(call)
                if self.is_named_call(&call.callee, Kind::ToRefs, "toRefs") =>
            {
                Some(self.fn_depth)
            }
            _ => None,
        }
    }

    fn is_named_call(&self, expr: &Expression<'_>, kind: Kind, property: &str) -> bool {
        match Self::peel(expr) {
            Expression::Identifier(id) => self.lookup(id.name.as_str()) == Some(kind),
            Expression::StaticMemberExpression(member) => {
                matches!(self.namespace_of(&member.object), Some(Kind::Namespace))
                    && member.property.name == property
            }
            _ => false,
        }
    }

    fn namespace_of(&self, expr: &Expression<'_>) -> Option<Kind> {
        let Expression::Identifier(id) = Self::peel(expr) else {
            return None;
        };
        self.lookup(id.name.as_str())
    }

    fn is_ref_factory(&self, expr: &Expression<'_>) -> bool {
        match Self::peel(expr) {
            Expression::Identifier(id) => self.lookup(id.name.as_str()) == Some(Kind::Factory),
            Expression::StaticMemberExpression(member) => {
                matches!(self.namespace_of(&member.object), Some(Kind::Namespace))
                    && REF_FACTORIES.contains(&member.property.name.as_str())
            }
            _ => false,
        }
    }

    fn is_define_model(&self, expr: &Expression<'_>) -> bool {
        match Self::peel(expr) {
            Expression::Identifier(id) => match self.lookup(id.name.as_str()) {
                None => id.name == "defineModel",
                Some(Kind::DefineModel) => true,
                _ => false,
            },
            Expression::StaticMemberExpression(member) => {
                matches!(self.namespace_of(&member.object), Some(Kind::Namespace))
                    && member.property.name == "defineModel"
            }
            _ => false,
        }
    }

    pub(super) fn same_scope_ref(&self, expr: &Expression<'_>) -> bool {
        self.ref_depth(expr) == Some(self.fn_depth)
    }

    pub(super) fn report(&mut self, span: Span) {
        self.result.add_diagnostic(LintDiagnostic::warn(
            RULE,
            MESSAGE,
            self.offset as u32 + span.start,
            self.offset as u32 + span.end,
        ));
    }

    fn bind_other(&mut self, pattern: &BindingPattern<'_>) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => self.insert(id.name.as_str(), Kind::Other),
            BindingPattern::ObjectPattern(object) => self.bind_other_object(object),
            BindingPattern::ArrayPattern(array) => self.bind_other_array(array),
            BindingPattern::AssignmentPattern(assignment) => self.bind_other(&assignment.left),
        }
    }

    fn bind_other_object(&mut self, object: &ObjectPattern<'_>) {
        for property in &object.properties {
            self.bind_other(&property.value);
        }
        if let Some(rest) = &object.rest {
            self.bind_other(&rest.argument);
        }
    }

    fn bind_other_array(&mut self, array: &ArrayPattern<'_>) {
        for element in array.elements.iter().flatten() {
            self.bind_other(element);
        }
        if let Some(rest) = &array.rest {
            self.bind_other(&rest.argument);
        }
    }

    pub(super) fn bind_params(&mut self, params: &FormalParameters<'_>) {
        for param in &params.items {
            self.bind_other(&param.pattern);
        }
        if let Some(rest) = &params.rest {
            self.bind_other(&rest.rest.argument);
        }
    }

    fn binding_name<'b>(pattern: &'b BindingPattern<'b>) -> Option<&'b str> {
        match pattern {
            BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
            BindingPattern::AssignmentPattern(assignment) => Self::binding_name(&assignment.left),
            _ => None,
        }
    }

    fn bind_ref_properties(&mut self, object: &ObjectPattern<'_>, depth: u32) {
        for property in &object.properties {
            if let Some(name) = Self::binding_name(&property.value) {
                self.insert(name, Kind::Ref(depth));
            } else {
                self.bind_other(&property.value);
            }
        }
        if let Some(rest) = &object.rest {
            self.bind_other(&rest.argument);
        }
    }

    fn bind_define_model_array(&mut self, array: &ArrayPattern<'_>) {
        for (index, element) in array.elements.iter().enumerate() {
            let Some(element) = element else {
                continue;
            };
            if index == 0
                && let Some(name) = Self::binding_name(element)
            {
                self.insert(name, Kind::Ref(self.fn_depth));
            } else {
                self.bind_other(element);
            }
        }
        if let Some(rest) = &array.rest {
            self.bind_other(&rest.argument);
        }
    }

    pub(super) fn record(&mut self, pattern: &BindingPattern<'_>, init: Option<&Expression<'_>>) {
        if let BindingPattern::ObjectPattern(object) = pattern {
            if let Some(depth) = init.and_then(|expr| self.bag_depth(expr)) {
                self.bind_ref_properties(object, depth);
                return;
            }
            let copies_value = object
                .properties
                .iter()
                .any(|property| property.key.static_name().as_deref() == Some("value"));
            if copies_value && init.is_some_and(|expr| self.same_scope_ref(expr)) {
                self.report(object.span);
            }
            self.bind_other(pattern);
            return;
        }
        if let BindingPattern::ArrayPattern(array) = pattern
            && init.is_some_and(|expr| {
                matches!(Self::peel(expr), Expression::CallExpression(call) if self.is_define_model(&call.callee))
            })
        {
            self.bind_define_model_array(array);
            return;
        }
        if let BindingPattern::BindingIdentifier(id) = pattern {
            let kind = match init {
                Some(expr) => self
                    .bag_depth(expr)
                    .map(Kind::Bag)
                    .or_else(|| self.binding_ref_depth(expr).map(Kind::Ref))
                    .unwrap_or(Kind::Other),
                None => Kind::Other,
            };
            self.insert(id.name.as_str(), kind);
            return;
        }
        self.bind_other(pattern);
    }

    pub(super) fn record_import(&mut self, import: &ImportDeclaration<'_>) {
        let vue = matches!(
            import.source.value.as_str(),
            "vue" | "@vue/composition-api" | "#imports"
        );
        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for specifier in specifiers {
            match specifier {
                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                    let imported = spec.imported.name().as_str();
                    let kind = if vue && REF_FACTORIES.contains(&imported) {
                        Kind::Factory
                    } else if vue && imported == "toRefs" {
                        Kind::ToRefs
                    } else if vue && imported == "defineModel" {
                        Kind::DefineModel
                    } else {
                        Kind::Other
                    };
                    self.insert(spec.local.name.as_str(), kind);
                }
                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                    self.insert(
                        spec.local.name.as_str(),
                        if vue { Kind::Namespace } else { Kind::Other },
                    );
                }
                oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                    self.insert(spec.local.name.as_str(), Kind::Other);
                }
            }
        }
    }
}
