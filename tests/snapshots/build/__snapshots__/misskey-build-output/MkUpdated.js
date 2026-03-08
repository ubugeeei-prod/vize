import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { onMounted, useTemplateRef } from 'vue'
import { version } from '@@/js/config.js'
import MkModal from '@/components/MkModal.vue'
import MkButton from '@/components/MkButton.vue'
import MkSparkle from '@/components/MkSparkle.vue'
import { i18n } from '@/i18n.js'
import { confetti } from '@/utility/confetti.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUpdated',
  emits: ["closed"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const modal = useTemplateRef('modal');
const isBeta = version.includes('-beta') || version.includes('-alpha') || version.includes('-rc');
function whatIsNew() {
	modal.value?.close();
	window.open(`https://misskey-hub.net/docs/releases/#_${version.replace(/\./g, '')}`, '_blank');
}
onMounted(() => {
	confetti({
		duration: 1000 * 3,
	});
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModal, {
      ref_key: "modal", ref: modal,
      preferType: "dialog",
      zPriority: 'middle',
      onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(modal)?.close())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.title)
          }, [
            _createVNode(MkSparkle, null, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.misskeyUpdated), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ]),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.version)
          }, "✨" + _toDisplayString(_unref(version)) + "🚀", 1 /* TEXT */),
          (_unref(isBeta))
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.beta)
            }, _toDisplayString(_unref(i18n).ts.thankYouForTestingBeta), 1 /* TEXT */))
            : _createCommentVNode("v-if", true),
          _createVNode(MkButton, {
            full: "",
            onClick: whatIsNew
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.whatIsNew), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            class: _normalizeClass(_ctx.$style.gotIt),
            primary: "",
            full: "",
            onClick: _cache[2] || (_cache[2] = ($event: any) => (_unref(modal)?.close()))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.gotIt), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["zPriority"]))
}
}

})
