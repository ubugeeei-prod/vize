import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass } from "vue"

import { computed, provide, ref } from 'vue'
import { instanceName } from '@@/js/config.js'
import XCommon from './_common_/common.vue'
import type { PageMetadata } from '@/page.js'
import { provideMetadataReceiver, provideReactiveMetadata } from '@/page.js'
import { mainRouter } from '@/router.js'
import { DI } from '@/di.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'minimum',
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

return (_ctx: any,_cache: any) => {
  const _component_RouterView = _resolveComponent("RouterView")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createVNode(_component_RouterView), _createVNode(XCommon) ]))
}
}

})
