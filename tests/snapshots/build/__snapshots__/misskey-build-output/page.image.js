import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass } from "vue";
import { onMounted, ref } from "vue";
import MkMediaList from "@/components/MkMediaList.vue";
export default {
  __name: "page.image",
  props: {
    block: {
      type: null,
      required: true
    },
    page: {
      type: null,
      required: true
    }
  },
  setup(__props) {
    const props = __props;
    const image = ref(null);
    onMounted(() => {
      image.value = props.page.attachedFiles.find((x) => x.id === props.block.fileId) ?? null;
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [image.value ? (_openBlock(), _createBlock(MkMediaList, {
          key: 0,
          mediaList: [image.value],
          class: _normalizeClass(_ctx.$style.mediaList)
        }, null, 10, ["mediaList"])) : _createCommentVNode("v-if", true)],
        2
        /* CLASS */
      );
    };
  }
};
