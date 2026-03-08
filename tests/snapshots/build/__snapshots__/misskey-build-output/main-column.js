import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"

import { provide, useTemplateRef, ref } from 'vue'
import { isLink } from '@@/js/is-link.js'
import XColumn from './column.vue'
import type { Column } from '@/deck.js'
import type { PageMetadata } from '@/page.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { provideMetadataReceiver, provideReactiveMetadata } from '@/page.js'
import { mainRouter } from '@/router.js'
import { prefer } from '@/preferences.js'
import { DI } from '@/di.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'main-column',
  props: {
    column: { type: null, required: true },
    isStacked: { type: Boolean, required: true }
  },
  setup(__props: any) {

const pageMetadata = ref<null | PageMetadata>(null);
const rootEl = useTemplateRef('rootEl');
provide(DI.router, mainRouter);
provideMetadataReceiver((metadataGetter) => {
	const info = metadataGetter();
	pageMetadata.value = info;
});
provideReactiveMetadata(pageMetadata);
/*
function back() {
	history.back();
}
*/
function onContextmenu(ev: PointerEvent) {
	if (!ev.target) return;
	if (isLink(ev.target as HTMLElement)) return;
	if (['INPUT', 'TEXTAREA', 'IMG', 'VIDEO', 'CANVAS'].includes((ev.target as HTMLElement).tagName) || (ev.target as HTMLElement).attributes.getNamedItem('contenteditable') != null) return;
	if (window.getSelection()?.toString() !== '') return;
	const path = mainRouter.currentRoute.value.path;
	os.contextMenu([{
		type: 'label',
		text: path,
	}, {
		icon: 'ti ti-window-maximize',
		text: i18n.ts.openInWindow,
		action: () => {
			os.pageWindow(path);
		},
	}], ev);
}
function onHeaderClick() {
	if (!rootEl.value) return;
	const scrollEl = rootEl.value.querySelector<HTMLElement>('._pageScrollable,._pageScrollableReversed');
	if (scrollEl) {
		scrollEl.scrollTo({
			top: 0,
			behavior: 'smooth',
		});
	}
}

return (_ctx: any,_cache: any) => {
  const _component_StackingRouterView = _resolveComponent("StackingRouterView")
  const _component_RouterView = _resolveComponent("RouterView")

  return (_unref(prefer).s['deck.alwaysShowMainColumn'] || _unref(mainRouter).currentRoute.value.name !== 'index')
      ? (_openBlock(), _createBlock(XColumn, {
        key: 0,
        column: __props.column,
        isStacked: __props.isStacked,
        handleScrollToTop: false,
        onHeaderClick: onHeaderClick
      }, {
        header: _withCtx(() => [
          (pageMetadata.value)
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _createElementVNode("i", {
                class: _normalizeClass(pageMetadata.value.icon)
              }, null, 2 /* CLASS */),
              _createTextVNode("\n\t\t\t"),
              _toDisplayString(pageMetadata.value.title)
            ], 64 /* STABLE_FRAGMENT */))
            : _createCommentVNode("v-if", true)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", {
            ref_key: "rootEl", ref: rootEl,
            style: "height: 100%;"
          }, [
            (_unref(prefer).s['experimental.stackingRouterView'])
              ? (_openBlock(), _createBlock(_component_StackingRouterView, {
                key: 0,
                onContextmenu: _withModifiers(onContextmenu, ["stop"])
              }))
              : (_openBlock(), _createBlock(_component_RouterView, {
                key: 1,
                onContextmenu: _withModifiers(onContextmenu, ["stop"])
              }))
          ], 512 /* NEED_PATCH */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["column", "isStacked", "handleScrollToTop"]))
      : _createCommentVNode("v-if", true)
}
}

})
