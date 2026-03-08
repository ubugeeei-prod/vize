import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"

import { computed, ref, defineAsyncComponent } from 'vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkSwiper from '@/components/MkSwiper.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'drive.file',
  props: {
    fileId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const XFileInfo = defineAsyncComponent(() => import('./drive.file.info.vue'));
const XNotes = defineAsyncComponent(() => import('./drive.file.notes.vue'));
const tab = ref('info');
const headerActions = computed(() => []);
const headerTabs = computed(() => [{
	key: 'info',
	title: i18n.ts.info,
	icon: 'ti ti-info-circle',
}, {
	key: 'notes',
	title: i18n.ts._fileViewer.attachedNotes,
	icon: 'ti ti-pencil',
}]);
definePage(() => ({
	title: i18n.ts._fileViewer.title,
	icon: 'ti ti-file',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkPageHeader = _resolveComponent("MkPageHeader")
  const _component_MkStickyContainer = _resolveComponent("MkStickyContainer")

  return (_openBlock(), _createBlock(_component_MkStickyContainer, null, {
      header: _withCtx(() => [
        _createVNode(_component_MkPageHeader, {
          actions: headerActions.value,
          tabs: headerTabs.value,
          tab: tab.value,
          "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
        }, null, 8 /* PROPS */, ["actions", "tabs", "tab"])
      ]),
      default: _withCtx(() => [
        _createVNode(MkSwiper, {
          tabs: headerTabs.value,
          tab: tab.value,
          "onUpdate:tab": _cache[1] || (_cache[1] = ($event: any) => ((tab).value = $event))
        }, {
          default: _withCtx(() => [
            (tab.value === 'info')
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "_spacer",
                style: "--MI_SPACER-w: 800px;"
              }, [
                _createVNode(XFileInfo, { fileId: __props.fileId }, null, 8 /* PROPS */, ["fileId"])
              ]))
              : (tab.value === 'notes')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  class: "_spacer",
                  style: "--MI_SPACER-w: 800px;"
                }, [
                  _createVNode(XNotes, { fileId: __props.fileId }, null, 8 /* PROPS */, ["fileId"])
                ]))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["tabs", "tab"])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
