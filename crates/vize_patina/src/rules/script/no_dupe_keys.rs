//! script/no-dupe-keys
//!
//! Disallow duplicate keys across Options API groups.
//!
//! A key declared as a `prop` must not be re-declared in `data`, `computed`,
//! `methods`, `setup`, or `inject` (and vice versa). All of these groups expose
//! their members on the component instance via `this`, so a duplicate name
//! silently shadows one declaration with another and is almost always a bug.
//!
//! This is a port of [`vue/no-dupe-keys`](https://eslint.vuejs.org/rules/no-dupe-keys.html)
//! from eslint-plugin-vue, covering the `props`, `data`, `computed`, `methods`,
//! `setup`, and `inject` groups.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: ['foo'],
//!   data() {
//!     return { foo: 1 } // duplicate of prop `foo`
//!   },
//!   computed: {
//!     bar() { return 2 }
//!   },
//!   methods: {
//!     bar() {} // duplicate of computed `bar`
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: ['foo'],
//!   data() {
//!     return { bar: 1 }
//!   },
//!   computed: {
//!     baz() { return 2 }
//!   }
//! }
//! ```

mod setup;
pub use setup::NoDupeKeysModuleScript;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;
use vize_croquis::OptionMember;
use vize_croquis::script_parser::collect_options_descriptor;
use vize_l0::{CompactString, FxHashMap};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-dupe-keys",
    description: "Disallow duplicate keys across Options API props/data/computed/methods/setup/inject",
    default_severity: Severity::Error,
};

/// Disallow duplicate keys across Options API groups.
pub struct NoDupeKeys;

impl ScriptRule for NoDupeKeys {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn as_setup_dupe_keys(&self) -> Option<&NoDupeKeys> {
        Some(self)
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some(descriptor) = collect_options_descriptor(program) else {
            return;
        };
        check_descriptor_members(&descriptor.members, offset, result);
    }
    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if sfc.is_sfc && sfc.is_script_setup {
            let module = if sfc.sole_script_block {
                NoDupeKeysModuleScript::Absent
            } else {
                NoDupeKeysModuleScript::Unavailable
            };
            self.check_setup_with_module(program, source, offset, module, result);
        } else {
            self.check_program(program, source, offset, result);
        }
    }
}

impl NoDupeKeys {
    pub(crate) fn setup_needs_module(program: &Program<'_>) -> bool {
        setup::has_candidate(program)
    }

    pub(crate) fn check_setup_with_module<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        module: NoDupeKeysModuleScript<'a>,
        result: &mut ScriptLintResult,
    ) {
        // Preserve every pre-existing Options descriptor diagnostic in this
        // block before adding the compiler-owned setup namespace check.
        self.check_program(program, source, offset, result);
        setup::check(program, offset, module, result);
    }
}

/// The Options API group a key was declared in. Stored alongside the first span
/// so a later duplicate can point back at the original declaration.
#[derive(Clone, Copy)]
struct KeyOrigin {
    group: &'static str,
    start: u32,
    end: u32,
}

fn check_descriptor_members(
    members: &[OptionMember],
    offset: usize,
    result: &mut ScriptLintResult,
) {
    // First declaration wins; subsequent declarations of the same key in any
    // group are reported as duplicates pointing back at the original.
    let mut seen: FxHashMap<CompactString, KeyOrigin> = FxHashMap::default();

    for member in members {
        record_key(&mut seen, member.group.label(), member, offset, result);
    }
}

fn record_key(
    seen: &mut FxHashMap<CompactString, KeyOrigin>,
    group: &'static str,
    member: &OptionMember,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let start = offset as u32 + member.start;
    let end = offset as u32 + member.end;

    if let Some(origin) = seen.get(&member.name) {
        let mut message = CompactString::with_capacity(member.name.len() + 48);
        message.push_str("Duplicated key '");
        message.push_str(&member.name);
        message.push('\'');

        let diagnostic = LintDiagnostic::error(META.name, message, start, end)
            .with_label(group_label(group), start, end)
            .with_label(
                first_declaration_label(origin.group),
                origin.start,
                origin.end,
            )
            .with_help(
                "props, data, computed, methods, setup, and inject share the component \
                 instance namespace; give each member a unique name.",
            );
        result.add_diagnostic(diagnostic);
        return;
    }

    seen.insert(member.name.clone(), KeyOrigin { group, start, end });
}

fn group_label(group: &'static str) -> CompactString {
    let mut label = CompactString::with_capacity(group.len() + 24);
    label.push_str("declared again in ");
    label.push_str(group);
    label
}

fn first_declaration_label(group: &'static str) -> CompactString {
    let mut label = CompactString::with_capacity(group.len() + 24);
    label.push_str("first declared in ");
    label.push_str(group);
    label
}

#[cfg(test)]
mod tests;
