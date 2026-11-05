use super::{Kind, Visitor};
use oxc_ast::ast::{
    Argument, ArrayPattern, AssignmentTarget, BindingPattern, CallExpression, Expression,
    FormalParameters, ObjectPattern,
};
use oxc_span::Span;
use vize_l0::FxHashMap;

impl Visitor<'_> {
    pub(super) fn skip_default<'b>(pattern: &'b BindingPattern<'b>) -> &'b BindingPattern<'b> {
        match pattern {
            BindingPattern::AssignmentPattern(assignment) => Self::skip_default(&assignment.left),
            other => other,
        }
    }

    pub(super) fn bind_define_props(&mut self, pattern: &BindingPattern<'_>) {
        match Self::skip_default(pattern) {
            BindingPattern::BindingIdentifier(id) => {
                self.insert(id.name.as_str(), Kind::Props(true))
            }
            BindingPattern::ObjectPattern(object) => self.bind_define_props_object(object),
            BindingPattern::ArrayPattern(array) => self.report_destructure(array.span),
            BindingPattern::AssignmentPattern(_) => {}
        }
    }

    fn bind_define_props_object(&mut self, object: &ObjectPattern<'_>) {
        for property in &object.properties {
            match Self::skip_default(&property.value) {
                BindingPattern::BindingIdentifier(id) => {
                    self.insert(id.name.as_str(), Kind::Prop(true));
                }
                BindingPattern::ObjectPattern(nested) => self.report_destructure(nested.span),
                BindingPattern::ArrayPattern(nested) => self.report_destructure(nested.span),
                BindingPattern::AssignmentPattern(_) => {}
            }
        }
        if let Some(rest) = &object.rest {
            self.bind_other(&rest.argument);
        }
    }

    pub(super) fn bind_other(&mut self, pattern: &BindingPattern<'_>) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => self.insert(id.name.as_str(), Kind::Other),
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.bind_other(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.bind_other(&rest.argument);
                }
            }
            BindingPattern::ArrayPattern(array) => self.bind_other_array(array),
            BindingPattern::AssignmentPattern(assignment) => self.bind_other(&assignment.left),
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

    pub(super) fn note_setup_param(&mut self, params: &FormalParameters<'_>) {
        let Some(param) = params.items.first() else {
            return;
        };
        match Self::skip_default(&param.pattern) {
            BindingPattern::BindingIdentifier(id) => {
                self.insert(id.name.as_str(), Kind::Props(false))
            }
            BindingPattern::ObjectPattern(object) => self.report_destructure(object.span),
            BindingPattern::ArrayPattern(array) => self.report_destructure(array.span),
            BindingPattern::AssignmentPattern(_) => {}
        }
    }

    pub(super) fn bind_params_other(&mut self, params: &FormalParameters<'_>) {
        for param in &params.items {
            self.bind_other(&param.pattern);
        }
        if let Some(rest) = &params.rest {
            self.bind_other(&rest.rest.argument);
        }
    }

    pub(super) fn in_setup_property(&self) -> bool {
        self.objects.last().is_some_and(|frame| {
            frame.is_vue && frame.depth == self.fn_depth && frame.key.as_deref() == Some("setup")
        })
    }

    fn callee_kind(&self, expr: &Expression<'_>) -> Option<Kind> {
        let Expression::Identifier(id) = Self::peel(expr) else {
            return None;
        };
        match self.lookup(id.name.as_str()).map(|binding| binding.kind) {
            None => match id.name.as_str() {
                "defineProps" => Some(Kind::DefineProps),
                "withDefaults" => Some(Kind::WithDefaults),
                "defineComponent" | "createApp" | "defineNuxtComponent" | "component" => {
                    Some(Kind::VueFactory)
                }
                _ => None,
            },
            Some(kind @ (Kind::DefineProps | Kind::WithDefaults | Kind::VueFactory)) => Some(kind),
            Some(_) => None,
        }
    }

    pub(super) fn is_define_props(&self, expr: &Expression<'_>) -> bool {
        let Expression::CallExpression(call) = Self::peel(expr) else {
            return false;
        };
        match self.callee_kind(&call.callee) {
            Some(Kind::DefineProps) => true,
            Some(Kind::WithDefaults) => call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .is_some_and(|argument| self.is_define_props(argument)),
            _ => false,
        }
    }

    pub(super) fn is_vue_factory_call(&self, call: &CallExpression<'_>) -> bool {
        if self.callee_kind(&call.callee) == Some(Kind::VueFactory) {
            return true;
        }
        let Expression::StaticMemberExpression(member) = Self::peel(&call.callee) else {
            return false;
        };
        let Expression::Identifier(_) = Self::peel(&member.object) else {
            return false;
        };
        match member.property.name.as_str() {
            "component" | "mixin" => true,
            "extend" => {
                matches!(Self::peel(&member.object), Expression::Identifier(id) if id.name == "Vue")
            }
            _ => false,
        }
    }

    pub(super) fn pattern_span(target: &AssignmentTarget<'_>) -> Option<Span> {
        match target {
            AssignmentTarget::ObjectAssignmentTarget(object) => Some(object.span),
            AssignmentTarget::ArrayAssignmentTarget(array) => Some(array.span),
            _ => None,
        }
    }

    pub(super) fn binding_pattern_span(pattern: &BindingPattern<'_>) -> Option<Span> {
        match pattern {
            BindingPattern::ObjectPattern(object) => Some(object.span),
            BindingPattern::ArrayPattern(array) => Some(array.span),
            BindingPattern::AssignmentPattern(assignment) => {
                Self::binding_pattern_span(&assignment.left)
            }
            BindingPattern::BindingIdentifier(_) => None,
        }
    }

    pub(super) fn mark_vue_argument(&mut self, call: &CallExpression<'_>) {
        if !self.is_vue_factory_call(call) {
            return;
        }
        if let Some(Argument::ObjectExpression(_)) = call.arguments.last() {
            self.force_vue = true;
        }
    }

    pub(super) fn record_import(&mut self, import: &oxc_ast::ast::ImportDeclaration<'_>) {
        let vue = matches!(
            import.source.value.as_str(),
            "vue" | "@vue/composition-api" | "#imports"
        );
        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for specifier in specifiers {
            let oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(spec) = specifier else {
                continue;
            };
            let imported = spec.imported.name();
            let kind = if vue {
                match imported.as_str() {
                    "defineProps" => Kind::DefineProps,
                    "withDefaults" => Kind::WithDefaults,
                    "defineComponent" | "createApp" | "defineNuxtComponent" | "component" => {
                        Kind::VueFactory
                    }
                    _ => Kind::Other,
                }
            } else {
                Kind::Other
            };
            self.insert(spec.local.name.as_str(), kind);
        }
    }

    pub(super) fn enter_function(&mut self, params: &FormalParameters<'_>, setup: bool) {
        self.fn_depth += 1;
        self.scope_id += 1;
        self.frames.push(FxHashMap::default());
        if setup {
            self.note_setup_param(params);
        } else {
            self.bind_params_other(params);
        }
    }

    pub(super) fn exit_function(&mut self) {
        self.frames.pop();
        self.fn_depth -= 1;
        self.scope_id -= 1;
    }
}
