import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check", style: "color: var(--MI_THEME-success)" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x", style: "color: var(--MI_THEME-error)" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-slash" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-corner-up-right" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-help-circle" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message-2" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message-2" })
import { provide, ref, watch } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { dateString } from '@/filters/date.js'
import MkFolder from '@/components/MkFolder.vue'
import RouterView from '@/components/global/RouterView.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { createRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAbuseReport',
  props: {
    report: { type: null, required: true }
  },
  emits: ["resolved"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const targetRouter = createRouter(`/admin/user/${props.report.targetUserId}`);
targetRouter.init();
const reporterRouter = createRouter(`/admin/user/${props.report.reporterId}`);
reporterRouter.init();
const moderationNote = ref(props.report.moderationNote ?? '');
watch(moderationNote, async () => {
	os.apiWithDialog('admin/update-abuse-user-report', {
		reportId: props.report.id,
		moderationNote: moderationNote.value,
	}).then(() => {
	});
});
function resolve(resolvedAs: 'accept' | 'reject' | null) {
	os.apiWithDialog('admin/resolve-abuse-user-report', {
		reportId: props.report.id,
		resolvedAs,
	}).then(() => {
		emit('resolved', props.report.id);
	});
}
function forward() {
	os.apiWithDialog('admin/forward-abuse-user-report', {
		reportId: props.report.id,
	}).then(() => {
	});
}
function showMenu(ev: PointerEvent) {
	os.popupMenu([{
		icon: 'ti ti-hash',
		text: 'Copy ID',
		action: () => {
			copyToClipboard(props.report.id);
		},
	}, {
		icon: 'ti ti-json',
		text: 'Copy JSON',
		action: () => {
			copyToClipboard(JSON.stringify(props.report, null, '\t'));
		},
	}], ev.currentTarget ?? ev.target);
}

return (_ctx: any,_cache: any) => {
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_Mfm = _resolveComponent("Mfm")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(MkFolder, null, {
      icon: _withCtx(() => [
        (__props.report.resolved && __props.report.resolvedAs === 'accept')
          ? (_openBlock(), _createElementBlock("i", {
            key: 0,
            class: "ti ti-check",
            style: "color: var(--MI_THEME-success)"
          }))
          : (__props.report.resolved && __props.report.resolvedAs === 'reject')
            ? (_openBlock(), _createElementBlock("i", {
              key: 1,
              class: "ti ti-x",
              style: "color: var(--MI_THEME-error)"
            }))
          : (__props.report.resolved)
            ? (_openBlock(), _createElementBlock("i", {
              key: 2,
              class: "ti ti-slash"
            }))
          : (_openBlock(), _createElementBlock("i", {
            key: 3,
            class: "ti ti-exclamation-circle",
            style: "color: var(--MI_THEME-warn)"
          }))
      ]),
      label: _withCtx(() => [
        _createVNode(_component_MkAcct, { user: __props.report.targetUser }, null, 8 /* PROPS */, ["user"]),
        _createTextVNode(" (by "),
        _createVNode(_component_MkAcct, { user: __props.report.reporter }, null, 8 /* PROPS */, ["user"]),
        _createTextVNode(")")
      ]),
      caption: _withCtx(() => [
        _createTextVNode(_toDisplayString(__props.report.comment), 1 /* TEXT */)
      ]),
      suffix: _withCtx(() => [
        _createVNode(_component_MkTime, { time: __props.report.createdAt }, null, 8 /* PROPS */, ["time"])
      ]),
      footer: _withCtx(() => [
        _createElementVNode("div", { class: "_buttons" }, [
          (!__props.report.resolved)
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _createVNode(MkButton, {
                onClick: _cache[0] || (_cache[0] = ($event: any) => (resolve('accept')))
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseUserReport.resolve), 1 /* TEXT */),
                  _createTextVNode(" ("),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseUserReport.accept), 1 /* TEXT */),
                  _createTextVNode(")")
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, {
                onClick: _cache[1] || (_cache[1] = ($event: any) => (resolve('reject')))
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseUserReport.resolve), 1 /* TEXT */),
                  _createTextVNode(" ("),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseUserReport.reject), 1 /* TEXT */),
                  _createTextVNode(")")
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, {
                onClick: _cache[2] || (_cache[2] = ($event: any) => (resolve(null)))
              }, {
                default: _withCtx(() => [
                  _hoisted_3,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseUserReport.resolve), 1 /* TEXT */),
                  _createTextVNode(" ("),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.other), 1 /* TEXT */),
                  _createTextVNode(")")
                ]),
                _: 1 /* STABLE */
              })
            ], 64 /* STABLE_FRAGMENT */))
            : _createCommentVNode("v-if", true),
          (__props.report.targetUser.host != null)
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _createVNode(MkButton, {
                disabled: __props.report.forwarded,
                primary: "",
                onClick: forward
              }, {
                default: _withCtx(() => [
                  _hoisted_4,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseUserReport.forward), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["disabled"]),
              _createElementVNode("div", { class: "_button _help" }, [
                _hoisted_5
              ])
            ], 64 /* STABLE_FRAGMENT */))
            : _createCommentVNode("v-if", true),
          _createElementVNode("button", {
            class: "_button",
            style: "margin-left: auto; width: 34px;",
            onClick: showMenu
          }, [
            _hoisted_6
          ])
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_s" }, [
          _createVNode(MkFolder, { withSpacer: false }, {
            icon: _withCtx(() => [
              _createVNode(_component_MkAvatar, {
                user: __props.report.targetUser,
                style: "width: 18px; height: 18px;"
              }, null, 8 /* PROPS */, ["user"])
            ]),
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.target), 1 /* TEXT */),
              _createTextVNode(": "),
              _createVNode(_component_MkAcct, { user: __props.report.targetUser }, null, 8 /* PROPS */, ["user"])
            ]),
            suffix: _withCtx(() => [
              _createTextVNode("#" + _toDisplayString(__props.report.targetUserId.toUpperCase()), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { style: "height: 300px; --MI-stickyTop: 0; --MI-stickyBottom: 0;" }, [
                _createVNode(RouterView, { router: _unref(targetRouter) }, null, 8 /* PROPS */, ["router"])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["withSpacer"]),
          _createVNode(MkFolder, { defaultOpen: true }, {
            icon: _withCtx(() => [
              _hoisted_7
            ]),
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.details), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_s" }, [
                _createVNode(_component_Mfm, {
                  text: __props.report.comment,
                  linkNavigationBehavior: 'window'
                }, null, 8 /* PROPS */, ["text", "linkNavigationBehavior"])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["defaultOpen"]),
          _createVNode(MkFolder, { withSpacer: false }, {
            icon: _withCtx(() => [
              _createVNode(_component_MkAvatar, {
                user: __props.report.reporter,
                style: "width: 18px; height: 18px;"
              }, null, 8 /* PROPS */, ["user"])
            ]),
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.reporter), 1 /* TEXT */),
              _createTextVNode(": "),
              _createVNode(_component_MkAcct, { user: __props.report.reporter }, null, 8 /* PROPS */, ["user"])
            ]),
            suffix: _withCtx(() => [
              _createTextVNode("#" + _toDisplayString(__props.report.reporterId.toUpperCase()), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { style: "height: 300px; --MI-stickyTop: 0; --MI-stickyBottom: 0;" }, [
                _createVNode(RouterView, { router: _unref(reporterRouter) }, null, 8 /* PROPS */, ["router"])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["withSpacer"]),
          _createVNode(MkFolder, { defaultOpen: false }, {
            icon: _withCtx(() => [
              _hoisted_8
            ]),
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.moderationNote), 1 /* TEXT */)
            ]),
            suffix: _withCtx(() => [
              _createTextVNode(_toDisplayString(moderationNote.value.length > 0 ? '...' : _unref(i18n).ts.none), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_s" }, [
                _createVNode(MkTextarea, {
                  manualSave: "",
                  modelValue: moderationNote.value,
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((moderationNote).value = $event))
                }, {
                  caption: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.moderationNoteDescription), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"])
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["defaultOpen"]),
          (__props.report.assignee)
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              _toDisplayString(_unref(i18n).ts.moderator),
              _createTextVNode(":\n\t\t\t"),
              _createVNode(_component_MkAcct, { user: __props.report.assignee }, null, 8 /* PROPS */, ["user"])
            ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
