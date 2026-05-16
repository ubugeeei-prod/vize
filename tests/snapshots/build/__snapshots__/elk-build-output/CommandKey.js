import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'CommandKey',
  props: {
    name: { type: String, required: true }
  },
  setup(__props: any) {

const isMac = useIsMac()
const keys = computed(() => __props.name.toLowerCase().split('+'))

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "flex items-center px-1" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(keys.value, (key, index) => {
        return (_openBlock(), _createElementBlock(_Fragment, { key: key }, [
          (index > 0)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "inline-block px-.5"
            }, "\n        +\n      "))
            : _createCommentVNode("v-if", true),
          _createElementVNode("div", {
            class: "p-1 grid place-items-center rounded-lg shadow-sm",
            text: "xs secondary",
            border: "1 base"
          }, [
            (key === 'enter')
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                "i-material-symbols:keyboard-return-rounded": ""
              }))
              : (key === 'meta' && _unref(isMac))
                ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  "i-material-symbols:keyboard-command-key": ""
                }))
              : (key === 'meta' && !_unref(isMac))
                ? (_openBlock(), _createElementBlock("div", {
                  key: 2,
                  "i-material-symbols:window-sharp": ""
                }))
              : (key === 'alt' && _unref(isMac))
                ? (_openBlock(), _createElementBlock("div", {
                  key: 3,
                  "i-material-symbols:keyboard-option-key-rounded": ""
                }))
              : (key === 'arrowup')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 4,
                  "i-ri:arrow-up-line": ""
                }))
              : (key === 'arrowdown')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 5,
                  "i-ri:arrow-down-line": ""
                }))
              : (key === 'arrowleft')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 6,
                  "i-ri:arrow-left-line": ""
                }))
              : (key === 'arrowright')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 7,
                  "i-ri:arrow-right-line": ""
                }))
              : (key === 'escape')
                ? (_openBlock(), _createElementBlock(_Fragment, { key: 8 }, [
                  _createTextVNode("\n          ESC\n        ")
                ], 64 /* STABLE_FRAGMENT */))
              : (_openBlock(), _createElementBlock("div", {
                key: 9,
                class: _normalizeClass({ 'px-.5': key.length === 1 })
              }, _toDisplayString(key[0].toUpperCase() + key.slice(1)), 1 /* TEXT */))
          ])
        ], 64 /* STABLE_FRAGMENT */))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
