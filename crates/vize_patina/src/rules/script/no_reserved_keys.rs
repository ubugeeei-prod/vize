//! script/no-reserved-keys
//!
//! Disallow Vue-reserved names as Options API keys.
//!
//! The default Options API groups are `props`, `data`, `computed`, `methods`,
//! and `setup`. Only exact Vue-reserved public instance spellings are forbidden
//! in those groups; a leading `_` is additionally reserved in returned `data`.
//! Custom `$` names and private `_` members in the other groups are valid.
//! The existing descriptor does not provide `asyncData` or setup macro props.
//!
//! This is a bounded port of [`vue/no-reserved-keys`](https://eslint.vuejs.org/rules/no-reserved-keys.html)
//! from eslint-plugin-vue; `inject` is outside its default groups.
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: ['$el'],
//!   data() { return { _private: 1 } },
//!   methods: { $emit() {} }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;
use vize_croquis::script_parser::collect_options_descriptor;
use vize_croquis::{OptionGroup, OptionMember};
use vize_l0::CompactString;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-reserved-keys",
    description: "Disallow Vue-reserved names as Options API props/data/computed/methods/setup keys",
    default_severity: Severity::Error,
};

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

/// Exact default reserved spellings from eslint-plugin-vue 10.9.2.
const RESERVED_PUBLIC_KEYS: &[&str] = &[
    "$data",
    "$props",
    "$el",
    "$options",
    "$parent",
    "$root",
    "$children",
    "$slots",
    "$scopedSlots",
    "$refs",
    "$isServer",
    "$attrs",
    "$listeners",
    "$watch",
    "$set",
    "$delete",
    "$on",
    "$once",
    "$off",
    "$emit",
    "$mount",
    "$forceUpdate",
    "$nextTick",
    "$destroy",
];

fn reserved_prefix(member: &OptionMember) -> Option<char> {
    match member.group {
        OptionGroup::Inject => None,
        OptionGroup::Data if member.name.starts_with('_') => Some('_'),
        OptionGroup::Props
        | OptionGroup::Computed
        | OptionGroup::Methods
        | OptionGroup::Data
        | OptionGroup::Setup => {
            if member.name.starts_with('$') {
                RESERVED_PUBLIC_KEYS
                    .contains(&member.name.as_str())
                    .then_some('$')
            } else {
                None
            }
        }
    }
}

fn check_member(member: &OptionMember, offset: usize, result: &mut ScriptLintResult) {
    let Some(prefix) = reserved_prefix(member) else {
        return;
    };

    let start = offset as u32 + member.start;
    let end = offset as u32 + member.end;
    let group = member.group.label();

    let mut message = CompactString::with_capacity(member.name.len() + 24);
    message.push_str("Reserved key '");
    message.push_str(&member.name);
    message.push('\'');

    let mut reason = CompactString::with_capacity(group.len() + 40);
    reason.push_str(group);
    reason.push_str(" key starts with reserved '");
    reason.push(prefix);
    reason.push('\'');

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
    fn test_computed_underscore_is_valid() {
        let source = r#"
export default {
  computed: {
    _foo() { return 1 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_method_bare_instance_name_is_distinct() {
        let source = r#"
export default {
  methods: {
    emit() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
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
    fn test_data_bare_instance_name_is_distinct() {
        let source = r#"
export default {
  data() {
    return { options: {} }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_inject_array_is_outside_default_groups() {
        let source = r#"
export default {
  inject: ['_theme']
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
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
        assert_eq!(result.error_count, 1);
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
