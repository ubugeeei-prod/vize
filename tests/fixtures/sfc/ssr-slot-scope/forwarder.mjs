// A pure forwarding wrapper. Scope propagation is independent of fallthrough attrs.
export default {
  inheritAttrs: false,
  render() {
    return this.$slots.default?.();
  },
};

// A real renderSlot introduces an incoming scope from this scoped owner.
export function scopedForwarder(renderSlot) {
  return {
    inheritAttrs: false,
    __scopeId: "data-v-forwarder",
    render() {
      return renderSlot(this.$slots, "default");
    },
  };
}
