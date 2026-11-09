//! Event declarations that collide with browser-native event names.

use super::{absolute_range, callee_name, collect_script_programs, property_name};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectPropertyKind};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::{String, cstr};

static META: RuleMeta = RuleMeta {
    name: "vue/no-shadow-native-events",
    description: "Disallow component events that shadow native events",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-shadow-native-events`.
pub struct NoShadowNativeEvents;

impl Rule for NoShadowNativeEvents {
    fn meta(&self) -> &'static RuleMeta {
        &META
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
