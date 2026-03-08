import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"

import { computed } from 'vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { antennasCache } from '@/cache.js'
import { useRouter } from '@/router.js'
import MkAntennaEditor from '@/components/MkAntennaEditor.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'create',
  setup(__props) {

const router = useRouter();
function onAntennaCreated() {
	antennasCache.delete();
	router.push('/my/antennas');
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.createAntenna,
	icon: 'ti ti-antenna',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createVNode(MkAntennaEditor, { onCreated: onAntennaCreated })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
