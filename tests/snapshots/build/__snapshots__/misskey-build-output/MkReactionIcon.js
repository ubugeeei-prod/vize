import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent } from "vue";
import { defineAsyncComponent, useTemplateRef } from "vue";
import { useTooltip } from "@/composables/use-tooltip.js";
import * as os from "@/os.js";
export default {
  __name: "MkReactionIcon",
  props: {
    reaction: {
      type: String,
      required: true
    },
    noStyle: {
      type: Boolean,
      required: false
    },
    emojiUrl: {
      type: String,
      required: false
    },
    withTooltip: {
      type: Boolean,
      required: false
    }
  },
  setup(__props) {
    const props = __props;
    const elRef = useTemplateRef("elRef");
    if (props.withTooltip) {
      useTooltip(elRef, (showing) => {
        if (elRef.value == null) return;
        const { dispose } = os.popup(defineAsyncComponent(() => import("@/components/MkReactionTooltip.vue")), {
          showing,
          reaction: props.reaction.replace(/^:(\w+):$/, ":$1@.:"),
          anchorElement: elRef.value.$el
        }, { closed: () => dispose() });
      });
    }
    return (_ctx, _cache) => {
      const _component_MkCustomEmoji = _resolveComponent("MkCustomEmoji");
      const _component_MkEmoji = _resolveComponent("MkEmoji");
      return __props.reaction[0] === ":" ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
        key: 0,
        ref_key: "elRef",
        ref: elRef,
        name: __props.reaction,
        normal: true,
        noStyle: __props.noStyle,
        url: __props.emojiUrl,
        fallbackToImage: true
      }, null, 8, [
        "name",
        "normal",
        "noStyle",
        "url",
        "fallbackToImage"
      ])) : (_openBlock(), _createBlock(_component_MkEmoji, {
        key: 1,
        ref_key: "elRef",
        ref: elRef,
        emoji: __props.reaction,
        normal: true,
        noStyle: __props.noStyle
      }, null, 8, [
        "emoji",
        "normal",
        "noStyle"
      ]));
    };
  }
};
