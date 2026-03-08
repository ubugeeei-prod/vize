import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("path", { d: "M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6 120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3 C122.9,97.6 130.6,101.9 134.4,103.2", fill: "currentColor", style: "transform-origin: 130px 106px;", class: "octo-arm" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("path", { d: "M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2 139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2 159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2 C183.1,58.6 187.2,61.8 190.9,65.4 C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2 216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9 152.7,124.9 L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z", fill: "currentColor", class: "octo-body" })
import { onMounted, provide, ref, computed } from 'vue'
import { instanceName } from '@@/js/config.js'
import XCommon from './_common_/common.vue'
import type { PageMetadata } from '@/page.js'
import * as os from '@/os.js'
import { instance } from '@/instance.js'
import { provideMetadataReceiver, provideReactiveMetadata } from '@/page.js'
import { i18n } from '@/i18n.js'
import MkVisitorDashboard from '@/components/MkVisitorDashboard.vue'
import { mainRouter } from '@/router.js'
import { DI } from '@/di.js'
import MkButton from '@/components/MkButton.vue'
const DESKTOP_THRESHOLD = 1100;

export default /*@__PURE__*/_defineComponent({
  __name: 'visitor',
  setup(__props) {

const isRoot = computed(() => mainRouter.currentRoute.value.name === 'index');
const pageMetadata = ref<null | PageMetadata>(null);
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
const isDesktop = ref(window.innerWidth >= DESKTOP_THRESHOLD);
const narrow = ref(window.innerWidth < 1280);
function goHome() {
	mainRouter.push('/');
}
onMounted(() => {
	if (!isDesktop.value) {
		window.addEventListener('resize', () => {
			if (window.innerWidth >= DESKTOP_THRESHOLD) isDesktop.value = true;
		}, { passive: true });
	}
});

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _component_RouterView = _resolveComponent("RouterView")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.root)
      }, [ (isRoot.value) ? (_openBlock(), _createElementBlock("a", {
            key: 0,
            href: "https://github.com/misskey-dev/misskey",
            target: "_blank",
            class: "github-corner",
            "aria-label": "View source on GitHub"
          }, [ _createElementVNode("svg", {
              width: "80",
              height: "80",
              viewBox: "0 0 250 250",
              style: "fill:var(--MI_THEME-panel); color:var(--MI_THEME-fg); position: fixed; z-index: 10; top: 0; border: 0; right: 0;",
              "aria-hidden": "true"
            }, [ _hoisted_1, _hoisted_2, _hoisted_3 ]) ])) : _createCommentVNode("v-if", true), (!narrow.value && !isRoot.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.side)
          }, [ _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.sideBanner),
              style: _normalizeStyle({ backgroundImage: _unref(instance).backgroundImageUrl ? `url(${ _unref(instance).backgroundImageUrl })` : 'none' })
            }, null, 4 /* STYLE */), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.sideDashboard)
            }, [ _createVNode(MkVisitorDashboard) ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.main)
        }, [ (narrow.value && !isRoot.value) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.header)
            }, [ _createElementVNode("img", {
                src: _unref(instance).iconUrl || '/favicon.ico',
                alt: "",
                class: _normalizeClass(_ctx.$style.headerIcon)
              }, null, 8 /* PROPS */, ["src"]), _createVNode(_component_MkA, {
                to: "/",
                class: _normalizeClass(_ctx.$style.headerTitle)
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(instanceName)), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkButton, {
                primary: "",
                rounded: "",
                class: _normalizeClass(_ctx.$style.headerButton),
                onClick: goHome
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.signup), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.content)
          }, [ _createVNode(_component_RouterView) ]) ]) ]), _createVNode(XCommon) ], 64 /* STABLE_FRAGMENT */))
}
}

})
