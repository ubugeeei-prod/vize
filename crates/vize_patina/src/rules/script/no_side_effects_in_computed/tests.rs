use super::NoSideEffectsInComputed;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoSideEffectsInComputed));
    linter
}

#[test]
fn test_valid_pure_getter() {
    let source = r#"
export default {
  data() {
    return { count: 0 }
  },
  computed: {
    doubled() {
      return this.count * 2
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_options_object() {
    let source = r#"
import { computed } from 'vue'
const doubled = computed(() => count.value * 2)
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_no_computed() {
    let source = r#"
export default {
  data() {
    return { count: 0 }
  },
  methods: {
    bump() {
      this.count++
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_assignment_to_this() {
    let source = r#"
export default {
  data() {
    return { count: 0 }
  },
  computed: {
    doubled() {
      this.count = this.count * 2
      return this.count
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_compound_assignment_to_this() {
    let source = r#"
export default {
  computed: {
    total() {
      this.sum += 1
      return this.sum
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_update_expression_on_this() {
    let source = r#"
export default {
  computed: {
    next() {
      this.count++
      return this.count
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_nested_member_assignment() {
    let source = r#"
export default {
  computed: {
    label() {
      this.state.label = 'x'
      return this.state.label
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_computed_member_assignment() {
    let source = r#"
export default {
  computed: {
    first() {
      this.items[0] = 1
      return this.items[0]
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_mutating_array_method() {
    let source = r#"
export default {
  computed: {
    reversed() {
      return this.items.reverse()
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_push_on_nested_this() {
    let source = r#"
export default {
  computed: {
    grow() {
      this.state.list.push(1)
      return this.state.list
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_mutating_method_on_local_copy() {
    // `reverse` on a fresh array literal is fine — no `this` receiver.
    let source = r#"
export default {
  computed: {
    reversed() {
      return [...this.items].reverse()
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_component_method_named_sort_is_not_assumed_to_mutate() {
    let source = r#"
export default {
  computed: {
    items() {
      return this.sort()
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_assignment_to_local_variable() {
    let source = r#"
export default {
  computed: {
    sum() {
      let total = 0
      for (const n of this.items) {
        total += n
      }
      return total
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_assignment_in_nested_function() {
    // A non-arrow nested function rebinds `this`, so writing to its `this`
    // does not mutate the component instance.
    let source = r#"
export default {
  computed: {
    handler() {
      return function () {
        this.count = 1
      }
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_accessor_object_getter() {
    let source = r#"
export default {
  computed: {
    value: {
      get() {
        this.count = 1
        return this.count
      },
      set(v) {
        this.count = v
      }
    }
  }
}
"#;
    // Only the getter is checked; the setter's assignment is allowed.
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_define_component() {
    let source = r#"
import { defineComponent } from 'vue'

export default defineComponent({
  computed: {
    doubled() {
      this.count *= 2
      return this.count
    }
  }
})
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_multiple_side_effects_reported() {
    let source = r#"
export default {
  computed: {
    a() {
      this.x = 1
      this.y++
      return this.x
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_valid_non_mutating_array_method() {
    let source = r#"
export default {
  computed: {
    mapped() {
      return this.items.map((n) => n * 2)
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 0);
}

#[test]
fn reports_this_set_in_options_computed() {
    let source = r#"
export default {
  computed: {
    rows() {
      this.$set(row, index, cell)
      return row
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        &source[diagnostic.start as usize..diagnostic.end as usize],
        "$set"
    );
}

#[test]
fn reports_object_assign_and_delete_on_external_state() {
    let source = r#"
import { computed, ref } from 'vue'
const settings = ref({ theme: 'light' })
const assigned = computed(() => Object.assign(settings.value, { theme: 'dark' }))
const removed = computed(() => delete settings.value.theme)
const fresh = computed(() => Object.assign({}, settings.value))
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2, "{:?}", result.diagnostics);
}

#[test]
fn reports_object_assign_and_delete_on_this() {
    let source = r#"
export default {
  computed: {
    changed() { return Object.assign(this.settings, { theme: 'dark' }) },
    removed() { return delete this.settings.theme }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.error_count, 2, "{:?}", result.diagnostics);
}
