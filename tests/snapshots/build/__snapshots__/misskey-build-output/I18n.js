import { useSlots as _useSlots } from "vue";
import { openBlock as _openBlock, createBlock as _createBlock } from "vue";
import { computed, h } from "vue";
export default {
  __name: "I18n",
  props: {
    src: {
      type: null,
      required: true
    },
    tag: {
      type: String,
      required: false,
      default: "span"
    },
    textTag: {
      type: String,
      required: false
    }
  },
  setup(__props) {
    const props = __props;
    const slots = _useSlots();
    const parsed = computed(() => {
      let str = props.src;
      const value = [];
      for (;;) {
        const nextBracketOpen = str.indexOf("{");
        const nextBracketClose = str.indexOf("}");
        if (nextBracketOpen === -1) {
          value.push(str);
          break;
        } else {
          if (nextBracketOpen > 0) value.push(str.substring(0, nextBracketOpen));
          value.push({ arg: str.substring(nextBracketOpen + 1, nextBracketClose) });
        }
        str = str.substring(nextBracketClose + 1);
      }
      return value;
    });
    const render = () => {
      return h(props.tag, parsed.value.map((x) => typeof x === "string" ? props.textTag ? h(props.textTag, x) : x : slots[x.arg]()));
    };
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(render);
    };
  }
};
