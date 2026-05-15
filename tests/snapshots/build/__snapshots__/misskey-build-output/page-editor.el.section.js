import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-note" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-pencil" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
import { defineAsyncComponent, onMounted, watch, ref } from "vue";
import XContainer from "../page-editor.container.vue";
import { genId } from "@/utility/id.js";
import * as os from "@/os.js";
import { i18n } from "@/i18n.js";
import { deepClone } from "@/utility/clone.js";
import MkButton from "@/components/MkButton.vue";
import { getPageBlockList } from "@/pages/page-editor/common.js";
export default {
  __name: "page-editor.el.section",
  props: { modelValue: {
    type: null,
    required: true
  } },
  emits: [
    "update:modelValue",
    "section",
    "remove"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const XBlocks = defineAsyncComponent(() => import("../page-editor.blocks.vue"));
    const children = ref(deepClone(props.modelValue.children ?? []));
    watch(children, () => {
      emit("update:modelValue", {
        ...props.modelValue,
        children: children.value
      });
    }, { deep: true });
    async function rename() {
      const { canceled, result: title } = await os.inputText({
        title: i18n.ts._pages.enterSectionTitle,
        default: props.modelValue.title
      });
      if (canceled || title == null) return;
      emit("update:modelValue", {
        ...props.modelValue,
        title
      });
    }
    async function add() {
      const { canceled, result: type } = await os.select({
        title: i18n.ts._pages.chooseBlock,
        items: getPageBlockList()
      });
      if (canceled || type == null) return;
      const id = genId();
      // TODO: page-editor.vueのと共通化
      if (type === "text") {
        children.value.push({
          id,
          type,
          text: ""
        });
      } else if (type === "section") {
        children.value.push({
          id,
          type,
          title: "",
          children: []
        });
      } else if (type === "image") {
        children.value.push({
          id,
          type,
          fileId: null
        });
      } else if (type === "note") {
        children.value.push({
          id,
          type,
          detailed: false,
          note: null
        });
      }
    }
    onMounted(() => {
      if (props.modelValue.title == null) {
        rename();
      }
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(XContainer, {
        draggable: true,
        onRemove: _cache[0] || (_cache[0] = () => emit("remove"))
      }, {
        header: _withCtx(() => [
          _hoisted_1,
          _createTextVNode(" "),
          _createTextVNode(
            _toDisplayString(props.modelValue.title),
            1
            /* TEXT */
          )
        ]),
        func: _withCtx(() => [_createElementVNode("button", {
          class: "_button",
          onClick: _cache[1] || (_cache[1] = ($event) => rename())
        }, [_hoisted_2])]),
        default: _withCtx(() => [_createElementVNode("section", { class: "ilrvjyvi" }, [_createVNode(XBlocks, {
          class: "children",
          modelValue: children.value,
          "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => children.value = $event)
        }, null, 8, ["modelValue"]), _createVNode(MkButton, {
          rounded: "",
          class: "add",
          onClick: _cache[3] || (_cache[3] = ($event) => add())
        }, {
          default: _withCtx(() => [_hoisted_3]),
          _: 1
        })])]),
        _: 1
      }, 8, ["draggable"]);
    };
  }
};
