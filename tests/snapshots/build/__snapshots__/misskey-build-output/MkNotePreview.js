import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, normalizeClass as _normalizeClass, vShow as _vShow } from "vue"

import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import type { PollEditorModelValue } from '@/components/MkPollEditor.vue'
import MkCwButton from '@/components/MkCwButton.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNotePreview',
  props: {
    text: { type: String, required: true },
    files: { type: Array, required: true },
    poll: { type: null, required: false },
    useCw: { type: Boolean, required: true },
    cw: { type: String, required: true },
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const showContent = ref(false);

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_Mfm = _resolveComponent("Mfm")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createVNode(_component_MkAvatar, {
        class: _normalizeClass(_ctx.$style.avatar),
        user: __props.user
      }, null, 8 /* PROPS */, ["user"]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.main)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.header)
        }, [ _createVNode(_component_MkUserName, {
            user: __props.user,
            nowrap: true
          }, null, 8 /* PROPS */, ["user", "nowrap"]) ]), _createElementVNode("div", null, [ (__props.useCw) ? (_openBlock(), _createElementBlock("p", {
              key: 0,
              class: _normalizeClass(_ctx.$style.cw)
            }, [ (__props.cw != null && __props.cw != '') ? (_openBlock(), _createBlock(_component_Mfm, {
                  key: 0,
                  text: __props.cw,
                  author: __props.user,
                  nyaize: 'respect',
                  i: __props.user,
                  style: "margin-right: 8px;"
                }, null, 8 /* PROPS */, ["text", "author", "nyaize", "i"])) : _createCommentVNode("v-if", true), _createVNode(MkCwButton, {
                text: __props.text.trim(),
                files: __props.files,
                poll: __props.poll,
                style: "margin: 4px 0;",
                modelValue: showContent.value,
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((showContent).value = $event))
              }, null, 8 /* PROPS */, ["text", "files", "poll", "modelValue"]) ])) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", null, [ _createVNode(_component_Mfm, {
              text: __props.text.trim(),
              author: __props.user,
              nyaize: 'respect',
              i: __props.user
            }, null, 8 /* PROPS */, ["text", "author", "nyaize", "i"]) ], 512 /* NEED_PATCH */), [ [_vShow, !__props.useCw || showContent.value] ]) ]) ]) ]))
}
}

})
