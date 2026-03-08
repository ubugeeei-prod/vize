import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import { computed, ref } from 'vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import XGridLocalComponent from '@/pages/admin/custom-emojis-manager.local.list.vue'
import XGridRemoteComponent from '@/pages/admin/custom-emojis-manager.remote.vue'
import XRegisterComponent from '@/pages/admin/custom-emojis-manager.register.vue'

type PageMode = 'local' | 'remote';

export default /*@__PURE__*/_defineComponent({
  __name: 'custom-emojis-manager2',
  setup(__props) {

const headerTab = ref<PageMode>('local');
const headerTabs = computed(() => [{
	key: 'local',
	title: i18n.ts.local,
}, {
	key: 'remote',
	title: i18n.ts.remote,
}, {
	key: 'register',
	title: i18n.ts._customEmojisManager._local.tabTitleRegister,
}]);
definePage(computed(() => ({
	title: i18n.ts.customEmojis,
	icon: 'ti ti-icons',
	needWideArea: true,
})));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      tabs: headerTabs.value,
      tab: headerTab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((headerTab).value = $event))
    }, {
      default: _withCtx(() => [
        (headerTab.value === 'local')
          ? (_openBlock(), _createBlock(XGridLocalComponent, {
            key: 0,
            class: _normalizeClass(_ctx.$style.local)
          }))
          : (headerTab.value === 'remote')
            ? (_openBlock(), _createBlock(XGridRemoteComponent, { key: 1 }))
          : (headerTab.value === 'register')
            ? (_openBlock(), _createBlock(XRegisterComponent, { key: 2 }))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["tabs", "tab"]))
}
}

})
