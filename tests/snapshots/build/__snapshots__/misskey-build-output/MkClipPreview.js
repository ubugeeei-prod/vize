import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import { computed } from 'vue'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'
import number from '@/filters/number.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkClipPreview',
  props: {
    clip: { type: null, required: true },
    noUserInfo: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props
const remaining = computed(() => {
	return ($i?.policies && props.clip.notesCount != null) ? ($i.policies.noteEachClipsLimit - props.clip.notesCount) : i18n.ts.unknown;
});

return (_ctx: any,_cache: any) => {
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createBlock(_component_MkA, {
      to: `/clips/${__props.clip.id}`,
      class: _normalizeClass(_ctx.$style.link)
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(["_panel _gaps_s", _ctx.$style.root])
        }, [
          _createElementVNode("b", null, _toDisplayString(__props.clip.name), 1 /* TEXT */),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.description)
          }, [
            (__props.clip.description)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                _createVNode(_component_Mfm, {
                  text: __props.clip.description,
                  plain: true,
                  nowrap: true
                }, null, 8 /* PROPS */, ["text", "plain", "nowrap"])
              ]))
              : _createCommentVNode("v-if", true),
            (__props.clip.lastClippedAt)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                _toDisplayString(_unref(i18n).ts.updatedAt),
                _createTextVNode(": "),
                _createVNode(_component_MkTime, {
                  time: __props.clip.lastClippedAt,
                  mode: "detail"
                }, null, 8 /* PROPS */, ["time"])
              ]))
              : _createCommentVNode("v-if", true),
            (__props.clip.notesCount != null)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts.notesCount) + ": " + _toDisplayString(number(__props.clip.notesCount)) + " / " + _toDisplayString(_unref($i)?.policies.noteEachClipsLimit) + " (" + _toDisplayString(_unref(i18n).tsx.remainingN({ n: remaining.value })) + ")", 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ]),
          (!props.noUserInfo)
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.divider)
              }),
              _createElementVNode("div", null, [
                _createVNode(_component_MkAvatar, {
                  user: __props.clip.user,
                  class: _normalizeClass(_ctx.$style.userAvatar),
                  indicator: "",
                  link: "",
                  preview: ""
                }, null, 8 /* PROPS */, ["user"]),
                _createTextVNode(),
                _createVNode(_component_MkUserName, {
                  user: __props.clip.user,
                  nowrap: false
                }, null, 8 /* PROPS */, ["user", "nowrap"])
              ])
            ], 64 /* STABLE_FRAGMENT */))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["to"]))
}
}

})
