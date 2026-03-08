import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "title" }
import * as Misskey from 'misskey-js'
import { computed, ref } from 'vue'
import MkImgWithBlurhash from '@/components/MkImgWithBlurhash.vue'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkGalleryPostPreview',
  props: {
    post: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const hover = ref(false);
const safe = computed(() => prefer.s.nsfw === 'ignore' || prefer.s.nsfw === 'respect' && !props.post.isSensitive);
const show = computed(() => safe.value || hover.value);
function enterHover(): void {
	hover.value = true;
}
function leaveHover(): void {
	hover.value = false;
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createBlock(_component_MkA, {
      to: `/gallery/${__props.post.id}`,
      class: "ttasepnz _panel",
      tabindex: "-1",
      onPointerenter: enterHover,
      onPointerleave: leaveHover
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "thumbnail" }, [
          _createVNode(_Transition, null, {
            default: _withCtx(() => [
              _createVNode(MkImgWithBlurhash, {
                class: "img layered",
                transition: safe.value ? null : {
  					duration: 500,
  					leaveActiveClass: _ctx.$style.transition_toggle_leaveActive,
  					leaveToClass: _ctx.$style.transition_toggle_leaveTo,
  				},
                src: __props.post.files?.[0]?.thumbnailUrl,
                hash: __props.post.files?.[0]?.blurhash,
                forceBlurhash: !show.value
              }, null, 8 /* PROPS */, ["transition", "src", "hash", "forceBlurhash"])
            ]),
            _: 1 /* STABLE */
          })
        ]),
        _createElementVNode("article", null, [
          _createElementVNode("header", null, [
            _createVNode(_component_MkAvatar, {
              user: __props.post.user,
              class: "avatar",
              link: "",
              preview: ""
            }, null, 8 /* PROPS */, ["user"])
          ]),
          _createElementVNode("footer", null, [
            _createElementVNode("span", _hoisted_1, _toDisplayString(__props.post.title), 1 /* TEXT */)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["to"]))
}
}

})
