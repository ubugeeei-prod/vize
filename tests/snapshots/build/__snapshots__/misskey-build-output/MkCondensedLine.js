import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue";
import { ref, watch } from "vue";
const contentSymbol = Symbol();
const observer = new ResizeObserver((entries) => {
  const results = [];
  for (const entry of entries) {
    const content = entry.target[contentSymbol] ? entry.target : entry.target.firstElementChild;
    const props = content[contentSymbol];
    const container = content.parentElement;
    const contentWidth = content.getBoundingClientRect().width;
    const containerWidth = container.getBoundingClientRect().width;
    results.push({
      container,
      transform: `scaleX(${Math.max(props.minScale, Math.min(1, containerWidth / contentWidth))})`
    });
  }
  for (const result of results) {
    result.container.style.transform = result.transform;
  }
});
export default {
  __name: "MkCondensedLine",
  setup(__props) {
    const props = __props;
    const content = ref();
    watch(content, (value, oldValue) => {
      if (oldValue != null) {
        delete oldValue[contentSymbol];
        observer.unobserve(oldValue);
        if (oldValue.parentElement) {
          observer.unobserve(oldValue.parentElement);
        }
      }
      if (value != null) {
        value[contentSymbol] = props;
        observer.observe(value);
        if (value.parentElement) {
          observer.observe(value.parentElement);
        }
      }
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "span",
        { class: _normalizeClass(_ctx.$style.container) },
        [_createElementVNode(
          "span",
          {
            ref_key: "content",
            ref: content,
            class: _normalizeClass(_ctx.$style.content),
            style: _normalizeStyle({ maxWidth: `${100 / _ctx.minScale}%` })
          },
          [_renderSlot(_ctx.$slots, "default")],
          6
          /* CLASS, STYLE */
        )],
        2
        /* CLASS */
      );
    };
  }
};
