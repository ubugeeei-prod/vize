import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue";
import { computed, ref } from "vue";
import XFeatured from "./explore.featured.vue";
import XUsers from "./explore.users.vue";
import XRoles from "./explore.roles.vue";
import { definePage } from "@/page.js";
import { i18n } from "@/i18n.js";
export default {
  __name: "explore",
  props: { initialTab: {
    type: String,
    required: false,
    default: "featured"
  } },
  setup(__props) {
    const props = __props;
    const tab = ref(props.initialTab);
    const headerActions = computed(() => []);
    const headerTabs = computed(() => [
      {
        key: "featured",
        icon: "ti ti-bolt",
        title: i18n.ts.featured
      },
      {
        key: "users",
        icon: "ti ti-users",
        title: i18n.ts.users
      },
      {
        key: "roles",
        icon: "ti ti-badges",
        title: i18n.ts.roles
      }
    ]);
    definePage(() => ({
      title: i18n.ts.explore,
      icon: "ti ti-hash"
    }));
    return (_ctx, _cache) => {
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value,
        swipable: true,
        tab: tab.value,
        "onUpdate:tab": _cache[0] || (_cache[0] = ($event) => tab.value = $event)
      }, {
        default: _withCtx(() => [tab.value === "featured" ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createVNode(XFeatured)])) : tab.value === "users" ? (_openBlock(), _createElementBlock("div", { key: 1 }, [_createVNode(XUsers)])) : tab.value === "roles" ? (_openBlock(), _createElementBlock("div", { key: 2 }, [_createVNode(XRoles)])) : _createCommentVNode("v-if", true)]),
        _: 2
      }, 1032, [
        "actions",
        "tabs",
        "swipable",
        "tab"
      ]);
    };
  }
};
