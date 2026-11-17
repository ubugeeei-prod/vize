use super::NextTickStyle;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

/// Source text of every reported range, in source order.
fn reported(source: &str) -> Vec<&str> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NextTickStyle));
    let mut diagnostics = Linter::with_registry(registry)
        .lint_sfc(source, "Component.vue")
        .diagnostics;
    diagnostics.sort_by_key(|diagnostic| diagnostic.start);
    diagnostics
        .iter()
        .filter_map(|diagnostic| source.get(diagnostic.start as usize..diagnostic.end as usize))
        .collect()
}

#[test]
fn reports_the_next_tick_identifier_like_eslint_plugin_vue() {
    let source = "<script>
import Vue, { nextTick as tick } from 'vue'
export default {
  mounted() {
    const vm = this
    this.$nextTick(() => this.measure())
    vm.$nextTick(this.measure)
    Vue.nextTick(this.measure)
    tick(() => undefined)
  },
}
</script><template />";
    assert_eq!(
        reported(source),
        ["$nextTick", "$nextTick", "nextTick", "tick"]
    );
}

#[test]
fn other_receivers_and_consumed_promises_stay_quiet() {
    let source = "<script>
export default {
  async mounted() {
    this.$refs.child.$nextTick(() => undefined)
    other.$nextTick(() => undefined)
    await this.$nextTick()
    this.$nextTick().then(() => undefined)
    this.$nextTick()?.then(() => undefined)
  },
}
</script><template />";
    assert_eq!(reported(source), Vec::<&str>::new());
}

#[test]
fn only_a_const_this_alias_in_scope_is_a_receiver() {
    let source = "<script>
export default {
  methods: {
    first() {
      const that = this
      that.$nextTick(() => undefined)
    },
    second() {
      var that = this
      setTimeout(function () {
        that.$nextTick(() => undefined)
      })
    },
    third() {
      let self = this
      self.$nextTick(() => undefined)
      const vm = this
      ;[1].forEach((vm) => vm.$nextTick(() => undefined))
    },
  },
}
</script><template />";
    assert_eq!(reported(source), ["$nextTick"]);
}
