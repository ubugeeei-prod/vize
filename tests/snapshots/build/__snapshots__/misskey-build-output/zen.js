import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import { computed, provide, ref } from 'vue'
import { instanceName, ui } from '@@/js/config.js'
import XCommon from './_common_/common.vue'
import type { PageMetadata } from '@/page.js'
import { provideMetadataReceiver, provideReactiveMetadata } from '@/page.js'
import { i18n } from '@/i18n.js'
import { mainRouter } from '@/router.js'
import { DI } from '@/di.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'zen',
  setup(__props) {

const isRoot = computed(() => mainRouter.currentRoute.value.name === 'index');
const pageMetadata = ref<null | PageMetadata>(null);
const showDeckNav = !(new URLSearchParams(window.location.search)).has('zen') && ui === 'deck';
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
function goToDeck() {
	window.location.href = '/';
}

return (_ctx: any,_cache: any) => {
  const _component_RouterView = _resolveComponent("RouterView")

  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.contents)
      }, [ (_unref(showDeckNav)) ? (_openBlock(), _createElementBlock("button", {
            key: 0,
            class: _normalizeClass(["_buttonPrimary", _ctx.$style.deckNav]),
            onClick: goToDeck
          }, _toDisplayString(_unref(i18n).ts.goToDeck), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("div", { style: "flex: 1; min-height: 0;" }, [ _createVNode(_component_RouterView) ]) ]), _createVNode(XCommon) ]))
}
}

})
