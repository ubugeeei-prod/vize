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
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
    insta::assert_debug_snapshot!(result.diagnostics);
    });
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
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
    insta::assert_debug_snapshot!(result.diagnostics);
    });
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
