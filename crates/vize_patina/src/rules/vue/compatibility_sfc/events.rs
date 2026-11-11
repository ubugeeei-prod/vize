//! Event declarations and emit calls that collide with browser-native names.
//!
//! Follows upstream's scope: the `defineEmits` macro (runtime or type
//! argument, a cross-file type through the SFC's type world) and a Vue
//! object's `emits` option; script calls of the `defineEmits` binding,
//! `this.$emit` and a `setup` context's `emit`; and template calls of `$emit`
//! and of the `<script setup>` binding's name.

use super::absolute_range;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::script::props_emits::require_explicit_emits::{
    MacroEntry, define_emits_entries, dispatched_events, options_emits_entries,
};
use crate::rules::script::setup_macros::setup_macros;
use crate::rules::script::template_scan::{DOLLAR_EMIT, for_each_template_call};
use crate::rules::script::vue_object::VueObjects;
use oxc_ast::ast::{BindingPattern, ObjectExpression, Program};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::ParserReturn;
use oxc_semantic::SemanticBuilder;
use vize_l0::{String, cstr};
use vize_relief::RootNode;

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
        let findings = script_findings(ctx);
        report(ctx, findings);
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        let (Some(surface), Some(_)) = (ctx.template_surface(), ctx.sfc_descriptor()) else {
            return;
        };
        let binding = setup_program(ctx).and_then(setup_emit_binding);
        if binding.is_none() && !ctx.source.contains(DOLLAR_EMIT) {
            return;
        }
        let mut findings = Vec::new();
        for_each_template_call(surface, |call| {
            if call.callee != DOLLAR_EMIT && Some(call.callee) != binding {
                return;
            }
            if let Some(event) = call.first_string_argument
                && is_native(event.value)
            {
                findings.push((
                    event.value.into(),
                    ByteRange {
                        start: event.start,
                        end: event.end,
                    },
                ));
            }
        });
        report(ctx, findings);
    }
}

fn report(ctx: &mut LintContext<'_>, findings: Vec<(String, ByteRange)>) {
    for (name, range) in findings {
        ctx.warn_at_with_help(
            cstr!("Avoid shadowing the native event `{}`", name),
            range,
            "Choose a component-specific event name so native listeners remain unambiguous",
        );
    }
}

fn clean<'p>(parsed: Option<&'p ParserReturn<'p>>) -> Option<&'p Program<'p>> {
    parsed
        .filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
        .map(|parsed| &parsed.program)
}

fn setup_program<'a>(ctx: &LintContext<'a>) -> Option<&'a Program<'a>> {
    clean(ctx.sfc_scripts()?.script_setup())
}

/// Upstream's template check names the binding of `const x = defineEmits()`.
fn setup_emit_binding<'a>(program: &'a Program<'a>) -> Option<&'a str> {
    setup_macros(program)
        .into_iter()
        .filter(|found| found.name == "defineEmits")
        .find_map(|found| match found.binding {
            Some(BindingPattern::BindingIdentifier(id)) => Some(id.name.as_str()),
            _ => None,
        })
}

struct Block<'a> {
    program: &'a Program<'a>,
    source: &'a str,
    offset: u32,
}

fn script_findings(ctx: &LintContext<'_>) -> Vec<(String, ByteRange)> {
    let (Some(descriptor), Some(scripts)) = (ctx.sfc_descriptor(), ctx.sfc_scripts()) else {
        return Vec::new();
    };
    let script = descriptor
        .script
        .as_ref()
        .zip(clean(scripts.script()))
        .map(|(block, program)| Block {
            program,
            source: block.content.as_ref(),
            offset: block.loc.start as u32,
        });
    let setup = descriptor
        .script_setup
        .as_ref()
        .zip(clean(scripts.script_setup()))
        .map(|(block, program)| Block {
            program,
            source: block.content.as_ref(),
            offset: block.loc.start as u32,
        });
    let mut findings = Vec::new();
    if let Some(block) = &script {
        block_findings(ctx, block, None, &mut findings);
    }
    if let Some(block) = &setup {
        block_findings(ctx, block, Some(script.as_ref()), &mut findings);
    }
    findings
}

/// `sibling` is `Some` for `<script setup>`, holding the plain `<script>`.
fn block_findings(
    ctx: &LintContext<'_>,
    block: &Block<'_>,
    sibling: Option<Option<&Block<'_>>>,
    findings: &mut Vec<(String, ByteRange)>,
) {
    // `emit`, `$emit`, `emits` and `defineEmits` all contain it.
    if !block.source.contains("mit") {
        return;
    }
    let built = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(block.program);
    let semantic = &built.semantic;
    let mut declared = Vec::new();
    let mut setup_emit = None;
    if let Some(sibling) = sibling {
        let sibling_program = sibling.map(|sibling| sibling.program);
        for found in setup_macros(block.program)
            .into_iter()
            .filter(|found| found.name == "defineEmits")
        {
            if let Some(BindingPattern::BindingIdentifier(id)) = found.binding {
                setup_emit = id.symbol_id.get();
            }
            let entries = |world| {
                define_emits_entries(
                    found.call,
                    semantic,
                    block.program,
                    sibling_program,
                    block.source,
                    world,
                )
            };
            let (mut entries_found, unresolved) = entries(None);
            if unresolved && let world @ Some(_) = ctx.macro_type_world() {
                entries_found = entries(world).0;
            }
            declared.extend(entries_found);
        }
    }

    let objects = VueObjects::collect(block.program, block.source, semantic, sibling.is_none());
    let mut options = OptionsEmits {
        objects: &objects,
        entries: Vec::new(),
    };
    options.visit_program(block.program);
    declared.extend(options.entries);

    let sibling_offset = sibling.flatten().map(|sibling| sibling.offset);
    for entry in declared.into_iter().filter(|entry| is_native(&entry.name)) {
        let offset = match (entry.sibling, sibling_offset) {
            (true, Some(offset)) => offset,
            _ => block.offset,
        };
        findings.push((
            entry.name.as_str().into(),
            absolute_range(offset, entry.span),
        ));
    }
    let calls = dispatched_events(block.program, semantic, &objects, setup_emit);
    for (name, span) in calls.into_iter().filter(|(name, _)| is_native(name)) {
        findings.push((name.as_str().into(), absolute_range(block.offset, span)));
    }
}

/// The `emits` option of every Vue object.
struct OptionsEmits<'o> {
    objects: &'o VueObjects,
    entries: Vec<MacroEntry>,
}

impl<'a> Visit<'a> for OptionsEmits<'_> {
    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        if self.objects.contains(it) {
            self.entries.extend(options_emits_entries(it));
        }
        walk::walk_object_expression(self, it);
    }
}

fn is_native(name: &str) -> bool {
    DOM_EVENTS.contains(&name.to_ascii_lowercase().as_str())
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
