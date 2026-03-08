import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass } from "vue"

import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkMediaList from '@/components/MkMediaList.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'page.image',
  props: {
    block: { type: null, required: true },
    page: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const image = ref<Misskey.entities.DriveFile | null>(null);
onMounted(() => {
	image.value = props.page.attachedFiles.find(x => x.id === props.block.fileId) ?? null;
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ (image.value) ? (_openBlock(), _createBlock(MkMediaList, {
          key: 0,
          mediaList: [image.value],
          class: _normalizeClass(_ctx.$style.mediaList)
        }, null, 8 /* PROPS */, ["mediaList"])) : _createCommentVNode("v-if", true) ]))
}
}

})
