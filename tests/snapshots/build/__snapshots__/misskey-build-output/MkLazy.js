import { openBlock as _openBlock, createElementBlock as _createElementBlock, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue";
import { nextTick, onMounted, onActivated, onBeforeUnmount, ref, useTemplateRef } from "vue";
export default {
  __name: "MkLazy",
  setup(__props) {
    const rootEl = useTemplateRef("rootEl");
    const showing = ref(false);
    const observer = new IntersectionObserver((entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        showing.value = true;
      }
    });
    onMounted(() => {
      nextTick(() => {
        observer.observe(rootEl.value);
      });
    });
    onActivated(() => {
      nextTick(() => {
        observer.observe(rootEl.value);
      });
    });
    onBeforeUnmount(() => {
      observer.disconnect();
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        {
          ref_key: "rootEl",
          ref: rootEl,
          class: _normalizeClass(_ctx.$style.root)
        },
        [!showing.value ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass(_ctx.$style.placeholder)
          },
          null,
          2
          /* CLASS */
        )) : _renderSlot(_ctx.$slots, "default", { key: 1 })],
        2
        /* CLASS */
      );
    };
  }
};
