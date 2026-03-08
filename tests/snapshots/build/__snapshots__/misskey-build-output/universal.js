import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"

import { defineAsyncComponent, provide, onMounted, computed, ref } from 'vue'
import { instanceName } from '@@/js/config.js'
import { isLink } from '@@/js/is-link.js'
import XCommon from './_common_/common.vue'
import type { PageMetadata } from '@/page.js'
import XMobileFooterMenu from '@/ui/_common_/mobile-footer-menu.vue'
import XPreferenceRestore from '@/ui/_common_/PreferenceRestore.vue'
import XReloadSuggestion from '@/ui/_common_/ReloadSuggestion.vue'
import XTitlebar from '@/ui/_common_/titlebar.vue'
import XSidebar from '@/ui/_common_/navbar.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'
import { provideMetadataReceiver, provideReactiveMetadata } from '@/page.js'
import { deviceKind } from '@/utility/device-kind.js'
import { miLocalStorage } from '@/local-storage.js'
import { mainRouter } from '@/router.js'
import { prefer } from '@/preferences.js'
import { shouldSuggestRestoreBackup } from '@/preferences/utility.js'
import { DI } from '@/di.js'
import { shouldSuggestReload } from '@/utility/reload-suggest.js'
const DESKTOP_THRESHOLD = 1100;
const MOBILE_THRESHOLD = 500;

export default /*@__PURE__*/_defineComponent({
  __name: 'universal',
  setup(__props) {

const XWidgets = defineAsyncComponent(() => import('./_common_/widgets.vue'));
const XStatusBars = defineAsyncComponent(() => import('@/ui/_common_/statusbars.vue'));
const XAnnouncements = defineAsyncComponent(() => import('@/ui/_common_/announcements.vue'));
const isRoot = computed(() => mainRouter.currentRoute.value.name === 'index');
// デスクトップでウィンドウを狭くしたときモバイルUIが表示されて欲しいことはあるので deviceKind === 'desktop' の判定は行わない
const showWidgetsSide = window.innerWidth >= DESKTOP_THRESHOLD;
const isMobile = ref(deviceKind === 'smartphone' || window.innerWidth <= MOBILE_THRESHOLD);
window.addEventListener('resize', () => {
	isMobile.value = deviceKind === 'smartphone' || window.innerWidth <= MOBILE_THRESHOLD;
});
const pageMetadata = ref<null | PageMetadata>(null);
const widgetsShowing = ref(false);
provide(DI.router, mainRouter);
provideMetadataReceiver((metadataGetter) => {
	const info = metadataGetter();
	pageMetadata.value = info;
	if (pageMetadata.value) {
		if (isRoot.value && pageMetadata.value.title === instanceName) {
			window.document.title = pageMetadata.value.title;
		} else {
			window.document.title = `${pageMetadata.value.title} | ${instanceName}`;
		}
	}
});
provideReactiveMetadata(pageMetadata);
const drawerMenuShowing = ref(false);
mainRouter.on('change', () => {
	drawerMenuShowing.value = false;
});
if (window.innerWidth > 1024) {
	const tempUI = miLocalStorage.getItem('ui_temp');
	if (tempUI) {
		miLocalStorage.setItem('ui', tempUI);
		miLocalStorage.removeItem('ui_temp');
		window.location.reload();
	}
}
function onContextmenu(ev: PointerEvent) {
	if (isLink(ev.target as HTMLElement)) return;
	if (['INPUT', 'TEXTAREA', 'IMG', 'VIDEO', 'CANVAS'].includes((ev.target as HTMLElement).tagName) || (ev.target as HTMLElement).attributes.getNamedItem('contenteditable') != null) return;
	if (window.getSelection()?.toString() !== '') return;
	const path = mainRouter.getCurrentFullPath();
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

return (_ctx: any,_cache: any) => {
  const _component_StackingRouterView = _resolveComponent("StackingRouterView")
  const _component_RouterView = _resolveComponent("RouterView")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, { '_forceShrinkSpacer': _unref(deviceKind) === 'smartphone' }])
    }, [ (_unref(prefer).r.showTitlebar.value) ? (_openBlock(), _createBlock(XTitlebar, {
          key: 0,
          style: "flex-shrink: 0;"
        })) : _createCommentVNode("v-if", true), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.nonTitlebarArea)
      }, [ (!isMobile.value) ? (_openBlock(), _createBlock(XSidebar, {
            key: 0,
            class: _normalizeClass(_ctx.$style.sidebar),
            showWidgetButton: !_unref(showWidgetsSide),
            onWidgetButtonClick: _cache[0] || (_cache[0] = ($event: any) => (widgetsShowing.value = true))
          }, null, 8 /* PROPS */, ["showWidgetButton"])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          class: _normalizeClass([_ctx.$style.contents, !isMobile.value && _unref(prefer).r.showTitlebar.value ? _ctx.$style.withSidebarAndTitlebar : null]),
          onContextmenu: _withModifiers(onContextmenu, ["stop"])
        }, [ _createElementVNode("div", null, [ (_unref(shouldSuggestReload)) ? (_openBlock(), _createBlock(XReloadSuggestion, { key: 0 })) : _createCommentVNode("v-if", true), (_unref(shouldSuggestRestoreBackup)) ? (_openBlock(), _createBlock(XPreferenceRestore, { key: 0 })) : _createCommentVNode("v-if", true), (_unref($i)) ? (_openBlock(), _createBlock(XAnnouncements, { key: 0 })) : _createCommentVNode("v-if", true), _createVNode(XStatusBars, {
              class: _normalizeClass(_ctx.$style.statusbars)
            }) ]), (_unref(prefer).s['experimental.stackingRouterView']) ? (_openBlock(), _createBlock(_component_StackingRouterView, {
              key: 0,
              class: _normalizeClass(_ctx.$style.content)
            })) : (_openBlock(), _createBlock(_component_RouterView, {
              key: 1,
              class: _normalizeClass(_ctx.$style.content)
            })), (isMobile.value) ? (_openBlock(), _createBlock(XMobileFooterMenu, {
              key: 0,
              ref: "navFooter",
              drawerMenuShowing: drawerMenuShowing.value,
              "onUpdate:drawerMenuShowing": _cache[1] || (_cache[1] = ($event: any) => ((drawerMenuShowing).value = $event)),
              widgetsShowing: widgetsShowing.value,
              "onUpdate:widgetsShowing": _cache[2] || (_cache[2] = ($event: any) => ((widgetsShowing).value = $event))
            }, null, 8 /* PROPS */, ["drawerMenuShowing", "widgetsShowing"])) : _createCommentVNode("v-if", true) ], 34 /* CLASS, NEED_HYDRATION */), (_unref(showWidgetsSide) && !pageMetadata.value?.needWideArea) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.widgets)
          }, [ _createVNode(XWidgets) ])) : _createCommentVNode("v-if", true) ]), _createVNode(XCommon, {
        drawerMenuShowing: drawerMenuShowing.value,
        "onUpdate:drawerMenuShowing": _cache[3] || (_cache[3] = ($event: any) => ((drawerMenuShowing).value = $event)),
        widgetsShowing: widgetsShowing.value,
        "onUpdate:widgetsShowing": _cache[4] || (_cache[4] = ($event: any) => ((widgetsShowing).value = $event))
      }, null, 8 /* PROPS */, ["drawerMenuShowing", "widgetsShowing"]) ], 2 /* CLASS */))
}
}

})
