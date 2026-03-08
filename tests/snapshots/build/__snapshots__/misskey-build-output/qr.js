import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-qrcode" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-scan" })
import { defineAsyncComponent, ref, shallowRef } from 'vue'
import MkQrShow from './qr.show.vue'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { ensureSignin } from '@/i'
import MkButton from '@/components/MkButton.vue'
import MkPolkadots from '@/components/MkPolkadots.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'qr',
  setup(__props) {

// router definitionでloginRequiredが設定されているためエラーハンドリングしない
const $i = ensureSignin();
const read = ref(false);
const MkQrRead = defineAsyncComponent(() => import('./qr.read.vue'));
definePage(() => ({
	title: i18n.ts.qr,
	icon: 'ti ti-qrcode',
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_pageScrollable", _ctx.$style.root])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(["_spacer", _ctx.$style.main])
      }, [ (read.value) ? (_openBlock(), _createBlock(MkButton, {
            key: 0,
            class: _normalizeClass(_ctx.$style.button),
            rounded: "",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (read.value = false))
          }, {
            default: _withCtx(() => [
              _hoisted_1,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts._qr.showTabTitle), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })) : (_openBlock(), _createBlock(MkButton, {
            key: 1,
            class: _normalizeClass(_ctx.$style.button),
            rounded: "",
            onClick: _cache[1] || (_cache[1] = ($event: any) => (read.value = true))
          }, {
            default: _withCtx(() => [
              _hoisted_2,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts._qr.readTabTitle), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })), (read.value) ? (_openBlock(), _createBlock(MkQrRead, { key: 0 })) : (_openBlock(), _createBlock(MkQrShow, { key: 1 })) ]), (!read.value) ? (_openBlock(), _createBlock(MkPolkadots, {
          key: 0,
          accented: "",
          revered: "",
          height: 200,
          style: "position: sticky; bottom: 0; margin-top: -200px;"
        }, null, 8 /* PROPS */, ["height"])) : _createCommentVNode("v-if", true) ]))
}
}

})
