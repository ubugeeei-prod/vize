import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeProps as _normalizeProps, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"

import { defineAsyncComponent, ref } from 'vue'
import { toUnicode as decodePunycode } from 'punycode.js'
import { url as local } from '@@/js/config.js'
import { maybeMakeRelative } from '@@/js/url.js'
import type { MkABehavior } from '@/components/global/MkA.vue'
import * as os from '@/os.js'
import { useTooltip } from '@/composables/use-tooltip.js'
import { isEnabledUrlPreview } from '@/utility/url-preview.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUrl',
  props: {
    url: { type: String, required: true },
    rel: { type: String, required: false },
    showUrlPreview: { type: Boolean, required: false, default: true },
    navigationBehavior: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
function safeURIDecode(str: string): string {
	try {
		return decodeURIComponent(str);
	} catch {
		return str;
	}
}
const maybeRelativeUrl = maybeMakeRelative(props.url, local);
const self = maybeRelativeUrl !== props.url;
const url = new URL(props.url);
if (!['http:', 'https:'].includes(url.protocol)) throw new Error('invalid url');
const el = ref();
if (props.showUrlPreview && isEnabledUrlPreview.value) {
	useTooltip(el, (showing) => {
		const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkUrlPreviewPopup.vue')), {
			showing,
			url: props.url,
			anchorElement: el.value instanceof HTMLElement ? el.value : el.value?.$el,
		}, {
			closed: () => dispose(),
		});
	});
}
const schema = url.protocol;
const hostname = decodePunycode(url.hostname);
const port = url.port;
const pathname = safeURIDecode(url.pathname);
const query = safeURIDecode(url.search);
const hash = safeURIDecode(url.hash);
const attr = self ? 'to' : 'href';
const target = self ? null : '_blank';

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(_resolveDynamicComponent(_unref(self) ? 'MkA' : 'a'), _normalizeProps({
      ref_key: "el", ref: el,
      class: _normalizeClass(["_link", _ctx.$style.root]),
      [_ctx.attr || ""]: _unref(maybeRelativeUrl),
      rel: __props.rel ?? 'nofollow noopener',
      target: _unref(target),
      behavior: props.navigationBehavior,
      onContextmenu: _cache[0] || (_cache[0] = _withModifiers(() => {}, ["stop"]))
    }), {
      default: _withCtx(() => [
        (!_unref(self))
          ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
            _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.schema)
            }, _toDisplayString(_unref(schema)) + "//", 1 /* TEXT */),
            _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.hostname)
            }, _toDisplayString(_unref(hostname)), 1 /* TEXT */),
            (_unref(port) != '')
              ? (_openBlock(), _createElementBlock("span", { key: 0 }, ":" + _toDisplayString(_unref(port)), 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ], 64 /* STABLE_FRAGMENT */))
          : _createCommentVNode("v-if", true),
        (_unref(pathname) === '/' && _unref(self))
          ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            class: _normalizeClass(_ctx.$style.self)
          }, _toDisplayString(_unref(hostname)), 1 /* TEXT */))
          : _createCommentVNode("v-if", true),
        (_unref(pathname) != '')
          ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            class: _normalizeClass(_ctx.$style.pathname)
          }, _toDisplayString(_unref(self) ? _unref(pathname).substring(1) : _unref(pathname)), 1 /* TEXT */))
          : _createCommentVNode("v-if", true),
        _createElementVNode("span", {
          class: _normalizeClass(_ctx.$style.query)
        }, _toDisplayString(_unref(query)), 1 /* TEXT */),
        _createElementVNode("span", {
          class: _normalizeClass(_ctx.$style.hash)
        }, _toDisplayString(_unref(hash)), 1 /* TEXT */),
        (_unref(target) === '_blank')
          ? (_openBlock(), _createElementBlock("i", {
            key: 0,
            class: _normalizeClass(["ti ti-external-link", _ctx.$style.icon])
          }))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 16 /* FULL_PROPS */, ["rel", "target", "behavior"]))
}
}

})
