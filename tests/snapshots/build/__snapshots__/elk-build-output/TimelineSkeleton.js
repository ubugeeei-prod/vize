import {
  openBlock as _openBlock,
  createElementBlock as _createElementBlock,
  createVNode as _createVNode,
  resolveComponent as _resolveComponent,
} from "vue";

export function render(_ctx, _cache) {
  const _component_StatusCardSkeleton = _resolveComponent("StatusCardSkeleton");

  return (
    _openBlock(),
    _createElementBlock("div", null, [
      _createVNode(_component_StatusCardSkeleton, {
        border: "b base",
        op50: "",
      }),
      _createVNode(_component_StatusCardSkeleton, {
        border: "b base",
        op35: "",
      }),
      _createVNode(_component_StatusCardSkeleton, {
        border: "b base",
        op25: "",
      }),
      _createVNode(_component_StatusCardSkeleton, {
        border: "b base",
        op10: "",
      }),
    ])
  );
}
