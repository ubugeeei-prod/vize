import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import { computed, watch, ref, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import MkStreamingNotesTimeline from '@/components/MkStreamingNotesTimeline.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { useRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'antenna-timeline',
  props: {
    antennaId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const antenna = ref<Misskey.entities.Antenna | null>(null);
const tlEl = useTemplateRef('tlEl');
function settings() {
	router.push('/my/antennas/:antennaId', {
		params: {
			antennaId: props.antennaId,
		},
	});
}
watch(() => props.antennaId, async () => {
	antenna.value = await misskeyApi('antennas/show', {
		antennaId: props.antennaId,
	});
}, { immediate: true });
const headerActions = computed(() => antenna.value ? [{
	icon: 'ti ti-settings',
	text: i18n.ts.settings,
	handler: settings,
}] : []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: antenna.value ? antenna.value.name : i18n.ts.antennas,
	icon: 'ti ti-antenna',
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
              ref_key: "tlEl", ref: tlEl,
              key: __props.antennaId,
              src: "antenna",
              antenna: __props.antennaId,
              sound: true
            }, null, 8 /* PROPS */, ["antenna", "sound"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
