import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, resolveDynamicComponent as _resolveDynamicComponent, renderSlot as _renderSlot, normalizeClass as _normalizeClass, normalizeProps as _normalizeProps, withCtx as _withCtx, unref as _unref } from "vue"

import { defineAsyncComponent, ref } from 'vue'
import { url as local } from '@@/js/config.js'
import { maybeMakeRelative } from '@@/js/url.js'
import type { MkABehavior } from '@/components/global/MkA.vue'
import { useTooltip } from '@/composables/use-tooltip.js'
import * as os from '@/os.js'
import { isEnabledUrlPreview } from '@/utility/url-preview.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkLink',
  props: {
    url: { type: String, required: true },
    rel: { type: String, required: false },
    navigationBehavior: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const maybeRelativeUrl = maybeMakeRelative(props.url, local);
const self = maybeRelativeUrl !== props.url;
const attr = self ? 'to' : 'href';
const target = self ? null : '_blank';
const el = ref<HTMLElement | { $el: HTMLElement }>();
if (isEnabledUrlPreview.value) {
	useTooltip(el, (showing) => {
		const anchorElement = el.value instanceof HTMLElement ? el.value : el.value?.$el;
		if (anchorElement == null) return;
		const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkUrlPreviewPopup.vue')), {
			showing,
			url: props.url,
			anchorElement: anchorElement,
		}, {
			closed: () => dispose(),
		});
	});
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(_unref(self) ? 'MkA' : 'a'), _normalizeProps({
      ref_key: "el", ref: el,
      style: "word-break: break-all;",
      class: "_link",
      [_ctx.attr || ""]: _unref(maybeRelativeUrl),
      rel: __props.rel ?? 'nofollow noopener',
      target: _unref(target),
      behavior: props.navigationBehavior,
      title: __props.url
    }), {
      default: _withCtx(() => [
        _renderSlot(_ctx.$slots, "default"),
        (_unref(target) === '_blank')
          ? (_openBlock(), _createElementBlock("i", {
            key: 0,
            class: _normalizeClass(["ti ti-external-link", _ctx.$style.icon])
          }))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 16 /* FULL_PROPS */, ["rel", "target", "behavior", "title"]))
}
}

})
