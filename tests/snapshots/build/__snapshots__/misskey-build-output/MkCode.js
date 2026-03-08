import { defineComponent as _defineComponent } from 'vue'
import { Suspense as _Suspense, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-copy" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
import { defineAsyncComponent, ref } from 'vue'
import { i18n } from '@/i18n.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkCode',
  props: {
    code: { type: String, required: true },
    forceShow: { type: Boolean, required: false, default: false },
    copyButton: { type: Boolean, required: false, default: true },
    withOuterStyle: { type: Boolean, required: false, default: true },
    lang: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const show = ref(props.forceShow === true ? true : !prefer.s.dataSaver.code);
const XCode = defineAsyncComponent(() => import('@/components/MkCode.core.vue'));
function copy() {
	copyToClipboard(props.code);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.codeBlockRoot)
    }, [ (__props.copyButton) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          class: _normalizeClass(["_button", [_ctx.$style.codeBlockCopyButton, { [_ctx.$style.withOuterStyle]: __props.withOuterStyle }]]),
          onClick: copy
        }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true), _createVNode(_Suspense, null, {
        fallback: _withCtx(() => [
          _createElementVNode("pre", {
            class: _normalizeClass(["_selectable", [_ctx.$style.codeBlockFallbackRoot, {
  					[_ctx.$style.outerStyle]: __props.withOuterStyle,
  				}]])
          }, [
            _createElementVNode("code", {
              class: _normalizeClass(_ctx.$style.codeBlockFallbackCode)
            }, "Loading...")
          ], 2 /* CLASS */)
        ]),
        default: _withCtx(() => [
          (show.value && __props.lang)
            ? (_openBlock(), _createBlock(XCode, {
              key: 0,
              class: "_selectable",
              code: __props.code,
              lang: __props.lang,
              withOuterStyle: __props.withOuterStyle
            }, null, 8 /* PROPS */, ["code", "lang", "withOuterStyle"]))
            : (show.value)
              ? (_openBlock(), _createElementBlock("pre", {
                key: 1,
                class: _normalizeClass(["_selectable", [_ctx.$style.codeBlockFallbackRoot, {
  				[_ctx.$style.outerStyle]: __props.withOuterStyle,
  			}]])
              }, [
                _createElementVNode("code", {
                  class: _normalizeClass(_ctx.$style.codeBlockFallbackCode)
                }, _toDisplayString(__props.code), 1 /* TEXT */)
              ]))
            : (_openBlock(), _createElementBlock("button", {
              key: 2,
              class: _normalizeClass(_ctx.$style.codePlaceholderRoot),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (show.value = true))
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.codePlaceholderContainer)
              }, [
                _createElementVNode("div", null, [
                  _hoisted_2,
                  _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.code), 1 /* TEXT */)
                ]),
                _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts.clickToShow), 1 /* TEXT */)
              ])
            ]))
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
