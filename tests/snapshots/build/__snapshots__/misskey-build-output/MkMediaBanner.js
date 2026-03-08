import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { i18n } from '@/i18n.js'
import MkMediaAudio from '@/components/MkMediaAudio.vue'
import { shouldHideFileByDefault, canRevealFile } from '@/utility/sensitive-file.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMediaBanner',
  props: {
    media: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const hide = ref(shouldHideFileByDefault(props.media));
async function reveal() {
	if (!(await canRevealFile(props.media))) {
		return;
	}
	hide.value = false;
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ (__props.media.type.startsWith('audio') && __props.media.type !== 'audio/midi') ? (_openBlock(), _createBlock(MkMediaAudio, {
          key: 0,
          audio: __props.media
        }, null, 8 /* PROPS */, ["audio"])) : (hide.value) ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: _normalizeClass(_ctx.$style.sensitive),
            onClick: reveal
          }, [ _createElementVNode("span", { style: "font-size: 1.6em;" }, [ _hoisted_1 ]), _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.sensitive), 1 /* TEXT */), _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts.clickToShow), 1 /* TEXT */) ])) : (_openBlock(), _createElementBlock("a", {
          key: 2,
          class: _normalizeClass(_ctx.$style.download),
          href: __props.media.url,
          title: __props.media.name,
          download: __props.media.name
        }, [ _createElementVNode("span", { style: "font-size: 1.6em;" }, [ _hoisted_2 ]), _createElementVNode("b", null, _toDisplayString(__props.media.name), 1 /* TEXT */) ])) ]))
}
}

})
