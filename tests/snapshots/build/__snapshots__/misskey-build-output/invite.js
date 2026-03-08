import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-plus" })
import { computed, markRaw, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkButton from '@/components/MkButton.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkInviteCode from '@/components/MkInviteCode.vue'
import { definePage } from '@/page.js'
import { instance } from '@/instance.js'
import { $i } from '@/i.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'invite',
  setup(__props) {

const currentInviteLimit = ref<null | number>(null);
const inviteLimit = (($i != null && $i.policies.inviteLimit) || (($i == null && instance.policies.inviteLimit))) as number;
const inviteLimitCycle = (($i != null && $i.policies.inviteLimitCycle) || ($i == null && instance.policies.inviteLimitCycle)) as number;
const paginator = markRaw(new Paginator('invite/list', {
	limit: 10,
}));
const resetCycle = computed<null | string>(() => {
	if (!inviteLimitCycle) return null;

	const minutes = inviteLimitCycle;
	if (minutes < 60) return minutes + i18n.ts._time.minute;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return hours + i18n.ts._time.hour;
	return Math.floor(hours / 24) + i18n.ts._time.day;
});
async function create() {
	const ticket = await misskeyApi('invite/create');
	os.alert({
		type: 'success',
		title: i18n.ts.inviteCodeCreated,
		text: ticket.code,
	});
	paginator.prepend(ticket);
	update();
}
function deleted(id: string) {
	paginator.removeItem(id);
	update();
}
async function update() {
	currentInviteLimit.value = (await misskeyApi('invite/limit')).remaining;
}
update();
definePage(() => ({
	title: i18n.ts.invite,
	icon: 'ti ti-user-plus',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        (!_unref(instance).disableRegistration || !(_unref($i) && (_unref($i).isAdmin || _unref($i).policies.canInvite)))
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_spacer",
            style: "--MI_SPACER-w: 1200px;"
          }, [
            _createVNode(_component_MkResult, { type: "empty" })
          ]))
          : (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: "_spacer",
            style: "--MI_SPACER-w: 800px;"
          }, [
            _createElementVNode("div", {
              class: "_gaps_m",
              style: "text-align: center;"
            }, [
              (resetCycle.value && _unref(inviteLimit))
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).tsx.inviteLimitResetCycle({ time: resetCycle.value, limit: _unref(inviteLimit) })), 1 /* TEXT */))
                : _createCommentVNode("v-if", true),
              _createVNode(MkButton, {
                inline: "",
                primary: "",
                rounded: "",
                disabled: currentInviteLimit.value !== null && currentInviteLimit.value <= 0,
                onClick: create
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.createInviteCode), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["disabled"]),
              (currentInviteLimit.value !== null)
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).tsx.createLimitRemaining({ limit: currentInviteLimit.value })), 1 /* TEXT */))
                : _createCommentVNode("v-if", true),
              _createVNode(MkPagination, { paginator: _unref(paginator) }, {
                default: _withCtx(({ items }) => [
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                      return (_openBlock(), _createBlock(MkInviteCode, {
                        key: item.id,
                        invite: item,
                        onDeleted: deleted
                      }, null, 8 /* PROPS */, ["invite", "onDeleted"]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["paginator"])
            ])
          ]))
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
