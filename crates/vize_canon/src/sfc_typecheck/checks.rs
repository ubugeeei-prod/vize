//! Type checking functions for Vue SFC diagnostics.

use super::{SfcTypeCheckResult, SfcTypeDiagnostic, SfcTypeSeverity};
use vize_carton::cstr;
use vize_croquis::reactivity::ReactivityLossKind;
use vize_croquis::setup_context::ViolationSeverity;

/// Check props typing.
pub fn check_props_typing(
    summary: &vize_croquis::Croquis,
    script_offset: u32,
    result: &mut SfcTypeCheckResult,
    strict: bool,
) {
    use vize_croquis::macros::MacroKind;

    // Find defineProps call
    let define_props = summary
        .macros
        .all_calls()
        .iter()
        .find(|c| matches!(c.kind, MacroKind::DefineProps));

    let Some(define_props) = define_props else {
        return;
    };

    // Check if defineProps has type arguments (TypeScript generic)
    if define_props.type_args.is_some() {
        // Props are fully typed via TypeScript
        return;
    }

    let props = summary.macros.props();

    // defineProps() called without type argument and without runtime props
    if props.is_empty() {
        let (start, end) = (
            define_props.start + script_offset,
            define_props.end + script_offset,
        );

        result.add_diagnostic(SfcTypeDiagnostic {
            severity: if strict {
                SfcTypeSeverity::Error
            } else {
                SfcTypeSeverity::Warning
            },
            message: "defineProps() should have a type definition".into(),
            start,
            end,
            code: Some("untyped-props".into()),
            help: Some("Use defineProps<{ propName: Type }>() to define prop types".into()),
            related: Vec::new(),
        });
        return;
    }

    // Check each prop for runtime type
    for prop in props {
        if prop.prop_type.is_none() {
            let (start, end) = (
                define_props.start + script_offset,
                define_props.end + script_offset,
            );

            result.add_diagnostic(SfcTypeDiagnostic {
                severity: if strict {
                    SfcTypeSeverity::Error
                } else {
                    SfcTypeSeverity::Warning
                },
                message: cstr!("Prop '{}' should have a type definition", prop.name),
                start,
                end,
                code: Some("untyped-prop".into()),
                help: Some("Use defineProps<{ propName: Type }>() or define runtime type".into()),
                related: Vec::new(),
            });
            break; // Only report once per defineProps
        }
    }
}

/// Check emits typing.
pub fn check_emits_typing(
    summary: &vize_croquis::Croquis,
    script_offset: u32,
    result: &mut SfcTypeCheckResult,
    strict: bool,
) {
    use vize_croquis::macros::MacroKind;

    // Find defineEmits call
    let define_emits = summary
        .macros
        .all_calls()
        .iter()
        .find(|c| matches!(c.kind, MacroKind::DefineEmits));

    let Some(define_emits) = define_emits else {
        return;
    };

    // Check if defineEmits has type arguments
    if define_emits.type_args.is_some() {
        // Emits are fully typed via TypeScript
        return;
    }

    let emits = summary.macros.emits();

    // defineEmits() called without type argument and without runtime emits.
    // Dynamic runtime declarations such as `defineEmits({ ...emitsObject })`
    // or `defineEmits([...emits])` are not statically enumerable here, but the
    // generated virtual TS can still infer their event contract from the
    // runtime expression.
    if emits.is_empty() {
        if define_emits
            .runtime_args
            .as_deref()
            .is_some_and(has_inferable_runtime_emits)
        {
            return;
        }

        let (start, end) = (
            define_emits.start + script_offset,
            define_emits.end + script_offset,
        );

        result.add_diagnostic(SfcTypeDiagnostic {
            severity: if strict {
                SfcTypeSeverity::Error
            } else {
                SfcTypeSeverity::Warning
            },
            message: "defineEmits() should have a type definition".into(),
            start,
            end,
            code: Some("untyped-emits".into()),
            help: Some("Use defineEmits<{ (e: 'event', payload: Type): void }>()".into()),
            related: Vec::new(),
        });
        return;
    }

    // Check each emit for payload type
    for emit in emits {
        if emit.payload_type.is_none() {
            let (start, end) = (
                define_emits.start + script_offset,
                define_emits.end + script_offset,
            );

            result.add_diagnostic(SfcTypeDiagnostic {
                severity: if strict {
                    SfcTypeSeverity::Error
                } else {
                    SfcTypeSeverity::Warning
                },
                message: cstr!("Emit '{}' should have a type definition", emit.name),
                start,
                end,
                code: Some("untyped-emit".into()),
                help: Some("Use defineEmits<{ event: [payload: Type] }>()".into()),
                related: Vec::new(),
            });
            break; // Only report once per defineEmits
        }
    }
}

fn has_inferable_runtime_emits(runtime_args: &str) -> bool {
    let args = runtime_args.trim();
    !matches!(args, "" | "[]" | "{}")
}

/// Check template bindings for undefined references.
pub fn check_template_bindings(
    summary: &vize_croquis::Croquis,
    template_offset: u32,
    result: &mut SfcTypeCheckResult,
    _strict: bool,
) {
    // Report undefined references using croquis scope analysis
    for undef_ref in &summary.undefined_refs {
        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Error,
            message: cstr!(
                "Undefined reference '{}' in {}",
                undef_ref.name,
                undef_ref.context
            ),
            start: undef_ref.offset + template_offset,
            end: undef_ref.offset + template_offset + undef_ref.name.len() as u32,
            code: Some("undefined-binding".into()),
            help: Some(cstr!(
                "Make sure '{}' is defined in script setup or imported",
                undef_ref.name
            )),
            related: Vec::new(),
        });
    }
}

/// Check for reactivity loss patterns.
pub fn check_reactivity(
    summary: &vize_croquis::Croquis,
    script_offset: u32,
    result: &mut SfcTypeCheckResult,
    strict: bool,
) {
    let severity = if strict {
        SfcTypeSeverity::Error
    } else {
        SfcTypeSeverity::Warning
    };

    for loss in summary.reactivity.losses() {
        let message = match &loss.kind {
            ReactivityLossKind::ReactiveDestructure { source_name, .. } => {
                cstr!("Destructuring reactive object '{source_name}' loses reactivity")
            }
            ReactivityLossKind::RefValueDestructure { source_name, .. } => {
                cstr!("Destructuring ref '{source_name}' loses reactivity")
            }
            ReactivityLossKind::RefValueExtract {
                source_name,
                target_name,
            } => cstr!("Extracting '{target_name}' from '{source_name}.value' loses reactivity"),
            ReactivityLossKind::ReactivePropertyExtract {
                source_name,
                prop_name,
                target_name,
            } => cstr!(
                "Extracting '{target_name}' from '{source_name}.{prop_name}' loses reactivity"
            ),
            ReactivityLossKind::PropsDestructure { destructured_props } => cstr!(
                "Destructuring props loses reactivity for: {}",
                destructured_props.join(", ")
            ),
            ReactivityLossKind::FunctionArgumentExtract {
                argument_name,
                callee_name,
                ..
            } => cstr!(
                "Passing '{argument_name}' to '{callee_name}' captures a non-reactive snapshot"
            ),
            ReactivityLossKind::GetterCallExtract {
                context_name,
                getter_name,
                target_name,
                ..
            } => cstr!(
                "Assigning '{context_name}.{getter_name}()' to '{target_name}' extracts a non-reactive snapshot"
            ),
            ReactivityLossKind::PlainValueAlias {
                source_name,
                alias_name,
                target_name,
            } => cstr!(
                "Assigning plain snapshot '{alias_name}' to '{target_name}' keeps reactivity lost from '{source_name}'"
            ),
            ReactivityLossKind::ReactiveSpread { source_name } => {
                cstr!("Spreading reactive object '{source_name}' loses reactivity")
            }
            ReactivityLossKind::ReactiveReassign { source_name } => {
                cstr!("Reassigning reactive variable '{source_name}' disconnects reactivity")
            }
        };

        result.add_diagnostic(SfcTypeDiagnostic {
            severity,
            message,
            start: loss.start + script_offset,
            end: loss.end + script_offset,
            code: Some("reactivity-loss".into()),
            help: None,
            related: Vec::new(),
        });
    }
}

/// Check for setup context violations (CSRP / memory leaks).
pub fn check_setup_context(
    summary: &vize_croquis::Croquis,
    script_offset: u32,
    result: &mut SfcTypeCheckResult,
) {
    for violation in summary.setup_context.violations() {
        let severity = match violation.kind.severity() {
            ViolationSeverity::Error => SfcTypeSeverity::Error,
            ViolationSeverity::Warning => SfcTypeSeverity::Warning,
            ViolationSeverity::Info => SfcTypeSeverity::Info,
        };

        result.add_diagnostic(SfcTypeDiagnostic {
            severity,
            message: violation.kind.description().into(),
            start: violation.start + script_offset,
            end: violation.end + script_offset,
            code: Some(violation.kind.to_display().into()),
            help: None,
            related: Vec::new(),
        });
    }
}

/// Check for invalid exports in `<script setup>`.
pub fn check_invalid_exports(
    summary: &vize_croquis::Croquis,
    script_offset: u32,
    result: &mut SfcTypeCheckResult,
) {
    for export in &summary.invalid_exports {
        result.add_diagnostic(SfcTypeDiagnostic {
            severity: SfcTypeSeverity::Error,
            message: cstr!("Cannot export '{}' from <script setup>", export.name),
            start: export.start + script_offset,
            end: export.end + script_offset,
            code: Some("invalid-export".into()),
            help: Some(
                "Only type exports are allowed in <script setup>. Use a separate <script> block for value exports.".into(),
            ),
            related: Vec::new(),
        });
    }
}

/// Check for fallthrough attrs issues with multi-root components.
pub fn check_fallthrough_attrs(
    summary: &vize_croquis::Croquis,
    result: &mut SfcTypeCheckResult,
    strict: bool,
) {
    if !summary.template_info.may_lose_fallthrough_attrs() {
        return;
    }

    let severity = if strict {
        SfcTypeSeverity::Error
    } else {
        SfcTypeSeverity::Warning
    };

    result.add_diagnostic(SfcTypeDiagnostic {
        severity,
        message: "Multi-root component may lose fallthrough attributes".into(),
        start: summary.template_info.content_start,
        end: summary.template_info.content_end,
        code: Some("fallthrough-attrs".into()),
        help: Some("Bind $attrs explicitly to one root element or use inheritAttrs: false".into()),
        related: Vec::new(),
    });
}
