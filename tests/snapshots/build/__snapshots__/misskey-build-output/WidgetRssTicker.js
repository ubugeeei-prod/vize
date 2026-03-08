import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rss" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
import { ref, watch, computed } from 'vue'
import * as Misskey from 'misskey-js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import MkMarqueeText from '@/components/MkMarqueeText.vue'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkContainer from '@/components/MkContainer.vue'
import { shuffle } from '@/utility/shuffle.js'
import { i18n } from '@/i18n.js'
import { url as base } from '@@/js/config.js'
import { useInterval } from '@@/js/use-interval.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'rssTicker';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetRssTicker',
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
	shuffle: {
		type: 'boolean',
		label: i18n.ts._widgetOptions._rssTicker.shuffle,
		default: true,
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
	duration: {
		type: 'range',
		label: i18n.ts._widgetOptions._rssTicker.duration,
		default: 70,
		step: 1,
		min: 5,
		max: 200,
	},
	reverse: {
		type: 'boolean',
		label: i18n.ts._widgetOptions._rssTicker.reverse,
		default: false,
	},
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: false,
	},
	transparent: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.transparent,
		default: false,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const rawItems = ref<Misskey.entities.FetchRssResponse['items']>([]);
const items = computed(() => {
	const newItems = rawItems.value.slice(0, widgetProps.maxEntries);
	if (widgetProps.shuffle) {
		shuffle(newItems);
	}
	return newItems;
});
const fetching = ref(true);
const fetchEndpoint = computed(() => {
	const url = new URL('/api/fetch-rss', base);
	url.searchParams.set('url', widgetProps.url);
	return url;
});
const intervalClear = ref<(() => void) | undefined>();
const key = ref(0);
const tick = () => {
	if (window.document.visibilityState === 'hidden' && rawItems.value.length !== 0) return;

	window.fetch(fetchEndpoint.value, {})
		.then(res => res.json())
		.then((feed: Misskey.entities.FetchRssResponse) => {
			rawItems.value = feed.items;
			fetching.value = false;
			key.value++;
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
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")

  return (_openBlock(), _createBlock(MkContainer, {
      naked: _unref(widgetProps).transparent,
      showHeader: _unref(widgetProps).showHeader,
      class: "mkw-rss-ticker"
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
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.feed)
        }, [
          (fetching.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.loading)
            }, [
              _createVNode(_component_MkEllipsis)
            ]))
            : (_openBlock(), _createElementBlock("div", { key: 1 }, [
              _createVNode(_Transition, {
                name: _ctx.$style.change,
                mode: "default",
                appear: ""
              }, {
                default: _withCtx(() => [
                  _createVNode(MkMarqueeText, {
                    key: key.value,
                    duration: _unref(widgetProps).duration,
                    reverse: _unref(widgetProps).reverse
                  }, {
                    default: _withCtx(() => [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items.value, (item) => {
                        return (_openBlock(), _createElementBlock("span", {
                          key: item.link,
                          class: _normalizeClass(_ctx.$style.item)
                        }, [
                          _createElementVNode("a", {
                            href: item.link,
                            rel: "nofollow noopener",
                            target: "_blank",
                            title: item.title
                          }, _toDisplayString(item.title), 9 /* TEXT, PROPS */, ["href", "title"]),
                          _createElementVNode("span", {
                            class: _normalizeClass(_ctx.$style.divider)
                          })
                        ]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["duration", "reverse"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["name"])
            ]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["naked", "showHeader"]))
}
}

})
