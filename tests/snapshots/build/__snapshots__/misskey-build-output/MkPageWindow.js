import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, onMounted, onUnmounted, provide, ref, useTemplateRef } from 'vue'
import { url } from '@@/js/config.js'
import type { PageMetadata } from '@/page.js'
import RouterView from '@/components/global/RouterView.vue'
import MkWindow from '@/components/MkWindow.vue'
import { popout as _popout } from '@/utility/popout.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { i18n } from '@/i18n.js'
import { provideMetadataReceiver, provideReactiveMetadata } from '@/page.js'
import { openingWindowsCount } from '@/os.js'
import { claimAchievement } from '@/utility/achievements.js'
import { createRouter, mainRouter } from '@/router.js'
import { analytics } from '@/analytics.js'
import { DI } from '@/di.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPageWindow',
  props: {
    initialPath: { type: String, required: true }
  },
  emits: ["closed"],
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const windowRouter = createRouter(props.initialPath);
const pageMetadata = ref<null | PageMetadata>(null);
const windowEl = useTemplateRef('windowEl');
const _history_ = ref<{ path: string; }[]>([{
	path: windowRouter.getCurrentFullPath(),
}]);
const buttonsLeft = computed(() => {
	return _history_.value.length > 1 ? [{
		icon: 'ti ti-arrow-left',
		title: i18n.ts.goBack,
		onClick: back,
	}] : [];
});
const buttonsRight = computed(() => {
	const buttons = [{
		icon: 'ti ti-reload',
		title: i18n.ts.reload,
		onClick: reload,
	}, {
		icon: 'ti ti-player-eject',
		title: i18n.ts.showInPage,
		onClick: expand,
	}];

	return buttons;
});
const reloadCount = ref(0);
function getSearchMarker(path: string) {
	const hash = path.split('#')[1];
	if (hash == null) return null;
	return hash;
}
const searchMarkerId = ref<string | null>(getSearchMarker(props.initialPath));
windowRouter.addListener('push', ctx => {
	_history_.value.push({ path: ctx.fullPath });
});
windowRouter.addListener('replace', ctx => {
	_history_.value.pop();
	_history_.value.push({ path: ctx.fullPath });
});
windowRouter.addListener('change', ctx => {
	if (_DEV_) console.log('windowRouter: change', ctx.fullPath);
	searchMarkerId.value = getSearchMarker(ctx.fullPath);
	analytics.page({
		path: ctx.fullPath,
		title: ctx.fullPath,
	});
});
windowRouter.init();
provide(DI.router, windowRouter);
provide(DI.inAppSearchMarkerId, searchMarkerId);
provideMetadataReceiver((metadataGetter) => {
	const info = metadataGetter();
	pageMetadata.value = info;
});
provideReactiveMetadata(pageMetadata);
provide('shouldOmitHeaderTitle', true);
provide('shouldHeaderThin', true);
const contextmenu = computed(() => ([{
	icon: 'ti ti-player-eject',
	text: i18n.ts.showInPage,
	action: expand,
}, {
	icon: 'ti ti-window-maximize',
	text: i18n.ts.popout,
	action: popout,
}, {
	icon: 'ti ti-external-link',
	text: i18n.ts.openInNewTab,
	action: () => {
		window.open(url + windowRouter.getCurrentFullPath(), '_blank', 'noopener');
		windowEl.value?.close();
	},
}, {
	icon: 'ti ti-link',
	text: i18n.ts.copyLink,
	action: () => {
		copyToClipboard(url + windowRouter.getCurrentFullPath());
	},
}]));
function back() {
	_history_.value.pop();
	windowRouter.replaceByPath(_history_.value.at(-1)!.path);
}
function reload() {
	reloadCount.value++;
}
function close() {
	windowEl.value?.close();
}
function expand() {
	mainRouter.pushByPath(windowRouter.getCurrentFullPath(), 'forcePage');
	windowEl.value?.close();
}
function popout() {
	_popout(windowRouter.getCurrentFullPath(), windowEl.value?.$el);
	windowEl.value?.close();
}
onMounted(() => {
	analytics.page({
		path: props.initialPath,
		title: props.initialPath,
	});
	openingWindowsCount.value++;
	if (openingWindowsCount.value >= 3) {
		claimAchievement('open3windows');
	}
});
onUnmounted(() => {
	openingWindowsCount.value--;
});
__expose({
	close,
})

return (_ctx: any,_cache: any) => {
  const _component_StackingRouterView = _resolveComponent("StackingRouterView")

  return (_openBlock(), _createBlock(MkWindow, {
      ref_key: "windowEl", ref: windowEl,
      initialWidth: 500,
      initialHeight: 500,
      canResize: true,
      closeButton: true,
      buttonsLeft: buttonsLeft.value,
      buttonsRight: buttonsRight.value,
      contextmenu: contextmenu.value,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        (pageMetadata.value)
          ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
            (pageMetadata.value.icon)
              ? (_openBlock(), _createElementBlock("i", {
                key: 0,
                class: _normalizeClass(pageMetadata.value.icon),
                style: "margin-right: 0.5em;"
              }))
              : _createCommentVNode("v-if", true),
            _createElementVNode("span", null, _toDisplayString(pageMetadata.value.title), 1 /* TEXT */)
          ], 64 /* STABLE_FRAGMENT */))
          : _createCommentVNode("v-if", true)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(["_forceShrinkSpacer", _ctx.$style.root])
        }, [
          (_unref(prefer).s['experimental.stackingRouterView'])
            ? (_openBlock(), _createBlock(_component_StackingRouterView, {
              key: reloadCount.value.toString() + ':stacking',
              router: _unref(windowRouter)
            }, null, 8 /* PROPS */, ["router"]))
            : (_openBlock(), _createBlock(RouterView, {
              key: reloadCount.value.toString() + ':non-stacking',
              router: _unref(windowRouter)
            }, null, 8 /* PROPS */, ["router"]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["initialWidth", "initialHeight", "canResize", "closeButton", "buttonsLeft", "buttonsRight", "contextmenu"]))
}
}

})
