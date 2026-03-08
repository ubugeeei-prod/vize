import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { instanceName } from '@@/js/config.js'
import type { MenuItem } from '@/types/menu.js'
import XSigninDialog from '@/components/MkSigninDialog.vue'
import XSignupDialog from '@/components/MkSignupDialog.vue'
import MkButton from '@/components/MkButton.vue'
import MkStreamingNotesTimeline from '@/components/MkStreamingNotesTimeline.vue'
import MkInfo from '@/components/MkInfo.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import MkNumber from '@/components/MkNumber.vue'
import XActiveUsersChart from '@/components/MkVisitorDashboard.ActiveUsersChart.vue'
import { openInstanceMenu } from '@/ui/_common_/common.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkVisitorDashboard',
  setup(__props) {

const stats = ref<Misskey.entities.StatsResponse | null>(null);
if (instance.clientOptions.showActivitiesForVisitor !== false) {
	misskeyApi('stats', {}).then((res) => {
		stats.value = res;
	});
}
function signin() {
	const { dispose } = os.popup(XSigninDialog, {
		autoSet: true,
	}, {
		closed: () => dispose(),
	});
}
function signup() {
	const { dispose } = os.popup(XSignupDialog, {
		autoSet: true,
	}, {
		closed: () => dispose(),
	});
}
function showMenu(ev: PointerEvent) {
	openInstanceMenu(ev);
}

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")

  return (_unref(instance))
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        class: _normalizeClass(_ctx.$style.root)
      }, [ _createElementVNode("div", {
          class: _normalizeClass([_ctx.$style.main, _ctx.$style.panel])
        }, [ _createElementVNode("img", {
            src: _unref(instance).iconUrl || '/favicon.ico',
            alt: "",
            class: _normalizeClass(_ctx.$style.mainIcon)
          }, null, 8 /* PROPS */, ["src"]), _createElementVNode("button", {
            class: _normalizeClass(["_button _acrylic", _ctx.$style.mainMenu]),
            onClick: showMenu
          }, [ _hoisted_1 ]), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.mainFg)
          }, [ _createElementVNode("h1", {
              class: _normalizeClass(_ctx.$style.mainTitle)
            }, [ _createVNode(_component_MkA, { to: "/" }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(instanceName)), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.mainAbout)
            }, [ _createElementVNode("div", { innerHTML: _unref(instance).description || _unref(i18n).ts.headlineMisskey }, null, 8 /* PROPS */, ["innerHTML"]) ]), (_unref(instance).disableRegistration || _unref(instance).federation !== 'all') ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(["_gaps_s", _ctx.$style.mainWarn])
              }, [ (_unref(instance).disableRegistration) ? (_openBlock(), _createBlock(MkInfo, {
                    key: 0,
                    warn: ""
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.invitationRequiredToRegister), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })) : _createCommentVNode("v-if", true), (_unref(instance).federation === 'specified') ? (_openBlock(), _createBlock(MkInfo, {
                    key: 0,
                    warn: ""
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.federationSpecified), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })) : (_unref(instance).federation === 'none') ? (_openBlock(), _createBlock(MkInfo, {
                      key: 1,
                      warn: ""
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.federationDisabled), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
              class: _normalizeClass(["_gaps_s", _ctx.$style.mainActions])
            }, [ _createVNode(MkButton, {
                class: _normalizeClass(_ctx.$style.mainAction),
                full: "",
                rounded: "",
                gradate: "",
                "data-cy-signup": "",
                style: "margin-right: 12px;",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (signup()))
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.joinThisServer), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkButton, {
                class: _normalizeClass(_ctx.$style.mainAction),
                full: "",
                rounded: "",
                link: "",
                to: "https://misskey-hub.net/servers/"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.exploreOtherServers), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }), _createVNode(MkButton, {
                class: _normalizeClass(_ctx.$style.mainAction),
                full: "",
                rounded: "",
                "data-cy-signin": "",
                onClick: _cache[1] || (_cache[1] = ($event: any) => (signin()))
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.login), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }) ]) ]) ]), (stats.value && _unref(instance).clientOptions.showActivitiesForVisitor !== false) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.stats)
          }, [ _createElementVNode("div", {
              class: _normalizeClass([_ctx.$style.statsItem, _ctx.$style.panel])
            }, [ _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.statsItemLabel)
              }, _toDisplayString(_unref(i18n).ts.users), 1 /* TEXT */), _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.statsItemCount)
              }, [ _createVNode(MkNumber, { value: stats.value.originalUsersCount }, null, 8 /* PROPS */, ["value"]) ]) ]), _createElementVNode("div", {
              class: _normalizeClass([_ctx.$style.statsItem, _ctx.$style.panel])
            }, [ _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.statsItemLabel)
              }, _toDisplayString(_unref(i18n).ts.notes), 1 /* TEXT */), _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.statsItemCount)
              }, [ _createVNode(MkNumber, { value: stats.value.originalNotesCount }, null, 8 /* PROPS */, ["value"]) ]) ]) ])) : _createCommentVNode("v-if", true), (_unref(instance).policies.ltlAvailable && _unref(instance).clientOptions.showTimelineForVisitor !== false) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass([_ctx.$style.tl, _ctx.$style.panel])
          }, [ _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.tlHeader)
            }, _toDisplayString(_unref(i18n).ts.letsLookAtTimeline), 1 /* TEXT */), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.tlBody)
            }, [ _createVNode(MkStreamingNotesTimeline, { src: "local" }) ]) ])) : _createCommentVNode("v-if", true), (_unref(instance).clientOptions.showActivitiesForVisitor !== false) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.panel)
          }, [ _createVNode(XActiveUsersChart) ])) : _createCommentVNode("v-if", true) ]))
      : _createCommentVNode("v-if", true)
}
}

})
