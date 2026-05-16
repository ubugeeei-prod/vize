import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { computed } from "vue";
import { concat } from "@/utility/array.js";
import { i18n } from "@/i18n.js";
import MkButton from "@/components/MkButton.vue";
export default {
  __name: "MkCwButton",
  props: {
    modelValue: {
      type: Boolean,
      required: true
    },
    text: {
      type: [String, null],
      required: true
    },
    renote: {
      type: null,
      required: false
    },
    files: {
      type: Array,
      required: false
    },
    poll: {
      type: null,
      required: false
    }
  },
  emits: ["update:modelValue"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const label = computed(() => {
      return concat([
        props.text ? [i18n.tsx._cw.chars({ count: props.text.length })] : [],
        props.renote ? [i18n.ts.quote] : [],
        props.files && props.files.length !== 0 ? [i18n.tsx._cw.files({ count: props.files.length })] : [],
        props.poll != null ? [i18n.ts.poll] : []
      ]).join(" / ");
    });
    function toggle() {
      emit("update:modelValue", !props.modelValue);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(
        MkButton,
        {
          rounded: "",
          full: "",
          small: "",
          onClick: toggle
        },
        {
          default: _withCtx(() => [_createElementVNode(
            "b",
            null,
            _toDisplayString(__props.modelValue ? _unref(i18n).ts._cw.hide : _unref(i18n).ts._cw.show),
            1
            /* TEXT */
          ), !__props.modelValue ? (_openBlock(), _createElementBlock(
            "span",
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.label)
            },
            _toDisplayString(label.value),
            3
            /* TEXT, CLASS */
          )) : _createCommentVNode("v-if", true)]),
          _: 2
        },
        1024
        /* DYNAMIC_SLOTS */
      );
    };
  }
};
