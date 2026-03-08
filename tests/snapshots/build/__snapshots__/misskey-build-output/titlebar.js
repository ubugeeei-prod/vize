import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
import { host } from '@@/js/config.js'
import { ref } from 'vue'
import { instance } from '@/instance.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'titlebar',
  setup(__props) {

const canBack = ref(true);
function goBack() {
	window.history.back();
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.title)
      }, [ _createElementVNode("img", {
          src: _unref(instance).iconUrl || '/favicon.ico',
          alt: "",
          class: _normalizeClass(_ctx.$style.instanceIcon)
        }, null, 8 /* PROPS */, ["src"]), _createElementVNode("span", {
          class: _normalizeClass(_ctx.$style.instanceTitle)
        }, _toDisplayString(_unref(instance).name ?? _unref(host)), 1 /* TEXT */) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.controls)
      }, [ _createElementVNode("span", {
          class: _normalizeClass(_ctx.$style.left)
        }, [ (canBack.value) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: _normalizeClass(["_button", _ctx.$style.button]),
              onClick: goBack
            }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("span", {
          class: _normalizeClass(_ctx.$style.right)
        }) ]) ]))
}
}

})
