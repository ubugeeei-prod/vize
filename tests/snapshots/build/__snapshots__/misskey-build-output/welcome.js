import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode } from "vue";
import { computed, ref } from "vue";
import { instanceName } from "@@/js/config.js";
import XSetup from "./welcome.setup.vue";
import XEntranceClassic from "./welcome.entrance.classic.vue";
import XEntranceSimple from "./welcome.entrance.simple.vue";
import { definePage } from "@/page.js";
import { fetchInstance } from "@/instance.js";
export default {
  __name: "welcome",
  setup(__props) {
    const instance = ref(null);
    fetchInstance(true).then((res) => {
      instance.value = res;
    });
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: instanceName,
      icon: null
    }));
    return (_ctx, _cache) => {
      return instance.value ? (_openBlock(), _createElementBlock("div", { key: 0 }, [instance.value.requireSetup ? (_openBlock(), _createBlock(XSetup, { key: 0 })) : (instance.value.clientOptions.entrancePageStyle ?? "classic") === "classic" ? (_openBlock(), _createBlock(XEntranceClassic, { key: 1 })) : (_openBlock(), _createBlock(XEntranceSimple, { key: 2 }))])) : _createCommentVNode("v-if", true);
    };
  }
};
