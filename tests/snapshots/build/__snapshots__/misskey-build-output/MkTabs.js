import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"

import { nextTick, onMounted, onUnmounted, useTemplateRef, watch } from 'vue'
import { prefer } from '@/preferences.js'
import { genId } from '@/utility/id.js'

export type Tab<K = string> = {
	key: K;
	onClick?: (ev: PointerEvent) => void;
	iconOnly?: boolean;
	title: string;
	icon?: string;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTabs',
  props: {
    tabs: { type: Array, required: false, default: () => ([] as T[]) },
    centered: { type: Boolean, required: false },
    tabHighlightUpper: { type: Boolean, required: false },
    "tab": {}
  },
  emits: ["tabClick", "update:tab"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const tab = _useModel(__props, "tab")
const cssAnchorSupported = CSS.supports('position-anchor', '--anchor-name');
const tabAnchorName = `--${genId()}-currentTab`;
const tabHighlightEl = useTemplateRef('tabHighlightEl');
const tabRefs: Record<string, HTMLElement | null> = {};
function getTabStyle(t: Tab): Record<string, string> {
	if (!cssAnchorSupported) return {};
	if (t.key === tab.value) {
		return {
			anchorName: tabAnchorName,
		};
	} else {
		return {};
	}
}
function onTabMousedown(selectedTab: Tab, ev: MouseEvent): void {
	// ユーザビリティの観点からmousedown時にはonClickは呼ばない
	if (selectedTab.key) {
		tab.value = selectedTab.key;
	}
}
function onTabClick(t: Tab, ev: PointerEvent): void {
	emit('tabClick', t.key);
	if (t.onClick) {
		ev.preventDefault();
		ev.stopPropagation();
		t.onClick(ev);
	}
	if (t.key) {
		tab.value = t.key;
	}
}
function renderTab() {
	if (cssAnchorSupported) return;
	const tabEl = tab.value ? tabRefs[tab.value] : undefined;
	if (tabEl && tabHighlightEl.value && tabHighlightEl.value.parentElement) {
		// offsetWidth や offsetLeft は少数を丸めてしまうため getBoundingClientRect を使う必要がある
		// https://developer.mozilla.org/ja/docs/Web/API/HTMLElement/offsetWidth#%E5%80%A4
		const parentRect = tabHighlightEl.value.parentElement.getBoundingClientRect();
		const rect = tabEl.getBoundingClientRect();
		tabHighlightEl.value.style.width = rect.width + 'px';
		tabHighlightEl.value.style.left = (rect.left - parentRect.left + tabHighlightEl.value.parentElement.scrollLeft) + 'px';
	}
}
let entering = false;
async function enter(el: Element) {
	if (!(el instanceof HTMLElement)) return;
	entering = true;
	const elementWidth = el.getBoundingClientRect().width;
	el.style.width = '0';
	el.style.paddingLeft = '0';
	el.offsetWidth; // reflow
	el.style.width = `${elementWidth}px`;
	el.style.paddingLeft = '';
	nextTick(() => {
		entering = false;
	});
	window.setTimeout(renderTab, 170);
}
function afterEnter(el: Element) {
	if (!(el instanceof HTMLElement)) return;
	// element.style.width = '';
}
async function leave(el: Element) {
	if (!(el instanceof HTMLElement)) return;
	const elementWidth = el.getBoundingClientRect().width;
	el.style.width = `${elementWidth}px`;
	el.style.paddingLeft = '';
	el.offsetWidth; // reflow
	el.style.width = '0';
	el.style.paddingLeft = '0';
}
function afterLeave(el: Element) {
	if (!(el instanceof HTMLElement)) return;
	el.style.width = '';
}
onMounted(() => {
	if (!cssAnchorSupported) {
		watch([tab, () => props.tabs], () => {
			nextTick(() => {
				if (entering) return;
				renderTab();
			});
		}, { immediate: true });
	}
});
onUnmounted(() => {
});

return (_ctx: any,_cache: any) => {
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.tabs, { [_ctx.$style.centered]: props.centered }]),
      style: _normalizeStyle({ '--tabAnchorName': _unref(tabAnchorName) })
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.tabsInner)
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.tabs, (t) => {
          return (_openBlock(), _createElementBlock("button", { ref: (el) => tabRefs[t.key] = el, class: _normalizeClass(["_button", [_ctx.$style.tab, {
  				[_ctx.$style.active]: t.key != null && t.key === tab.value,
  				[_ctx.$style.animate]: _unref(prefer).s.animation,
  			}]]), style: _normalizeStyle(getTabStyle(t)), onMousedown: _cache[0] || (_cache[0] = (ev) => onTabMousedown(t, ev)), onClick: _cache[1] || (_cache[1] = (ev) => onTabClick(t, ev)) }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.tabInner)
            }, [
              (t.icon)
                ? (_openBlock(), _createElementBlock("i", {
                  key: 0,
                  class: _normalizeClass([_ctx.$style.tabIcon, t.icon])
                }))
                : _createCommentVNode("v-if", true),
              (!t.iconOnly || (!_unref(prefer).s.animation && t.key === tab.value))
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.tabTitle)
                }, _toDisplayString(t.title), 1 /* TEXT */))
                : (_openBlock(), _createBlock(_Transition, {
                  key: 1,
                  mode: "in-out",
                  onEnter: enter,
                  onAfterEnter: afterEnter,
                  onLeave: leave,
                  onAfterLeave: afterLeave
                }, {
                  default: _withCtx(() => [
                    _withDirectives(_createElementVNode("div", {
                      class: _normalizeClass([_ctx.$style.tabTitle, _ctx.$style.animate])
                    }, _toDisplayString(t.title), 1 /* TEXT */), [
                      [_vShow, t.key === tab.value]
                    ])
                  ]),
                  _: 2 /* DYNAMIC */
                }))
            ])
          ], 550 /* CLASS, STYLE, NEED_HYDRATION, NEED_PATCH */))
        }), 256 /* UNKEYED_FRAGMENT */)) ]), _createElementVNode("div", {
        ref_key: "tabHighlightEl", ref: tabHighlightEl,
        class: _normalizeClass([_ctx.$style.tabHighlight, { [_ctx.$style.animate]: _unref(prefer).s.animation, [_ctx.$style.tabHighlightUpper]: __props.tabHighlightUpper }])
      }, null, 2 /* CLASS */) ], 6 /* CLASS, STYLE */))
}
}

})
