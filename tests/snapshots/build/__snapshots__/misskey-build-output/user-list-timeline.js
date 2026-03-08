import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import { computed, watch, ref, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import MkStreamingNotesTimeline from '@/components/MkStreamingNotesTimeline.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { useRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'user-list-timeline',
  props: {
    listId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const list = ref<Misskey.entities.UserList | null>(null);
watch(() => props.listId, async () => {
	list.value = await misskeyApi('users/lists/show', {
		listId: props.listId,
	});
}, { immediate: true });
function settings() {
	router.push('/my/lists/:listId', {
		params: {
			listId: props.listId,
		}
	});
}
const headerActions = computed(() => list.value ? [{
	icon: 'ti ti-settings',
	text: i18n.ts.settings,
	handler: settings,
}] : []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: list.value ? list.value.name : i18n.ts.lists,
	icon: 'ti ti-list',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.tl)
          }, [
            _createVNode(MkStreamingNotesTimeline, {
              ref: "tlEl",
              key: __props.listId,
              src: "list",
              list: __props.listId,
              sound: true
            }, null, 8 /* PROPS */, ["list", "sound"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
