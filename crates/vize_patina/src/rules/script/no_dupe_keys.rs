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

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;
use vize_carton::{CompactString, FxHashMap};
use vize_croquis::OptionMember;
use vize_croquis::script_parser::collect_options_descriptor;

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
mod tests {
    use super::NoDupeKeys;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDupeKeys));
        linter
    }

    #[test]
    fn test_valid_unique_keys() {
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
const foo = ref(0)
const foo2 = ref(1)
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_prop_duplicated_in_data() {
        let source = r#"
export default {
  props: ['foo'],
  data() {
    return { foo: 1 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_computed_duplicated_in_methods() {
        let source = r#"
export default {
  computed: {
    bar() { return 2 }
  },
  methods: {
    bar() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_prop_object_form_duplicated_in_computed() {
        let source = r#"
export default {
  props: {
    count: Number
  },
  computed: {
    count() { return 1 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_define_component_duplicate() {
        let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  data() {
    return { value: 1 }
  },
  methods: {
    value() {}
  }
})
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_identifier_export_duplicate() {
        let source = r#"
const component = {
  props: ['name'],
  methods: {
    name() {}
  }
}

export default component
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_arrow_data_concise_body_duplicate() {
        let source = r#"
export default {
  inject: ['theme'],
  data: () => ({ theme: 'dark' })
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_setup_return_duplicated_in_data() {
        let source = r#"
export default {
  data() {
    return { open: false }
  },
  setup() {
    return { open: true }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_multiple_duplicates_reported() {
        let source = r#"
export default {
  props: ['a', 'b'],
  computed: {
    a() { return 1 }
  },
  methods: {
    b() {}
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_inject_object_form_duplicate() {
        let source = r#"
export default {
  inject: {
    foo: { from: 'bar' }
  },
  computed: {
    foo() { return 1 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_spread_in_object_is_ignored() {
        let source = r#"
export default {
  props: ['foo'],
  computed: {
    ...mapGetters(['count'])
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }
}
