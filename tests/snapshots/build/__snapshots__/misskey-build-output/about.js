import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, defineAsyncComponent, ref, watch } from 'vue'
import { instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { claimAchievement } from '@/utility/achievements.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'about',
  props: {
    initialTab: { type: String, required: false, default: 'overview' }
  },
  setup(__props: any) {

const props = __props
const XOverview = defineAsyncComponent(() => import('@/pages/about.overview.vue'));
const XEmojis = defineAsyncComponent(() => import('@/pages/about.emojis.vue'));
const XFederation = defineAsyncComponent(() => import('@/pages/about.federation.vue'));
const MkInstanceStats = defineAsyncComponent(() => import('@/components/MkInstanceStats.vue'));
const tab = ref(props.initialTab);
watch(tab, () => {
	if (tab.value === 'charts') {
		claimAchievement('viewInstanceChart');
	}
});
const headerActions = computed(() => []);
const headerTabs = computed(() => [{
	key: 'overview',
	title: i18n.ts.overview,
}, {
	key: 'emojis',
	title: i18n.ts.customEmojis,
	icon: 'ti ti-icons',
}, ...(instance.federation !== 'none' ? [{
	key: 'federation',
	title: i18n.ts.federation,
	icon: 'ti ti-whirl',
}] : []), {
	key: 'charts',
	title: i18n.ts.charts,
	icon: 'ti ti-chart-line',
}]);
definePage(() => ({
	title: i18n.ts.instanceInfo,
	icon: 'ti ti-info-circle',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      swipable: true,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        (tab.value === 'overview')
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_spacer",
            style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 20px;"
          }, [
            _createVNode(XOverview)
          ]))
          : (tab.value === 'emojis')
            ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "_spacer",
              style: "--MI_SPACER-w: 1000px; --MI_SPACER-min: 20px;"
            }, [
              _createVNode(XEmojis)
            ]))
          : (_unref(instance).federation !== 'none' && tab.value === 'federation')
            ? (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "_spacer",
              style: "--MI_SPACER-w: 1000px; --MI_SPACER-min: 20px;"
            }, [
              _createVNode(XFederation)
            ]))
          : (tab.value === 'charts')
            ? (_openBlock(), _createElementBlock("div", {
              key: 3,
              class: "_spacer",
              style: "--MI_SPACER-w: 1000px; --MI_SPACER-min: 20px;"
            }, [
              _createVNode(MkInstanceStats)
            ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "swipable", "tab"]))
}
}

})
