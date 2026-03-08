import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rss" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
import { ref, watch, computed } from 'vue'
import * as Misskey from 'misskey-js'
import { url as base } from '@@/js/config.js'
import { useInterval } from '@@/js/use-interval.js'
import { useWidgetPropsManager } from './widget.js'
import { i18n } from '@/i18n.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkContainer from '@/components/MkContainer.vue'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'rss';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetRss',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	url: {
		type: 'string',
		label: i18n.ts._widgetOptions._rss.url,
		default: 'http://feeds.afpbb.com/rss/afpbb/afpbbnews',
		manualSave: true,
	},
	refreshIntervalSec: {
		type: 'number',
		label: i18n.ts._widgetOptions._rss.refreshIntervalSec,
		default: 60,
	},
	maxEntries: {
		type: 'number',
		label: i18n.ts._widgetOptions._rss.maxEntries,
		default: 15,
	},
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const rawItems = ref<Misskey.entities.FetchRssResponse['items']>([]);
const items = computed(() => rawItems.value.slice(0, widgetProps.maxEntries));
const fetching = ref(true);
const fetchEndpoint = computed(() => {
	const url = new URL('/api/fetch-rss', base);
	url.searchParams.set('url', widgetProps.url);
	return url.toString();
});
const intervalClear = ref<(() => void) | undefined>();
const tick = () => {
	if (window.document.visibilityState === 'hidden' && rawItems.value.length !== 0) return;

	window.fetch(fetchEndpoint.value, {})
		.then(res => res.json())
		.then((feed: Misskey.entities.FetchRssResponse) => {
			rawItems.value = feed.items;
			fetching.value = false;
		});
};
watch(fetchEndpoint, tick);
watch(() => widgetProps.refreshIntervalSec, () => {
	if (intervalClear.value) {
		intervalClear.value();
	}
	intervalClear.value = useInterval(tick, Math.max(10000, widgetProps.refreshIntervalSec * 1000), {
		immediate: true,
		afterMounted: true,
	});
}, { immediate: true });
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkResult = _resolveComponent("MkResult")

  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      "data-cy-mkw-rss": "",
      class: "mkw-rss"
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode("RSS")
      ]),
      func: _withCtx(({ buttonStyleClass }) => [
        _createElementVNode("button", {
          class: _normalizeClass(["_button", buttonStyleClass]),
          onClick: _cache[0] || (_cache[0] = (...args) => (configure && configure(...args)))
        }, [
          _hoisted_2
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { class: "ekmkgxbj" }, [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : ((!items.value || items.value.length === 0) && _unref(widgetProps).showHeader)
              ? (_openBlock(), _createBlock(_component_MkResult, {
                key: 1,
                type: "empty"
              }))
            : (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: _normalizeClass(_ctx.$style.feed)
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items.value, (item) => {
                return (_openBlock(), _createElementBlock("a", {
                  key: item.link,
                  class: _normalizeClass(_ctx.$style.item),
                  href: item.link,
                  rel: "nofollow noopener",
                  target: "_blank",
                  title: item.title
                }, _toDisplayString(item.title), 9 /* TEXT, PROPS */, ["href", "title"]))
              }), 128 /* KEYED_FRAGMENT */))
            ]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader"]))
}
}

})
