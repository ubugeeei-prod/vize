import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "acct _monospace" }
const _hoisted_2 = { class: "_monospace" }
import { computed } from 'vue'
import * as Misskey from 'misskey-js'
import { acct } from '@/filters/user.js'
import { i18n } from '@/i18n.js'
import MkKeyValue from '@/components/MkKeyValue.vue'
import FormSection from '@/components/form/section.vue'
import MkObjectView from '@/components/MkObjectView.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'raw',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const moderator = computed(() => props.user.isModerator ?? false);
const silenced = computed(() => props.user.isSilenced ?? false);
const suspended = computed(() => props.user.isSuspended ?? false);

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkTime = _resolveComponent("MkTime")

  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
    }, [ _createElementVNode("div", { class: "_gaps_m" }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.userMInfoRoot)
        }, [ _createVNode(_component_MkAvatar, {
            class: _normalizeClass(_ctx.$style.userMInfoAvatar),
            user: __props.user,
            indicator: "",
            link: "",
            preview: ""
          }, null, 8 /* PROPS */, ["user"]), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.userMInfoMetaRoot)
          }, [ _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.userMInfoMetaName)
            }, [ _createVNode(_component_MkUserName, {
                class: _normalizeClass(_ctx.$style.userMInfoMetaName),
                user: __props.user
              }, null, 8 /* PROPS */, ["user"]) ]), _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.userMInfoMetaSub)
            }, [ _createElementVNode("span", _hoisted_1, "@" + _toDisplayString(_unref(acct)(__props.user)), 1 /* TEXT */) ]), _createElementVNode("span", {
              class: _normalizeClass(_ctx.$style.userMInfoMetaState)
            }, [ (suspended.value) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.suspended)
                }, "Suspended")) : _createCommentVNode("v-if", true), (silenced.value) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.silenced)
                }, "Silenced")) : _createCommentVNode("v-if", true), (moderator.value) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.moderator)
                }, "Moderator")) : _createCommentVNode("v-if", true) ]) ]) ]), _createElementVNode("div", { style: "display: flex; flex-direction: column; gap: 1em;" }, [ _createVNode(MkKeyValue, {
            copy: __props.user.id,
            oneline: ""
          }, {
            key: _withCtx(() => [
              _createTextVNode("ID")
            ]),
            value: _withCtx(() => [
              _createElementVNode("span", _hoisted_2, _toDisplayString(__props.user.id), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["copy"]), _createVNode(MkKeyValue, { oneline: "" }, {
            key: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.createdAt), 1 /* TEXT */)
            ]),
            value: _withCtx(() => [
              _createElementVNode("span", { class: "_monospace" }, [
                _createVNode(_component_MkTime, {
                  time: __props.user.createdAt,
                  mode: 'detail'
                }, null, 8 /* PROPS */, ["time", "mode"])
              ])
            ]),
            _: 1 /* STABLE */
          }) ]), _createVNode(FormSection, null, {
          label: _withCtx(() => [
            _createTextVNode("Raw")
          ]),
          default: _withCtx(() => [
            _createVNode(MkObjectView, {
              tall: "",
              value: __props.user
            }, null, 8 /* PROPS */, ["value"])
          ]),
          _: 1 /* STABLE */
        }) ]) ]))
}
}

})
