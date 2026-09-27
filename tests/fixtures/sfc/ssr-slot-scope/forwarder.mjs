// A pure forwarding wrapper. Scope propagation is independent of fallthrough attrs.
export default {
  inheritAttrs: false,
  render() {
    return this.$slots.default?.();
  },
};
