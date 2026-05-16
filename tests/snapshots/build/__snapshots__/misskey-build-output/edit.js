import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue";
import { ref, computed } from "vue";
import MkAntennaEditor from "@/components/MkAntennaEditor.vue";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import { antennasCache } from "@/cache.js";
import { useRouter } from "@/router.js";
export default {
  __name: "edit",
  props: { antennaId: {
    type: String,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const router = useRouter();
    const antenna = ref(null);
    function onAntennaUpdated() {
      antennasCache.delete();
      router.push("/my/antennas");
    }
    misskeyApi("antennas/show", { antennaId: props.antennaId }).then((antennaResponse) => {
      antenna.value = antennaResponse;
    });
    const headerActions = computed(() => antenna.value ? [{
      icon: "ti ti-timeline",
      text: i18n.ts.timeline,
      handler: () => {
        router.push("/timeline/antenna/:antennaId", { params: { antennaId: antenna.value.id } });
      }
    }] : []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.editAntenna,
      icon: "ti ti-antenna"
    }));
    return (_ctx, _cache) => {
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value
      }, {
        default: _withCtx(() => [antenna.value ? (_openBlock(), _createBlock(MkAntennaEditor, {
          key: 0,
          antenna: antenna.value,
          onUpdated: onAntennaUpdated
        }, null, 8, ["antenna"])) : _createCommentVNode("v-if", true)]),
        _: 2
      }, 1032, ["actions", "tabs"]);
    };
  }
};
