import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "acct" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import * as Misskey from 'misskey-js'
import { computed, markRaw, ref, watch } from 'vue'
import MkPagination from '@/components/MkPagination.vue'
import MkButton from '@/components/MkButton.vue'
import { userPage, acct } from '@/filters/user.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { $i } from '@/i.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'follow-requests',
  setup(__props) {

const tab = ref($i?.isLocked ? 'list' : 'sent');
let paginator: Paginator<'following/requests/list' | 'following/requests/sent'>;
watch(tab, (newTab) => {
	if (newTab === 'list') {
		paginator = markRaw(new Paginator('following/requests/list', { limit: 10 }));
	} else {
		paginator = markRaw(new Paginator('following/requests/sent', { limit: 10 }));
	}
}, { immediate: true });
function accept(user: Misskey.entities.UserLite) {
	os.apiWithDialog('following/requests/accept', { userId: user.id }).then(() => {
		paginator.reload();
	});
}
async function reject(user: Misskey.entities.UserLite) {
	const { canceled } = await os.confirm({
		type: 'question',
		text: i18n.tsx.rejectFollowRequestConfirm({ name: user.name || user.username }),
	});
	if (canceled) return;
	await os.apiWithDialog('following/requests/reject', { userId: user.id }).then(() => {
		paginator.reload();
	});
}
async function cancel(user: Misskey.entities.UserLite) {
	const { canceled } = await os.confirm({
		type: 'question',
		text: i18n.tsx.cancelFollowRequestConfirm({ name: user.name || user.username }),
	});
	if (canceled) return;
	await os.apiWithDialog('following/requests/cancel', { userId: user.id }).then(() => {
		paginator.reload();
	});
}
function displayUser(req: Misskey.entities.FollowingRequestsListResponse[number]) {
	return tab.value === 'list' ? req.follower : req.followee;
}
const headerActions = computed(() => []);
const headerTabs = computed(() => [
	{
		key: 'list',
		title: i18n.ts._followRequest.recieved,
		icon: 'ti ti-download',
	}, {
		key: 'sent',
		title: i18n.ts._followRequest.sent,
		icon: 'ti ti-upload',
	},
]);
definePage(() => ({
	title: i18n.ts.followRequests,
	icon: 'ti ti-user-plus',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkA = _resolveComponent("MkA")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_user_preview = _resolveDirective("user-preview")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      swipable: true,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          key: tab.value,
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createVNode(MkPagination, { paginator: _unref(paginator) }, {
            empty: _withCtx(() => [
              _createVNode(_component_MkResult, {
                type: "empty",
                text: _unref(i18n).ts.noFollowRequests
              }, null, 8 /* PROPS */, ["text"])
            ]),
            default: _withCtx(({items}) => [
              _createElementVNode("div", { class: "mk-follow-requests _gaps" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (req) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: req.id,
                    class: "user _panel"
                  }, [
                    _createVNode(_component_MkAvatar, {
                      class: "avatar",
                      user: displayUser(req),
                      indicator: "",
                      link: "",
                      preview: ""
                    }, null, 8 /* PROPS */, ["user"]),
                    _createElementVNode("div", { class: "body" }, [
                      _createElementVNode("div", { class: "name" }, [
                        _createVNode(_component_MkA, {
                          class: "name",
                          to: _unref(userPage)(displayUser(req))
                        }, {
                          default: _withCtx(() => [
                            _createVNode(_component_MkUserName, { user: displayUser(req) }, null, 8 /* PROPS */, ["user"])
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 8 /* PROPS */, ["to"]),
                        _createElementVNode("p", _hoisted_1, "@" + _toDisplayString(_unref(acct)(displayUser(req))), 1 /* TEXT */)
                      ]),
                      (tab.value === 'list')
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: "commands"
                        }, [
                          _createVNode(MkButton, {
                            class: "command",
                            rounded: "",
                            primary: "",
                            onClick: _cache[1] || (_cache[1] = ($event: any) => (accept(displayUser(req))))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_2,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.accept), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }),
                          _createVNode(MkButton, {
                            class: "command",
                            rounded: "",
                            danger: "",
                            onClick: _cache[2] || (_cache[2] = ($event: any) => (reject(displayUser(req))))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_3,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.reject), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          })
                        ]))
                        : (_openBlock(), _createElementBlock("div", {
                          key: 1,
                          class: "commands"
                        }, [
                          _createVNode(MkButton, {
                            class: "command",
                            rounded: "",
                            danger: "",
                            onClick: _cache[3] || (_cache[3] = ($event: any) => (cancel(displayUser(req))))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_4,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.cancel), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          })
                        ]))
                    ])
                  ]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["paginator"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "swipable", "tab"]))
}
}

})
