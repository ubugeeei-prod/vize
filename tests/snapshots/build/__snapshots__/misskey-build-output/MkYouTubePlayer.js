import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "icon ti ti-brand-youtube", style: "margin-right: 0.5em;" })
import { ref } from 'vue'
import { versatileLang } from '@@/js/intl-const.js'
import MkWindow from '@/components/MkWindow.vue'
import { transformPlayerUrl } from '@/utility/url-preview.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkYouTubePlayer',
  props: {
    url: { type: String, required: true }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const requestUrl = new URL(props.url);
if (!['http:', 'https:'].includes(requestUrl.protocol)) throw new Error('invalid url');
const fetching = ref(true);
const title = ref<string | null>(null);
const player = ref({
	url: null as string | null,
	width: null,
	height: null,
});
const ytFetch = (): void => {
	fetching.value = true;
	window.fetch(`/url?url=${encodeURIComponent(requestUrl.href)}&lang=${versatileLang}`).then(res => {
		res.json().then(info => {
			if (info.url == null) return;
			title.value = info.title;
			fetching.value = false;
			player.value = info.player;
		});
	});
};
ytFetch();

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkError = _resolveComponent("MkError")

  return (_openBlock(), _createBlock(MkWindow, {
      initialWidth: 640,
      initialHeight: 402,
      canResize: true,
      closeButton: true,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createElementVNode("span", null, _toDisplayString(title.value ?? 'YouTube'), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { class: "poamfof" }, [
          _createVNode(_Transition, {
            name: _unref(prefer).s.animation ? 'fade' : '',
            mode: "out-in"
          }, {
            default: _withCtx(() => [
              (player.value.url && (player.value.url.startsWith('http://') || player.value.url.startsWith('https://')))
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: "player"
                }, [
                  (!fetching.value)
                    ? (_openBlock(), _createElementBlock("iframe", {
                      key: 0,
                      src: _unref(transformPlayerUrl)(player.value.url),
                      frameborder: "0",
                      allow: "autoplay; encrypted-media",
                      allowfullscreen: ""
                    }))
                    : _createCommentVNode("v-if", true)
                ]))
                : (_openBlock(), _createElementBlock("span", { key: 1 }, "invalid url"))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["name"]),
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (!player.value.url)
              ? (_openBlock(), _createBlock(_component_MkError, {
                key: 1,
                onRetry: _cache[1] || (_cache[1] = ($event: any) => (ytFetch()))
              }))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["initialWidth", "initialHeight", "canResize", "closeButton"]))
}
}

})
