//! script/no-reserved-keys
//!
//! Disallow Vue-reserved names as Options API keys.
//!
//! Every member declared in `props`, `data`, `computed`, `methods`, `setup`, or
//! `inject` is exposed on the component instance via `this`. Names that start
//! with `$` or `_`, or that collide with a Vue built-in instance property
//! (`$el`, `$data`, `$props`, `$emit`, `$nextTick`, ...), are reserved by Vue:
//! the declaration is shadowed by the framework internal and silently breaks at
//! runtime.
//!
//! This is a port of [`vue/no-reserved-keys`](https://eslint.vuejs.org/rules/no-reserved-keys.html)
//! from eslint-plugin-vue, covering the `props`, `data`, `computed`, `methods`,
//! `setup`, and `inject` groups.
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: ['$el'],       // reserved: starts with `$`
//!   computed: { _foo() { return 1 } }, // reserved: starts with `_`
//!   methods: { emit() {} } // reserved: Vue built-in instance name
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;
use vize_carton::CompactString;
use vize_croquis::OptionMember;
use vize_croquis::script_parser::collect_options_descriptor;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-reserved-keys",
    description: "Disallow Vue-reserved names as Options API props/data/computed/methods/setup/inject keys",
    default_severity: Severity::Error,
};

/// Vue built-in instance property names, with their conventional `$` prefix
/// stripped. These are exposed on the instance (as `$el`, `$data`, ...), so a
/// bare member of the same name still collides with the framework internal once
/// Vue re-exposes it. The `$`/`_` prefix check already covers the prefixed
/// spellings; this list also flags the bare reserved words.
const RESERVED_INSTANCE_NAMES: &[&str] = &[
    "el",
    "data",
    "props",
    "attrs",
    "slots",
    "refs",
    "parent",
    "root",
    "options",
    "emit",
    "nextTick",
    "watch",
    "forceUpdate",
    "isMounted",
];

/// Disallow Vue-reserved names as Options API keys.
pub struct NoReservedKeys;

impl ScriptRule for NoReservedKeys {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
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
        for member in &descriptor.members {
            check_member(member, offset, result);
        }
    }
}

/// Why a member name is reserved, used to phrase the diagnostic.
enum Reservation {
    /// Starts with `$` or `_`.
    Prefix(char),
    /// Bare collision with a Vue built-in instance name.
    BuiltIn,
}

/// Classify a member name as reserved, if it is.
fn reservation_for(name: &str) -> Option<Reservation> {
    if let Some(first) = name.chars().next()
        && (first == '$' || first == '_')
    {
        return Some(Reservation::Prefix(first));
    }
    if RESERVED_INSTANCE_NAMES.contains(&name) {
        return Some(Reservation::BuiltIn);
    }
    None
}

fn check_member(member: &OptionMember, offset: usize, result: &mut ScriptLintResult) {
    let Some(reservation) = reservation_for(&member.name) else {
        return;
    };

    let start = offset as u32 + member.start;
    let end = offset as u32 + member.end;
    let group = member.group.label();

    let mut message = CompactString::with_capacity(member.name.len() + 24);
    message.push_str("Reserved key '");
    message.push_str(&member.name);
    message.push('\'');

    let reason: CompactString = match reservation {
        Reservation::Prefix(prefix) => {
            let mut label = CompactString::with_capacity(group.len() + 40);
            label.push_str(group);
            label.push_str(" key starts with reserved '");
            label.push(prefix);
            label.push('\'');
            label
        }
        Reservation::BuiltIn => {
            let mut label = CompactString::with_capacity(group.len() + 48);
            label.push_str(group);
            label.push_str(" key collides with a Vue built-in instance property");
            label
        }
    };

    let diagnostic = LintDiagnostic::error(META.name, message, start, end)
        .with_label(reason, start, end)
        .with_help(
            "Vue exposes built-in instance properties (`$el`, `$data`, `$emit`, ...) and \
             reserves names starting with `$` or `_`; rename this member so it is not \
             shadowed by a framework internal.",
        );
    result.add_diagnostic(diagnostic);
}

#[cfg(test)]
mod tests {
    use super::NoReservedKeys;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoReservedKeys));
        linter
    }

    #[test]
    fn test_valid_unreserved_keys() {
        let source = r#"
export default {
  props: ['foo'],
  data() {
    return { bar: 1 }
  },
  computed: {
    baz() { return 2 }
  },
  methods: {
    qux() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_options_object() {
        let source = r#"
import { ref } from 'vue'
const _private = ref(0)
const $special = ref(1)
"#;
        // Reserved-looking *locals* are not Options API members; nothing to flag.
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_prop_dollar_prefixed_array_form() {
        let source = r#"
export default {
  props: ['$el']
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_computed_underscore_prefixed() {
        let source = r#"
export default {
  computed: {
    _foo() { return 1 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_method_bare_builtin_name() {
        let source = r#"
export default {
  methods: {
    emit() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_prop_object_form_dollar_prefixed() {
        let source = r#"
export default {
  props: {
    $props: Number
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_data_bare_builtin_name() {
        let source = r#"
export default {
  data() {
    return { options: {} }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_inject_array_form_underscore() {
        let source = r#"
export default {
  inject: ['_theme']
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_setup_return_dollar_prefixed() {
        let source = r#"
export default {
  setup() {
    return { $emit: () => {} }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_multiple_reserved_reported() {
        let source = r#"
export default {
  props: ['$el', '_foo'],
  methods: {
    emit() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 3);
    }

    #[test]
    fn test_bare_builtin_not_in_tracked_group_is_ignored() {
        // `name` is a top-level option key, not a tracked group member, and is
        // not itself reserved; `watch` here is the option key (also untracked).
        let source = r#"
export default {
  name: 'MyComponent',
  watch: {
    foo() {}
  },
  computed: {
    title() { return 'ok' }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_spread_in_object_is_ignored() {
        let source = r#"
export default {
  computed: {
    ...mapGetters(['count'])
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_similar_but_unreserved_name() {
        // `emitter` merely starts with the reserved word `emit` but is not equal
        // to it and has no reserved prefix, so it must not be flagged.
        let source = r#"
export default {
  methods: {
    emitter() {},
    rootCause() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }
}
