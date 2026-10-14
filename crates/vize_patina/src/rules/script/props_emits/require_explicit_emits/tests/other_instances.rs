//! `other.$emit(...)` emits on another instance (#9076).

use super::create_linter;

#[test]
fn emits_on_other_instances_are_not_this_components() {
    let source = r#"
export default {
  emits: ['change'],
  methods: {
    pick(picker) {
      picker.$emit('pick', new Date())
      ;(this.$datepicker).$emit('week-number-click', 1)
      this.$emit('change')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn this_emits_still_need_a_declaration() {
    let source = r#"
export default {
  emits: ['change'],
  methods: {
    pick() {
      this.$emit('pick')
      ;(this).$emit('other')
    }
  }
}
"#;
    let result = create_linter().lint(source, 0);
    assert_eq!(result.warning_count, 2);
}
