import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue";
import { computed, watch, ref } from "vue";
import MkStreamingNotesTimeline from "@/components/MkStreamingNotesTimeline.vue";
import { misskeyApi } from "@/utility/misskey-api.js";
import { definePage } from "@/page.js";
import { i18n } from "@/i18n.js";
import { useRouter } from "@/router.js";
export default {
  __name: "user-list-timeline",
  props: { listId: {
    type: String,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const router = useRouter();
    const list = ref(null);
    watch(() => props.listId, async () => {
      list.value = await misskeyApi("users/lists/show", { listId: props.listId });
    }, { immediate: true });
    function settings() {
      router.push("/my/lists/:listId", { params: { listId: props.listId } });
    }
    const headerActions = computed(() => list.value ? [{
      icon: "ti ti-settings",
      text: i18n.ts.settings,
      handler: settings
    }] : []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: list.value ? list.value.name : i18n.ts.lists,
      icon: "ti ti-list"
    }));
    return (_ctx, _cache) => {
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value
      }, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.tl) },
          [_createVNode(MkStreamingNotesTimeline, {
            ref: "tlEl",
            key: __props.listId,
            src: "list",
            list: __props.listId,
            sound: true
          }, null, 8, ["list", "sound"])],
          2
          /* CLASS */
        )])]),
        _: 1
      }, 8, ["actions", "tabs"]);
    };
  }
};
