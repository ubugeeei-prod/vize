import { useModel as _useModel } from "vue";
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue";
export {};
export default {
  __name: "MkTab",
  props: {
    tabs: {
      type: Array,
      required: true
    },
    "modelValue": {}
  },
  emits: ["update:modelValue"],
  setup(__props) {
    const model = _useModel(__props, "modelValue");
    function update(key) {
      model.value = key;
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.tabsRoot) },
        [(_openBlock(true), _createElementBlock(
          _Fragment,
          null,
          _renderList(__props.tabs, (option) => {
            return _openBlock(), _createElementBlock("button", {
              key: option.key,
              class: _normalizeClass([
                "_button",
                _ctx.$style.tabButton,
                { [_ctx.$style.active]: _ctx.modelValue === option.key }
              ]),
              disabled: _ctx.modelValue === option.key,
              onClick: ($event) => update(option.key)
            }, [option.icon ? (_openBlock(), _createElementBlock(
              "i",
              {
                key: 0,
                class: _normalizeClass([option.icon, _ctx.$style.icon])
              },
              null,
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true), _createTextVNode(
              " " + _toDisplayString(option.label),
              1
              /* TEXT */
            )], 10, ["disabled", "onClick"]);
          }),
          128
          /* KEYED_FRAGMENT */
        ))],
        2
        /* CLASS */
      );
    };
  }
};
