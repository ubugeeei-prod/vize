import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-copy" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_3 = { class: "_selectableAtomic" }
import { computed } from 'vue'
import * as Misskey from 'misskey-js'
import MkFolder from '@/components/MkFolder.vue'
import MkButton from '@/components/MkButton.vue'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkInviteCode',
  props: {
    invite: { type: null, required: true },
    moderator: { type: Boolean, required: false }
  },
  emits: ["deleted"],
  setup(__props: any, { emit: __emit }) {

const emits = __emit
const props = __props
const isExpired = computed(() => {
	return props.invite.expiresAt && new Date(props.invite.expiresAt) < new Date();
});
function deleteCode() {
	os.apiWithDialog('invite/delete', {
		inviteId: props.invite.id,
	});
	emits('deleted', props.invite.id);
}
function copyInviteCode() {
	copyToClipboard(props.invite.code);
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkTime = _resolveComponent("MkTime")

  return (_openBlock(), _createBlock(MkFolder, null, {
      label: _withCtx(() => [
        _createTextVNode(_toDisplayString(__props.invite.code), 1 /* TEXT */)
      ]),
      suffix: _withCtx(() => [
        (__props.invite.used)
          ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(_unref(i18n).ts.used), 1 /* TEXT */))
          : (isExpired.value)
            ? (_openBlock(), _createElementBlock("span", {
              key: 1,
              style: "color: var(--MI_THEME-error)"
            }, _toDisplayString(_unref(i18n).ts.expired), 1 /* TEXT */))
          : (_openBlock(), _createElementBlock("span", {
            key: 2,
            style: "color: var(--MI_THEME-success)"
          }, _toDisplayString(_unref(i18n).ts.unused), 1 /* TEXT */))
      ]),
      footer: _withCtx(() => [
        _createElementVNode("div", { class: "_buttons" }, [
          (!__props.invite.used && !isExpired.value)
            ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              primary: "",
              rounded: "",
              onClick: _cache[0] || (_cache[0] = ($event: any) => (copyInviteCode()))
            }, {
              default: _withCtx(() => [
                _hoisted_1,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.copy), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : _createCommentVNode("v-if", true),
          (!__props.invite.used || __props.moderator)
            ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              danger: "",
              rounded: "",
              onClick: _cache[1] || (_cache[1] = ($event: any) => (deleteCode()))
            }, {
              default: _withCtx(() => [
                _hoisted_2,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.items)
          }, [
            _createElementVNode("div", null, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.label)
              }, _toDisplayString(_unref(i18n).ts.invitationCode), 1 /* TEXT */),
              _createElementVNode("div", _hoisted_3, _toDisplayString(__props.invite.code), 1 /* TEXT */)
            ]),
            (__props.moderator)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.label)
                }, _toDisplayString(_unref(i18n).ts.inviteCodeCreator), 1 /* TEXT */),
                (__props.invite.createdBy)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.user)
                  }, [
                    _createVNode(_component_MkAvatar, {
                      user: __props.invite.createdBy,
                      class: _normalizeClass(_ctx.$style.avatar),
                      link: "",
                      preview: ""
                    }, null, 8 /* PROPS */, ["user"]),
                    _createVNode(_component_MkUserName, {
                      user: __props.invite.createdBy,
                      nowrap: false
                    }, null, 8 /* PROPS */, ["user", "nowrap"]),
                    (__props.moderator)
                      ? (_openBlock(), _createElementBlock("div", { key: 0 }, "(" + _toDisplayString(__props.invite.createdBy.id) + ")", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ]))
                  : (_openBlock(), _createElementBlock("div", { key: 1 }, "system"))
              ]))
              : _createCommentVNode("v-if", true),
            (__props.invite.used)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.label)
                }, _toDisplayString(_unref(i18n).ts.registeredUserUsingInviteCode), 1 /* TEXT */),
                (__props.invite.usedBy)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.user)
                  }, [
                    _createVNode(_component_MkAvatar, {
                      user: __props.invite.usedBy,
                      class: _normalizeClass(_ctx.$style.avatar),
                      link: "",
                      preview: ""
                    }, null, 8 /* PROPS */, ["user"]),
                    _createVNode(_component_MkUserName, {
                      user: __props.invite.usedBy,
                      nowrap: false
                    }, null, 8 /* PROPS */, ["user", "nowrap"]),
                    (__props.moderator)
                      ? (_openBlock(), _createElementBlock("div", { key: 0 }, "(" + _toDisplayString(__props.invite.usedBy.id) + ")", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ]))
                  : (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts.unknown) + " (" + _toDisplayString(_unref(i18n).ts.waitingForMailAuth) + ")", 1 /* TEXT */))
              ]))
              : _createCommentVNode("v-if", true),
            (__props.invite.expiresAt && !__props.invite.used)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.label)
                }, _toDisplayString(_unref(i18n).ts.expirationDate), 1 /* TEXT */),
                _createElementVNode("div", null, [
                  _createVNode(_component_MkTime, {
                    time: __props.invite.expiresAt,
                    mode: "absolute"
                  }, null, 8 /* PROPS */, ["time"])
                ])
              ]))
              : _createCommentVNode("v-if", true),
            (__props.invite.usedAt)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.label)
                }, _toDisplayString(_unref(i18n).ts.inviteCodeUsedAt), 1 /* TEXT */),
                _createElementVNode("div", null, [
                  _createVNode(_component_MkTime, {
                    time: __props.invite.usedAt,
                    mode: "absolute"
                  }, null, 8 /* PROPS */, ["time"])
                ])
              ]))
              : _createCommentVNode("v-if", true),
            (__props.moderator)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.label)
                }, _toDisplayString(_unref(i18n).ts.createdAt), 1 /* TEXT */),
                _createElementVNode("div", null, [
                  _createVNode(_component_MkTime, {
                    time: __props.invite.createdAt,
                    mode: "absolute"
                  }, null, 8 /* PROPS */, ["time"])
                ])
              ]))
              : _createCommentVNode("v-if", true)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
