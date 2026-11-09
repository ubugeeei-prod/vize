//! Compatibility rules whose evidence belongs to the SFC or script surface.
//!
//! These rules intentionally consume the descriptor and script parses already
//! owned by the lint engine.  Keeping them here avoids reparsing every block
//! once per compatibility rule and gives every diagnostic an authored SFC
//! range.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, Statement, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::{FxHashSet, String, cstr};

macro_rules! meta {
    ($name:literal, $description:literal) => {
        RuleMeta {
            name: $name,
            description: $description,
            category: RuleCategory::Recommended,
            fixable: false,
            default_severity: Severity::Warning,
        }
    };
}

static ENFORCE_STYLE_ATTRIBUTE_META: RuleMeta = meta!(
    "vue/enforce-style-attribute",
    "Enforce allowed style-block attributes"
);
static NEXT_TICK_STYLE_META: RuleMeta = meta!(
    "vue/next-tick-style",
    "Enforce Promise-style nextTick usage"
);
static NO_CONTENT_OUTSIDE_BLOCK_META: RuleMeta = meta!(
    "vue/no-content-outside-block",
    "Disallow content outside SFC top-level blocks"
);
static NO_DEFINE_MODEL_DEFAULT_META: RuleMeta = meta!(
    "vue/no-define-model-default",
    "Disallow default values in defineModel"
);
static NO_PROPS_SHADOW_META: RuleMeta = meta!(
    "vue/no-props-shadow",
    "Disallow setup declarations that shadow props"
);
static NO_SHADOW_NATIVE_EVENTS_META: RuleMeta = meta!(
    "vue/no-shadow-native-events",
    "Disallow component events that shadow native events"
);
static REQUIRE_MACRO_VARIABLE_NAME_META: RuleMeta = meta!(
    "vue/require-macro-variable-name",
    "Require conventional macro variable names"
);
static SORT_KEYS_META: RuleMeta = meta!(
    "vue/sort-keys",
    "Require sorted keys in component option objects"
);

pub(crate) fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(EnforceStyleAttribute::default()),
        Box::new(NextTickStyle),
        Box::new(NoContentOutsideBlock),
        Box::new(NoDefineModelDefault),
        Box::new(NoPropsShadow),
        Box::new(NoShadowNativeEvents),
        Box::new(RequireMacroVariableName),
        Box::new(SortKeys::default()),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Authored style-block modes accepted by `vue/enforce-style-attribute`.
pub enum StyleAttribute {
    /// A plain `<style>` block without `scoped` or `module`.
    Plain,
    /// A `<style scoped>` block.
    Scoped,
    /// A `<style module>` block.
    Module,
}

/// `vue/enforce-style-attribute`; upstream defaults to `scoped` only.
pub struct EnforceStyleAttribute {
    allow_plain: bool,
    allow_scoped: bool,
    allow_module: bool,
}

impl Default for EnforceStyleAttribute {
    fn default() -> Self {
        Self {
            allow_plain: false,
            allow_scoped: true,
            allow_module: false,
        }
    }
}

impl EnforceStyleAttribute {
    /// Build the rule from the style modes accepted by the project.
    pub fn new(allowed: &[StyleAttribute]) -> Self {
        Self {
            allow_plain: allowed.contains(&StyleAttribute::Plain),
            allow_scoped: allowed.contains(&StyleAttribute::Scoped),
            allow_module: allowed.contains(&StyleAttribute::Module),
        }
    }
}

impl Rule for EnforceStyleAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &ENFORCE_STYLE_ATTRIBUTE_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let finding = ctx.sfc_descriptor().and_then(|descriptor| {
            descriptor.styles.iter().find_map(|style| {
                let allowed = if style.scoped {
                    self.allow_scoped
                } else if style.module.is_some() {
                    self.allow_module
                } else {
                    self.allow_plain
                };
                (!allowed).then_some((style.loc.tag_start as u32, style.loc.start as u32))
            })
        });
        if let Some((start, end)) = finding {
            ctx.report_in_sfc(
                LintDiagnostic::warn(
                    ENFORCE_STYLE_ATTRIBUTE_META.name,
                    "This style block uses a disallowed style attribute mode",
                    start,
                    end,
                )
                .with_help("Use one of the style modes allowed by this rule's configuration"),
            );
        }
    }
}

/// `vue/no-content-outside-block`.
pub struct NoContentOutsideBlock;

impl Rule for NoContentOutsideBlock {
    fn meta(&self) -> &'static RuleMeta {
        &NO_CONTENT_OUTSIDE_BLOCK_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let ranges = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let mut ranges = Vec::with_capacity(
                usize::from(descriptor.template.is_some())
                    + usize::from(descriptor.script.is_some())
                    + usize::from(descriptor.script_setup.is_some())
                    + descriptor.styles.len()
                    + descriptor.custom_blocks.len(),
            );
            if let Some(block) = &descriptor.template {
                ranges.push((block.loc.tag_start, block.loc.tag_end));
            }
            if let Some(block) = &descriptor.script {
                ranges.push((block.loc.tag_start, block.loc.tag_end));
            }
            if let Some(block) = &descriptor.script_setup {
                ranges.push((block.loc.tag_start, block.loc.tag_end));
            }
            ranges.extend(
                descriptor
                    .styles
                    .iter()
                    .map(|block| (block.loc.tag_start, block.loc.tag_end)),
            );
            ranges.extend(
                descriptor
                    .custom_blocks
                    .iter()
                    .map(|block| (block.loc.tag_start, block.loc.tag_end)),
            );
            ranges.sort_unstable_by_key(|range| range.0);
            ranges
        };

        let mut cursor = 0;
        for (start, end) in ranges {
            if let Some(range) = visible_text_range(ctx.source, cursor, start) {
                report_outside_content(ctx, range);
            }
            cursor = cursor.max(end);
        }
        if let Some(range) = visible_text_range(ctx.source, cursor, ctx.source.len()) {
            report_outside_content(ctx, range);
        }
    }
}

fn visible_text_range(source: &str, start: usize, end: usize) -> Option<ByteRange> {
    let slice = source.get(start..end)?;
    let bytes = slice.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if slice
            .get(cursor..)
            .is_some_and(|tail| tail.starts_with("<!--"))
        {
            cursor += slice
                .get(cursor + 4..)
                .and_then(|tail| tail.find("-->"))
                .map_or(bytes.len() - cursor, |length| length + 7);
            continue;
        }
        let character = slice.get(cursor..)?.chars().next()?;
        if !character.is_whitespace() {
            let content_start = cursor;
            let content_end = slice
                .get(content_start..)?
                .find("<!--")
                .map_or(bytes.len(), |length| content_start + length);
            let trimmed = slice.get(content_start..content_end)?.trim_end();
            return Some(ByteRange {
                start: (start + content_start) as u32,
                end: (start + content_start + trimmed.len()) as u32,
            });
        }
        cursor += character.len_utf8();
    }
    None
}

fn report_outside_content(ctx: &mut LintContext<'_>, range: ByteRange) {
    ctx.warn_at_with_help(
        "Unexpected content outside of the top-level blocks",
        range,
        "Move the content into a top-level block or remove it; Vue drops it during compilation",
    );
}

/// `vue/no-define-model-default`.
pub struct NoDefineModelDefault;

impl Rule for NoDefineModelDefault {
    fn meta(&self) -> &'static RuleMeta {
        &NO_DEFINE_MODEL_DEFAULT_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = setup_program(ctx, |program, offset, _| {
            let mut visitor = DefineModelDefaults {
                offset,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            visitor.findings
        });
        for finding in findings {
            ctx.warn_at_with_help(
                "Unexpected default value in `defineModel`",
                finding,
                "Keep the parent and child model values synchronized by applying the default in the parent",
            );
        }
    }
}

struct DefineModelDefaults {
    offset: u32,
    findings: Vec<ByteRange>,
}

impl<'a> Visit<'a> for DefineModelDefaults {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if callee_name(call) == Some("defineModel")
            && let Some(object) = call.arguments.iter().rev().find_map(object_argument)
            && let Some(span) = property_span(object, "default")
        {
            self.findings.push(absolute_range(self.offset, span));
        }
        walk::walk_call_expression(self, call);
    }
}

/// `vue/require-macro-variable-name` with the upstream defaults.
pub struct RequireMacroVariableName;

impl Rule for RequireMacroVariableName {
    fn meta(&self) -> &'static RuleMeta {
        &REQUIRE_MACRO_VARIABLE_NAME_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = setup_program(ctx, |program, offset, _| {
            let mut visitor = MacroVariableNames {
                offset,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            visitor.findings
        });
        for (macro_name, expected, range) in findings {
            ctx.warn_at_with_help(
                cstr!(
                    "The variable name of `{}` must be `{}`",
                    macro_name,
                    expected
                ),
                range,
                cstr!("Rename this binding to `{}`", expected),
            );
        }
    }
}

struct MacroVariableNames {
    offset: u32,
    findings: Vec<(&'static str, &'static str, ByteRange)>,
}

impl<'a> Visit<'a> for MacroVariableNames {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
            walk::walk_variable_declarator(self, declarator);
            return;
        };
        let Some(initializer) = declarator.init.as_ref() else {
            return;
        };
        let Some((macro_name, expected)) = macro_binding(initializer) else {
            walk::walk_variable_declarator(self, declarator);
            return;
        };
        if binding.name.as_str() != expected {
            self.findings.push((
                macro_name,
                expected,
                absolute_range(self.offset, binding.span),
            ));
        }
        walk::walk_variable_declarator(self, declarator);
    }
}

fn macro_binding(expression: &Expression<'_>) -> Option<(&'static str, &'static str)> {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    let name = callee_name(call)?;
    let name = if name == "withDefaults" {
        call.arguments
            .first()
            .and_then(Argument::as_expression)
            .and_then(|argument| match argument.get_inner_expression() {
                Expression::CallExpression(inner) => callee_name(inner),
                _ => None,
            })?
    } else {
        name
    };
    match name {
        "defineProps" => Some(("defineProps", "props")),
        "defineEmits" => Some(("defineEmits", "emit")),
        "defineSlots" => Some(("defineSlots", "slots")),
        "useSlots" => Some(("useSlots", "slots")),
        "useAttrs" => Some(("useAttrs", "attrs")),
        _ => None,
    }
}

/// `vue/next-tick-style`; the upstream default is Promise style.
pub struct NextTickStyle;

impl Rule for NextTickStyle {
    fn meta(&self) -> &'static RuleMeta {
        &NEXT_TICK_STYLE_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut findings = Vec::new();
        collect_script_programs(ctx, |program, offset, source| {
            let mut visitor = NextTickCalls {
                offset,
                source,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            findings.extend(visitor.findings);
        });
        for finding in findings {
            ctx.warn_at_with_help(
                "Use the Promise returned by `nextTick`",
                finding,
                "Await `nextTick()` or chain `.then(...)` instead of passing a callback",
            );
        }
    }
}

struct NextTickCalls<'s> {
    offset: u32,
    source: &'s str,
    findings: Vec<ByteRange>,
}

impl<'a> Visit<'a> for NextTickCalls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_next_tick_call(call)
            && (!call.arguments.is_empty() || !promise_is_consumed(self.source, call.span))
        {
            self.findings
                .push(absolute_range(self.offset, call.callee.span()));
        }
        walk::walk_call_expression(self, call);
    }
}

fn is_next_tick_call(call: &CallExpression<'_>) -> bool {
    match call.callee.get_inner_expression() {
        Expression::Identifier(identifier) => identifier.name == "nextTick",
        Expression::StaticMemberExpression(member) => {
            member.property.name == "$nextTick"
                || (member.property.name == "nextTick"
                    && matches!(
                        member.object.get_inner_expression(),
                        Expression::Identifier(identifier) if identifier.name == "Vue"
                    ))
        }
        _ => false,
    }
}

fn promise_is_consumed(source: &str, span: Span) -> bool {
    source
        .get(..span.start as usize)
        .is_some_and(|prefix| prefix.trim_end().ends_with("await"))
        || source
            .get(span.end as usize..)
            .is_some_and(|suffix| suffix.trim_start().starts_with(".then"))
}

/// `vue/no-props-shadow`.
pub struct NoPropsShadow;

impl Rule for NoPropsShadow {
    fn meta(&self) -> &'static RuleMeta {
        &NO_PROPS_SHADOW_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut semantic_props = FxHashSet::default();
        if let Some(analysis) = ctx.analysis() {
            semantic_props.extend(
                analysis
                    .macros
                    .props()
                    .iter()
                    .map(|prop| String::from(prop.name.as_str())),
            );
        }
        let findings = setup_program(ctx, |program, offset, source| {
            let mut props = semantic_props;
            collect_runtime_props(program, &mut props);
            collect_prop_shadows(program, source, offset, &props)
        });
        for (name, range) in findings {
            ctx.warn_at_with_help(
                cstr!(
                    "Declaration of `{}` shadows the prop with the same name",
                    name
                ),
                range,
                "Rename the local declaration or access the prop through a distinct binding",
            );
        }
    }
}

/// `vue/no-shadow-native-events`.
pub struct NoShadowNativeEvents;

impl Rule for NoShadowNativeEvents {
    fn meta(&self) -> &'static RuleMeta {
        &NO_SHADOW_NATIVE_EVENTS_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut findings = Vec::new();
        collect_script_programs(ctx, |program, offset, _| {
            let mut visitor = NativeEvents {
                offset,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            findings.extend(visitor.findings);
        });
        for (name, range) in findings {
            ctx.warn_at_with_help(
                cstr!("Avoid shadowing the native event `{}`", name),
                range,
                "Choose a component-specific event name so native listeners remain unambiguous",
            );
        }
    }
}

struct NativeEvents {
    offset: u32,
    findings: Vec<(String, ByteRange)>,
}

impl<'a> Visit<'a> for NativeEvents {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        let name = callee_name(call);
        if matches!(name, Some("emit" | "$emit"))
            && let Some(Argument::StringLiteral(event)) = call.arguments.first()
        {
            self.collect(event.value.as_str(), event.span);
        }
        if name == Some("defineEmits")
            && let Some(argument) = call.arguments.first().and_then(Argument::as_expression)
        {
            match argument.get_inner_expression() {
                Expression::ArrayExpression(array) => {
                    for element in &array.elements {
                        if let Some(Expression::StringLiteral(event)) = element.as_expression() {
                            self.collect(event.value.as_str(), event.span);
                        }
                    }
                }
                Expression::ObjectExpression(object) => {
                    for property in &object.properties {
                        let ObjectPropertyKind::ObjectProperty(property) = property else {
                            continue;
                        };
                        if let Some(name) = property_name(&property.key) {
                            self.collect(name, property.key.span());
                        }
                    }
                }
                _ => {}
            }
        }
        walk::walk_call_expression(self, call);
    }
}

impl NativeEvents {
    fn collect(&mut self, name: &str, span: Span) {
        if DOM_EVENTS.contains(&name.to_ascii_lowercase().as_str()) {
            self.findings
                .push((name.into(), absolute_range(self.offset, span)));
        }
    }
}

const DOM_EVENTS: &[&str] = &[
    "copy",
    "cut",
    "paste",
    "compositionend",
    "compositionstart",
    "compositionupdate",
    "drag",
    "dragend",
    "dragenter",
    "dragexit",
    "dragleave",
    "dragover",
    "dragstart",
    "drop",
    "focus",
    "focusin",
    "focusout",
    "blur",
    "change",
    "beforeinput",
    "formdata",
    "input",
    "reset",
    "submit",
    "invalid",
    "fullscreenchange",
    "fullscreenerror",
    "load",
    "error",
    "keydown",
    "keypress",
    "keyup",
    "auxclick",
    "click",
    "contextmenu",
    "gotpointercapture",
    "lostpointercapture",
    "dblclick",
    "mousedown",
    "mouseenter",
    "mouseleave",
    "mousemove",
    "mouseout",
    "mouseover",
    "mouseup",
    "abort",
    "canplay",
    "canplaythrough",
    "durationchange",
    "emptied",
    "encrypted",
    "ended",
    "loadeddata",
    "loadedmetadata",
    "loadstart",
    "pause",
    "play",
    "playing",
    "progress",
    "ratechange",
    "seeked",
    "seeking",
    "stalled",
    "suspend",
    "timeupdate",
    "volumechange",
    "waiting",
    "select",
    "scroll",
    "scrollend",
    "touchcancel",
    "touchend",
    "touchmove",
    "touchstart",
    "pointerdown",
    "pointermove",
    "pointerup",
    "pointercancel",
    "pointerenter",
    "pointerleave",
    "pointerover",
    "pointerout",
    "beforetoggle",
    "toggle",
    "wheel",
    "animationcancel",
    "animationstart",
    "animationend",
    "animationiteration",
    "securitypolicyviolation",
    "transitioncancel",
    "transitionend",
    "transitionrun",
    "transitionstart",
];

/// `vue/sort-keys`; default ascending, case-sensitive lexical order.
pub struct SortKeys {
    descending: bool,
    case_sensitive: bool,
    natural: bool,
}

impl Default for SortKeys {
    fn default() -> Self {
        Self {
            descending: false,
            case_sensitive: true,
            natural: false,
        }
    }
}

impl SortKeys {
    /// Build the rule from its ordering, case and natural-sort policy.
    pub fn new(descending: bool, case_sensitive: bool, natural: bool) -> Self {
        Self {
            descending,
            case_sensitive,
            natural,
        }
    }
}

impl Rule for SortKeys {
    fn meta(&self) -> &'static RuleMeta {
        &SORT_KEYS_META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut findings = Vec::new();
        collect_script_programs(ctx, |program, offset, _| {
            let mut visitor = SortedVueObjectKeys {
                offset,
                rule: self,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            findings.extend(visitor.findings);
        });
        for (current, previous, range) in findings {
            ctx.warn_at_with_help(
                cstr!("`{}` should be before `{}`", current, previous),
                range,
                "Keep keys in the configured order inside component option objects",
            );
        }
    }
}

struct SortedVueObjectKeys<'r> {
    offset: u32,
    rule: &'r SortKeys,
    findings: Vec<(String, String, ByteRange)>,
}

impl<'a> Visit<'a> for SortedVueObjectKeys<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if matches!(callee_name(call), Some("defineComponent" | "Vue.extend"))
            && let Some(object) = call.arguments.first().and_then(object_argument)
        {
            self.check_vue_object(object);
            return;
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        if let Some(expression) = declaration.declaration.as_expression()
            && let Expression::ObjectExpression(object) = expression.get_inner_expression()
        {
            self.check_vue_object(object);
            return;
        }
        walk::walk_export_default_declaration(self, declaration);
    }
}

impl SortedVueObjectKeys<'_> {
    fn check_vue_object(&mut self, object: &ObjectExpression<'_>) {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            let Some(parent) = property_name(&property.key) else {
                continue;
            };
            if parent == "model" {
                continue;
            }
            if let Expression::ObjectExpression(child) = property.value.get_inner_expression() {
                self.check_object(child);
            }
        }
    }

    fn check_object(&mut self, object: &ObjectExpression<'_>) {
        let mut previous: Option<&str> = None;
        for entry in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = entry else {
                previous = None;
                continue;
            };
            let Some(current) = property_name(&property.key) else {
                previous = None;
                continue;
            };
            if let Some(before) = previous
                && !self.rule.in_order(before, current)
            {
                self.findings.push((
                    current.into(),
                    before.into(),
                    absolute_range(self.offset, property.key.span()),
                ));
            }
            previous = Some(current);
        }
    }
}

impl SortKeys {
    fn in_order(&self, left: &str, right: &str) -> bool {
        let (left, right) = if self.case_sensitive {
            (left.into(), right.into())
        } else {
            (left.to_ascii_lowercase(), right.to_ascii_lowercase())
        };
        let ordering = if self.natural {
            natural_cmp(&left, &right)
        } else {
            left.cmp(&right)
        };
        if self.descending {
            !ordering.is_lt()
        } else {
            !ordering.is_gt()
        }
    }
}

fn natural_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    let left_key = natural_key(left);
    let right_key = natural_key(right);
    left_key.cmp(&right_key)
}

fn natural_key(value: &str) -> Vec<NaturalPart<'_>> {
    let mut parts = Vec::new();
    let mut start = 0;
    let bytes = value.as_bytes();
    while start < bytes.len() {
        let digits = bytes[start].is_ascii_digit();
        let mut end = start + 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() == digits {
            end += 1;
        }
        if let Some(part) = value.get(start..end) {
            parts.push(if digits {
                NaturalPart::Number(part.parse().unwrap_or(u64::MAX))
            } else {
                NaturalPart::Text(part)
            });
        }
        start = end;
    }
    parts
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum NaturalPart<'a> {
    Number(u64),
    Text(&'a str),
}

fn setup_program<T>(ctx: &LintContext<'_>, collect: impl FnOnce(&Program<'_>, u32, &str) -> T) -> T
where
    T: Default,
{
    let Some(descriptor) = ctx.sfc_descriptor() else {
        return T::default();
    };
    let Some(block) = descriptor.script_setup.as_ref() else {
        return T::default();
    };
    let Some(parsed) = ctx
        .sfc_scripts()
        .and_then(|scripts| scripts.script_setup())
        .filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
    else {
        return T::default();
    };
    collect(
        &parsed.program,
        block.loc.start as u32,
        block.content.as_ref(),
    )
}

fn collect_script_programs(
    ctx: &LintContext<'_>,
    mut collect: impl FnMut(&Program<'_>, u32, &str),
) {
    let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
        return;
    };
    if let (Some(block), Some(parsed)) = (descriptor.script.as_ref(), scripts.script())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        collect(
            &parsed.program,
            block.loc.start as u32,
            block.content.as_ref(),
        );
    }
    if let (Some(block), Some(parsed)) = (descriptor.script_setup.as_ref(), scripts.script_setup())
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
    {
        collect(
            &parsed.program,
            block.loc.start as u32,
            block.content.as_ref(),
        );
    }
}

fn collect_runtime_props(program: &Program<'_>, props: &mut FxHashSet<String>) {
    struct RuntimeProps<'p> {
        props: &'p mut FxHashSet<String>,
    }
    impl<'a> Visit<'a> for RuntimeProps<'_> {
        fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
            if callee_name(call) == Some("defineProps")
                && let Some(argument) = call.arguments.first().and_then(Argument::as_expression)
            {
                match argument.get_inner_expression() {
                    Expression::ArrayExpression(array) => {
                        for element in &array.elements {
                            if let Some(Expression::StringLiteral(prop)) = element.as_expression() {
                                self.props.insert(prop.value.as_str().into());
                            }
                        }
                    }
                    Expression::ObjectExpression(object) => {
                        for property in &object.properties {
                            let ObjectPropertyKind::ObjectProperty(property) = property else {
                                continue;
                            };
                            if let Some(name) = property_name(&property.key) {
                                self.props.insert(name.into());
                            }
                        }
                    }
                    _ => {}
                }
            }
            walk::walk_call_expression(self, call);
        }
    }
    RuntimeProps { props }.visit_program(program);
}

fn collect_prop_shadows(
    program: &Program<'_>,
    source: &str,
    offset: u32,
    props: &FxHashSet<String>,
) -> Vec<(String, ByteRange)> {
    let mut findings = Vec::new();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    let is_props_binding = declarator.init.as_ref().is_some_and(|initializer| {
                        source
                            .get(initializer.span().start as usize..initializer.span().end as usize)
                            .is_some_and(|text| text.contains("defineProps"))
                    });
                    if !is_props_binding {
                        collect_shadowed_pattern(&declarator.id, props, offset, &mut findings);
                    }
                }
            }
            Statement::FunctionDeclaration(function) if !function.declare => {
                if let Some(identifier) = &function.id
                    && props.contains(identifier.name.as_str())
                {
                    findings.push((
                        identifier.name.as_str().into(),
                        absolute_range(offset, identifier.span),
                    ));
                }
            }
            Statement::ClassDeclaration(class) if !class.declare => {
                if let Some(identifier) = &class.id
                    && props.contains(identifier.name.as_str())
                {
                    findings.push((
                        identifier.name.as_str().into(),
                        absolute_range(offset, identifier.span),
                    ));
                }
            }
            _ => {}
        }
    }
    findings
}

fn collect_shadowed_pattern(
    pattern: &BindingPattern<'_>,
    props: &FxHashSet<String>,
    offset: u32,
    findings: &mut Vec<(String, ByteRange)>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            if props.contains(identifier.name.as_str()) {
                findings.push((
                    identifier.name.as_str().into(),
                    absolute_range(offset, identifier.span),
                ));
            }
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_shadowed_pattern(&property.value, props, offset, findings);
            }
            if let Some(rest) = &object.rest {
                collect_shadowed_pattern(&rest.argument, props, offset, findings);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_shadowed_pattern(element, props, offset, findings);
            }
            if let Some(rest) = &array.rest {
                collect_shadowed_pattern(&rest.argument, props, offset, findings);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_shadowed_pattern(&assignment.left, props, offset, findings);
        }
    }
}

fn callee_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    match call.callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
        _ => None,
    }
}

fn object_argument<'a>(argument: &'a Argument<'a>) -> Option<&'a ObjectExpression<'a>> {
    match argument.as_expression()?.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::Identifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn property_span(object: &ObjectExpression<'_>, name: &str) -> Option<Span> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        (property_name(&property.key) == Some(name)).then(|| property.span())
    })
}

fn absolute_range(offset: u32, span: Span) -> ByteRange {
    ByteRange {
        start: offset + span.start,
        end: offset + span.end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn lint(rule: Box<dyn Rule>, source: &str) -> crate::linter::LintResult {
        let mut registry = RuleRegistry::new();
        registry.register(rule);
        Linter::with_registry(registry).lint_sfc(source, "Component.vue")
    }

    #[test]
    fn reports_style_mode_and_content_outside_blocks() {
        let style = lint(
            Box::new(EnforceStyleAttribute::default()),
            "<template><div /></template><style module>.x{}</style>",
        );
        assert_eq!(style.warning_count, 1);
        let outside = lint(
            Box::new(NoContentOutsideBlock),
            "stray<!-- kept --><template><div /></template>",
        );
        assert_eq!(outside.warning_count, 1);
    }

    #[test]
    fn reports_setup_macro_compatibility_failures() {
        let model = lint(
            Box::new(NoDefineModelDefault),
            "<script setup>defineModel({ default: 1 })</script><template />",
        );
        assert_eq!(model.warning_count, 1);
        let variable = lint(
            Box::new(RequireMacroVariableName),
            "<script setup>const p = defineProps({ value: String })</script><template />",
        );
        assert_eq!(variable.warning_count, 1);
        let shadow = lint(
            Box::new(NoPropsShadow),
            "<script setup>defineProps({ value: String }); const value = 1</script><template />",
        );
        assert_eq!(shadow.warning_count, 1);
    }

    #[test]
    fn reports_next_tick_native_events_and_unsorted_options() {
        let next_tick = lint(
            Box::new(NextTickStyle),
            "<script setup>nextTick(() => ready())</script><template />",
        );
        assert_eq!(next_tick.warning_count, 1);
        let event = lint(
            Box::new(NoShadowNativeEvents),
            "<script setup>const emit = defineEmits(['click'])</script><template />",
        );
        assert_eq!(event.warning_count, 1);
        let sorted = lint(
            Box::new(SortKeys::default()),
            "<script>export default { methods: { zebra() {}, alpha() {} } }</script><template />",
        );
        assert_eq!(sorted.warning_count, 1);
    }
}
