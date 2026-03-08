import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "acct _monospace" }
const _hoisted_2 = { class: "_monospace" }
const _hoisted_3 = { class: "_monospace" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-key" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-license" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-password" })
const _hoisted_7 = { class: "date" }
const _hoisted_8 = { class: "ip" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-circle" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-ban" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_16 = { class: "label" }
const _hoisted_17 = { class: "label" }
import { computed, defineAsyncComponent, watch, ref, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import { url } from '@@/js/config.js'
import type { ChartSrc } from '@/components/MkChart.vue'
import MkChart from '@/components/MkChart.vue'
import MkObjectView from '@/components/MkObjectView.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import FormLink from '@/components/form/link.vue'
import FormSection from '@/components/form/section.vue'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkFileListForAdmin from '@/components/MkFileListForAdmin.vue'
import MkInfo from '@/components/MkInfo.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { acct } from '@/filters/user.js'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import { ensureSignin, iAmAdmin, iAmModerator } from '@/i.js'
import MkRolePreview from '@/components/MkRolePreview.vue'
import MkPagination from '@/components/MkPagination.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'admin-user',
  props: {
    userId: { type: String, required: true },
    initialTab: { type: String, required: false, default: 'overview' }
  },
  async setup(__props: any) {

let __temp: any, __restore: any

const props = __props
const $i = ensureSignin();
const result = await _fetch_();
const tab = ref(props.initialTab);
const {
	model: chartSrc,
	def: chartSrcDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.notes, value: 'per-user-notes' },
	],
	initialValue: 'per-user-notes',
});
const user = ref(result.user);
const info = ref(result.info);
const ips = ref(result.ips);
const ap = ref<Misskey.entities.ApGetResponse | null>(null);
const moderator = ref(info.value.isModerator);
const silenced = ref(info.value.isSilenced);
const suspended = ref(info.value.isSuspended);
const isSystem = ref(user.value.host == null && user.value.username.includes('.'));
const moderationNote = ref(info.value.moderationNote);
const filesPaginator = markRaw(new Paginator('admin/drive/files', {
	limit: 10,
	computedParams: computed(() => ({
		userId: props.userId,
	})),
}));
const {
	model: announcementsStatus,
	def: announcementsStatusDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.active, value: 'active' },
		{ label: i18n.ts.archived, value: 'archived' },
	],
	initialValue: 'active',
});
const announcementsPaginator = markRaw(new Paginator('admin/announcements/list', {
	limit: 10,
	computedParams: computed(() => ({
		userId: props.userId,
		status: announcementsStatus.value,
	})),
}));
const expandedRoleIds = ref<(typeof info.value.roles[number]['id'])[]>([]);
function _fetch_() {
	return Promise.all([misskeyApi('users/show', {
		userId: props.userId,
	}), misskeyApi('admin/show-user', {
		userId: props.userId,
	}), iAmAdmin ? misskeyApi('admin/get-user-ips', {
		userId: props.userId,
	}) : Promise.resolve(null)]).then(([_user, _info, _ips]) => ({
		user: _user,
		info: _info,
		ips: _ips,
	}));
}
watch(moderationNote, async () => {
;(
  ([__temp,__restore] = _withAsyncContext(() => misskeyApi('admin/update-user-note', { userId: user.value.id, text: moderationNote.value }))),
  await __temp,
  __restore()
)
;(
  ([__temp,__restore] = _withAsyncContext(() => refreshUser())),
  await __temp,
  __restore()
)
});
async function refreshUser() {
	const result = await _fetch_();
	user.value = result.user;
	info.value = result.info;
	ips.value = result.ips;
	moderator.value = info.value.isModerator;
	silenced.value = info.value.isSilenced;
	suspended.value = info.value.isSuspended;
	isSystem.value = user.value.host == null && user.value.username.includes('.');
	moderationNote.value = info.value.moderationNote;
}
async function updateRemoteUser() {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('federation/update-remote-user', { userId: user.value.id }))),
  await __temp,
  __restore()
)
	refreshUser();
}
async function resetPassword() {
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.resetPasswordConfirm,
	});
	if (confirm.canceled) {
		return;
	} else {
		const { password } = await misskeyApi('admin/reset-password', {
			userId: user.value.id,
		});
		os.alert({
			type: 'success',
			text: i18n.tsx.newPasswordIs({ password }),
		});
	}
}
async function toggleSuspend(v: boolean) {
	const confirm = await os.confirm({
		type: 'warning',
		text: v ? i18n.ts.suspendConfirm : i18n.ts.unsuspendConfirm,
	});
	if (confirm.canceled) {
		suspended.value = !v;
	} else {
;(
  ([__temp,__restore] = _withAsyncContext(() => misskeyApi(v ? 'admin/suspend-user' : 'admin/unsuspend-user', { userId: user.value.id }))),
  await __temp,
  __restore()
)
;(
  ([__temp,__restore] = _withAsyncContext(() => refreshUser())),
  await __temp,
  __restore()
)
	}
}
async function unsetUserAvatar() {
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.unsetUserAvatarConfirm,
	});
	if (confirm.canceled) return;
	const process = async () => {
;(
  ([__temp,__restore] = _withAsyncContext(() => misskeyApi('admin/unset-user-avatar', { userId: user.value.id }))),
  await __temp,
  __restore()
)
		os.success();
	};
;(
  ([__temp,__restore] = _withAsyncContext(() => process().catch(err => {)),
  await __temp,
  __restore()
)
		os.alert({
			type: 'error',
			text: err.toString(),
		});
	});
	refreshUser();
}
async function unsetUserBanner() {
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.unsetUserBannerConfirm,
	});
	if (confirm.canceled) return;
	const process = async () => {
;(
  ([__temp,__restore] = _withAsyncContext(() => misskeyApi('admin/unset-user-banner', { userId: user.value.id }))),
  await __temp,
  __restore()
)
		os.success();
	};
;(
  ([__temp,__restore] = _withAsyncContext(() => process().catch(err => {)),
  await __temp,
  __restore()
)
		os.alert({
			type: 'error',
			text: err.toString(),
		});
	});
	refreshUser();
}
async function deleteAllFiles() {
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.deleteAllFilesConfirm,
	});
	if (confirm.canceled) return;
	const process = async () => {
;(
  ([__temp,__restore] = _withAsyncContext(() => misskeyApi('admin/delete-all-files-of-a-user', { userId: user.value.id }))),
  await __temp,
  __restore()
)
		os.success();
	};
;(
  ([__temp,__restore] = _withAsyncContext(() => process().catch(err => {)),
  await __temp,
  __restore()
)
		os.alert({
			type: 'error',
			text: err.toString(),
		});
	});
;(
  ([__temp,__restore] = _withAsyncContext(() => refreshUser())),
  await __temp,
  __restore()
)
}
async function deleteAccount() {
	const confirm = await os.confirm({
		type: 'warning',
		text: i18n.ts.deleteAccountConfirm,
	});
	if (confirm.canceled) return;
	const typed = await os.inputText({
		text: i18n.tsx.typeToConfirm({ x: user.value?.username }),
	});
	if (typed.canceled) return;
	if (typed.result === user.value?.username) {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/delete-account', {)),
  await __temp,
  __restore()
)
			userId: user.value.id,
		});
	} else {
		os.alert({
			type: 'error',
			text: 'input not match',
		});
	}
}
async function assignRole() {
	const roles = await misskeyApi('admin/roles/list').then(it => it.filter(r => r.target === 'manual'));
	const { canceled, result: roleId } = await os.select({
		title: i18n.ts._role.chooseRoleToAssign,
		items: roles.map(r => ({ label: r.name, value: r.id })),
	});
	if (canceled || roleId == null) return;
	const { canceled: canceled2, result: period } = await os.select({
		title: i18n.ts.period + ': ' + roles.find(r => r.id === roleId)!.name,
		items: [{
			value: 'indefinitely', label: i18n.ts.indefinitely,
		}, {
			value: 'oneHour', label: i18n.ts.oneHour,
		}, {
			value: 'oneDay', label: i18n.ts.oneDay,
		}, {
			value: 'oneWeek', label: i18n.ts.oneWeek,
		}, {
			value: 'oneMonth', label: i18n.ts.oneMonth,
		}],
		default: 'indefinitely',
	});
	if (canceled2) return;
	const expiresAt = period === 'indefinitely' ? null
		: period === 'oneHour' ? Date.now() + (1000 * 60 * 60)
		: period === 'oneDay' ? Date.now() + (1000 * 60 * 60 * 24)
		: period === 'oneWeek' ? Date.now() + (1000 * 60 * 60 * 24 * 7)
		: period === 'oneMonth' ? Date.now() + (1000 * 60 * 60 * 24 * 30)
		: null;
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/roles/assign', { roleId, userId: user.value.id, expiresAt }))),
  await __temp,
  __restore()
)
	refreshUser();
}
async function unassignRole(role: typeof info.value.roles[number], ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.unassign,
		icon: 'ti ti-x',
		danger: true,
		action: async () => {
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('admin/roles/unassign', { roleId: role.id, userId: user.value.id }))),
  await __temp,
  __restore()
)
			refreshUser();
		},
	}], ev.currentTarget ?? ev.target);
}
function toggleRoleItem(role: typeof info.value.roles[number]) {
	if (expandedRoleIds.value.includes(role.id)) {
		expandedRoleIds.value = expandedRoleIds.value.filter(x => x !== role.id);
	} else {
		expandedRoleIds.value.push(role.id);
	}
}
async function createAnnouncement() {
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkUserAnnouncementEditDialog.vue').then(x => x.default), {
		user: user.value,
	}, {
		closed: () => dispose(),
	});
}
async function editAnnouncement(announcement: Misskey.entities.AdminAnnouncementsListResponse[number]) {
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkUserAnnouncementEditDialog.vue').then(x => x.default), {
		user: user.value,
		announcement,
	}, {
		closed: () => dispose(),
	});
}
watch(user, () => {
	misskeyApi('ap/get', {
		uri: user.value.uri ?? `${url}/users/${user.value.id}`,
	}).then(res => {
		ap.value = res;
	});
});
const headerActions = computed(() => []);
const headerTabs = computed(() => isSystem.value ? [{
	key: 'overview',
	title: i18n.ts.overview,
	icon: 'ti ti-info-circle',
}, {
	key: 'raw',
	title: 'Raw',
	icon: 'ti ti-code',
}] : [{
	key: 'overview',
	title: i18n.ts.overview,
	icon: 'ti ti-info-circle',
}, {
	key: 'roles',
	title: i18n.ts.roles,
	icon: 'ti ti-badges',
}, {
	key: 'announcements',
	title: i18n.ts.announcements,
	icon: 'ti ti-speakerphone',
}, {
	key: 'drive',
	title: i18n.ts.drive,
	icon: 'ti ti-cloud',
}, {
	key: 'chart',
	title: i18n.ts.charts,
	icon: 'ti ti-chart-line',
}, {
	key: 'raw',
	title: 'Raw',
	icon: 'ti ti-code',
}]);
definePage(() => ({
	title: user.value ? acct(user.value) : i18n.ts.userInfo,
	icon: 'ti ti-user-exclamation',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          (tab.value === 'overview')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps_m"
            }, [
              _createElementVNode("div", { class: "aeakzknw" }, [
                _createVNode(_component_MkAvatar, {
                  class: "avatar",
                  user: user.value,
                  indicator: "",
                  link: "",
                  preview: ""
                }, null, 8 /* PROPS */, ["user"]),
                _createElementVNode("div", { class: "body" }, [
                  _createElementVNode("span", { class: "name" }, [
                    _createVNode(_component_MkUserName, {
                      class: "name",
                      user: user.value
                    }, null, 8 /* PROPS */, ["user"])
                  ]),
                  _createElementVNode("span", { class: "sub" }, [
                    _createElementVNode("span", _hoisted_1, "@" + _toDisplayString(_unref(acct)(user.value)), 1 /* TEXT */)
                  ]),
                  _createElementVNode("span", { class: "state" }, [
                    (suspended.value)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "suspended"
                      }, "Suspended"))
                      : _createCommentVNode("v-if", true),
                    (silenced.value)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "silenced"
                      }, "Silenced"))
                      : _createCommentVNode("v-if", true),
                    (moderator.value)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "moderator"
                      }, "Moderator"))
                      : _createCommentVNode("v-if", true)
                  ])
                ])
              ]),
              (isSystem.value)
                ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.isSystemAccount), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              (user.value.host)
                ? (_openBlock(), _createBlock(FormLink, {
                  key: 0,
                  to: `/instance-info/${user.value.host}`
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.instanceInfo), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"]))
                : _createCommentVNode("v-if", true),
              _createElementVNode("div", { style: "display: flex; flex-direction: column; gap: 1em;" }, [
                _createVNode(MkKeyValue, {
                  copy: user.value.id,
                  oneline: ""
                }, {
                  key: _withCtx(() => [
                    _createTextVNode("ID")
                  ]),
                  value: _withCtx(() => [
                    _createElementVNode("span", _hoisted_2, _toDisplayString(user.value.id), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["copy"]),
                _createTextVNode("\n\t\t\t\t" + "\n\t\t\t\t"),
                (!isSystem.value)
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    _createVNode(MkKeyValue, { oneline: "" }, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.createdAt), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        _createElementVNode("span", { class: "_monospace" }, [
                          _createVNode(_component_MkTime, {
                            time: user.value.createdAt,
                            mode: 'detail'
                          }, null, 8 /* PROPS */, ["time", "mode"])
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }),
                    (info.value)
                      ? (_openBlock(), _createBlock(MkKeyValue, {
                        key: 0,
                        oneline: ""
                      }, {
                        key: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.lastActiveDate), 1 /* TEXT */)
                        ]),
                        value: _withCtx(() => [
                          _createElementVNode("span", { class: "_monospace" }, [
                            _createVNode(_component_MkTime, {
                              time: info.value.lastActiveDate,
                              mode: 'detail'
                            }, null, 8 /* PROPS */, ["time", "mode"])
                          ])
                        ]),
                        _: 1 /* STABLE */
                      }))
                      : _createCommentVNode("v-if", true),
                    (info.value)
                      ? (_openBlock(), _createBlock(MkKeyValue, {
                        key: 0,
                        oneline: ""
                      }, {
                        key: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.email), 1 /* TEXT */)
                        ]),
                        value: _withCtx(() => [
                          _createElementVNode("span", _hoisted_3, _toDisplayString(info.value.email), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }))
                      : _createCommentVNode("v-if", true)
                  ], 64 /* STABLE_FRAGMENT */))
                  : _createCommentVNode("v-if", true)
              ]),
              (!isSystem.value)
                ? (_openBlock(), _createBlock(MkTextarea, {
                  key: 0,
                  manualSave: "",
                  modelValue: moderationNote.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((moderationNote).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.moderationNote), 1 /* TEXT */)
                  ]),
                  caption: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.moderationNoteDescription), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]))
                : _createCommentVNode("v-if", true),
              _createTextVNode("\n\n\t\t\t"),
              _createTextVNode("\n\n\t\t\t"),
              (!isSystem.value)
                ? (_openBlock(), _createBlock(FormSection, { key: 0 }, {
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps" }, [
                      _createVNode(MkSwitch, {
                        "onUpdate:modelValue": [toggleSuspend, ($event: any) => ((suspended).value = $event)],
                        modelValue: suspended.value
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.suspend), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"]),
                      _createElementVNode("div", null, [
                        (user.value.host == null)
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            inline: "",
                            style: "margin-right: 8px;",
                            onClick: resetPassword
                          }, {
                            default: _withCtx(() => [
                              _hoisted_4,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.resetPassword), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _createVNode(MkFolder, null, {
                        icon: _withCtx(() => [
                          _hoisted_5
                        ]),
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts._role.policies), 1 /* TEXT */)
                        ]),
                        default: _withCtx(() => [
                          _createElementVNode("div", { class: "_gaps" }, [
                            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(Object.keys(info.value.policies), (policy) => {
                              return (_openBlock(), _createElementBlock("div", { key: policy }, _toDisplayString(policy) + " ... " + _toDisplayString(info.value.policies[policy]), 1 /* TEXT */))
                            }), 128 /* KEYED_FRAGMENT */))
                          ])
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(MkFolder, null, {
                        icon: _withCtx(() => [
                          _hoisted_6
                        ]),
                        label: _withCtx(() => [
                          _createTextVNode("IP")
                        ]),
                        default: _withCtx(() => [
                          (!_unref(iAmAdmin))
                            ? (_openBlock(), _createBlock(MkInfo, {
                              key: 0,
                              warn: ""
                            }, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.requireAdminForView), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }))
                            : (_openBlock(), _createBlock(MkInfo, { key: 1 }, {
                              default: _withCtx(() => [
                                _createTextVNode("The date is the IP address was first acknowledged.")
                              ]),
                              _: 1 /* STABLE */
                            })),
                          (_unref(iAmAdmin) && ips.value)
                            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(ips.value, (record) => {
                                return (_openBlock(), _createElementBlock("div", {
                                  key: record.ip,
                                  class: _normalizeClass(["_monospace", _ctx.$style.ip]),
                                  style: "margin: 1em 0;"
                                }, [
                                  _createElementVNode("span", _hoisted_7, _toDisplayString(record.createdAt), 1 /* TEXT */),
                                  _createElementVNode("span", _hoisted_8, _toDisplayString(record.ip), 1 /* TEXT */)
                                ]))
                              }), 128 /* KEYED_FRAGMENT */))
                            ], 64 /* STABLE_FRAGMENT */))
                            : _createCommentVNode("v-if", true)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createElementVNode("div", null, [
                        (_unref(iAmModerator))
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            inline: "",
                            danger: "",
                            style: "margin-right: 8px;",
                            onClick: unsetUserAvatar
                          }, {
                            default: _withCtx(() => [
                              _hoisted_9,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.unsetUserAvatar), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }))
                          : _createCommentVNode("v-if", true),
                        (_unref(iAmModerator))
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            inline: "",
                            danger: "",
                            onClick: unsetUserBanner
                          }, {
                            default: _withCtx(() => [
                              _hoisted_10,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.unsetUserBanner), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }))
                          : _createCommentVNode("v-if", true)
                      ]),
                      (_unref($i).isAdmin)
                        ? (_openBlock(), _createBlock(MkButton, {
                          key: 0,
                          inline: "",
                          danger: "",
                          onClick: deleteAccount
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.deleteAccount), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true)
            ]))
            : (tab.value === 'roles')
              ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                class: "_gaps"
              }, [
                (user.value.host == null)
                  ? (_openBlock(), _createBlock(MkButton, {
                    key: 0,
                    primary: "",
                    rounded: "",
                    onClick: assignRole
                  }, {
                    default: _withCtx(() => [
                      _hoisted_11,
                      _createTextVNode(" "),
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.assign), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }))
                  : _createCommentVNode("v-if", true),
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(info.value.roles, (role) => {
                  return (_openBlock(), _createElementBlock("div", { key: role.id }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.roleItemMain)
                    }, [
                      _createVNode(MkRolePreview, {
                        class: _normalizeClass(_ctx.$style.role),
                        role: role,
                        forModeration: true
                      }, null, 8 /* PROPS */, ["role", "forModeration"]),
                      _createElementVNode("button", {
                        class: "_button",
                        onClick: _cache[2] || (_cache[2] = ($event: any) => (toggleRoleItem(role)))
                      }, [
                        _hoisted_12
                      ]),
                      (role.target === 'manual')
                        ? (_openBlock(), _createElementBlock("button", {
                          key: 0,
                          class: _normalizeClass(["_button", _ctx.$style.roleUnassign]),
                          onClick: _cache[3] || (_cache[3] = ($event: any) => (unassignRole(role, $event)))
                        }, [
                          _hoisted_13
                        ]))
                        : (_openBlock(), _createElementBlock("button", {
                          key: 1,
                          class: _normalizeClass(["_button", _ctx.$style.roleUnassign]),
                          disabled: ""
                        }, [
                          _hoisted_14
                        ]))
                    ]),
                    (expandedRoleIds.value.includes(role.id))
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.roleItemSub)
                      }, [
                        _createElementVNode("div", null, [
                          _createTextVNode("Assigned: "),
                          _createVNode(_component_MkTime, {
                            time: info.value.roleAssigns.find((a) => a.roleId === role.id).createdAt,
                            mode: "detail"
                          }, null, 8 /* PROPS */, ["time"])
                        ]),
                        (info.value.roleAssigns.find((a) => a.roleId === role.id).expiresAt)
                          ? (_openBlock(), _createElementBlock("div", { key: 0 }, "Period: " + _toDisplayString(new Date(info.value.roleAssigns.find((a) => a.roleId === role.id).expiresAt).toLocaleString()), 1 /* TEXT */))
                          : (_openBlock(), _createElementBlock("div", { key: 1 }, "Period: " + _toDisplayString(_unref(i18n).ts.indefinitely), 1 /* TEXT */))
                      ]))
                      : _createCommentVNode("v-if", true)
                  ]))
                }), 128 /* KEYED_FRAGMENT */))
              ]))
            : (tab.value === 'announcements')
              ? (_openBlock(), _createElementBlock("div", {
                key: 2,
                class: "_gaps"
              }, [
                _createVNode(MkButton, {
                  primary: "",
                  rounded: "",
                  onClick: createAnnouncement
                }, {
                  default: _withCtx(() => [
                    _hoisted_15,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.createNew), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkSelect, {
                  items: _unref(announcementsStatusDef),
                  modelValue: _unref(announcementsStatus),
                  "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((announcementsStatus).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.filter), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["items", "modelValue"]),
                _createVNode(MkPagination, { paginator: _unref(announcementsPaginator) }, {
                  default: _withCtx(({ items }) => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (announcement) => {
                        return (_openBlock(), _createElementBlock("div", {
                          key: announcement.id,
                          class: _normalizeClass(_ctx.$style.announcementItem),
                          onClick: _cache[5] || (_cache[5] = ($event: any) => (editAnnouncement(announcement)))
                        }, [
                          ('icon' in announcement)
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 0,
                              style: "margin-right: 0.5em;"
                            }, [
                              (announcement.icon === 'info')
                                ? (_openBlock(), _createElementBlock("i", {
                                  key: 0,
                                  class: "ti ti-info-circle"
                                }))
                                : (announcement.icon === 'warning')
                                  ? (_openBlock(), _createElementBlock("i", {
                                    key: 1,
                                    class: "ti ti-alert-triangle",
                                    style: "color: var(--MI_THEME-warn);"
                                  }))
                                : (announcement.icon === 'error')
                                  ? (_openBlock(), _createElementBlock("i", {
                                    key: 2,
                                    class: "ti ti-circle-x",
                                    style: "color: var(--MI_THEME-error);"
                                  }))
                                : (announcement.icon === 'success')
                                  ? (_openBlock(), _createElementBlock("i", {
                                    key: 3,
                                    class: "ti ti-check",
                                    style: "color: var(--MI_THEME-success);"
                                  }))
                                : _createCommentVNode("v-if", true)
                            ]))
                            : _createCommentVNode("v-if", true),
                          _createElementVNode("span", null, _toDisplayString(announcement.title), 1 /* TEXT */),
                          (announcement.reads > 0)
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 0,
                              style: "margin-left: auto; opacity: 0.7;"
                            }, _toDisplayString(_unref(i18n).ts.messageRead), 1 /* TEXT */))
                            : _createCommentVNode("v-if", true)
                        ]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["paginator"])
              ]))
            : (tab.value === 'drive')
              ? (_openBlock(), _createElementBlock("div", {
                key: 3,
                class: "_gaps"
              }, [
                _createVNode(MkFileListForAdmin, {
                  paginator: _unref(filesPaginator),
                  viewMode: "grid"
                }, null, 8 /* PROPS */, ["paginator"])
              ]))
            : (tab.value === 'chart')
              ? (_openBlock(), _createElementBlock("div", {
                key: 4,
                class: "_gaps_m"
              }, [
                _createElementVNode("div", { class: "cmhjzshm" }, [
                  _createElementVNode("div", { class: "selects" }, [
                    _createVNode(MkSelect, {
                      items: _unref(chartSrcDef),
                      style: "margin: 0 10px 0 0; flex: 1;",
                      modelValue: _unref(chartSrc),
                      "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((chartSrc).value = $event))
                    }, null, 8 /* PROPS */, ["items", "modelValue"])
                  ]),
                  _createElementVNode("div", { class: "charts" }, [
                    _createElementVNode("div", _hoisted_16, _toDisplayString(_unref(i18n).tsx.recentNHours({ n: 90 })), 1 /* TEXT */),
                    _createVNode(MkChart, {
                      class: "chart",
                      src: _unref(chartSrc),
                      span: "hour",
                      limit: 90,
                      args: { user: user.value, withoutAll: true },
                      detailed: true
                    }, null, 8 /* PROPS */, ["src", "limit", "args", "detailed"]),
                    _createElementVNode("div", _hoisted_17, _toDisplayString(_unref(i18n).tsx.recentNDays({ n: 90 })), 1 /* TEXT */),
                    _createVNode(MkChart, {
                      class: "chart",
                      src: _unref(chartSrc),
                      span: "day",
                      limit: 90,
                      args: { user: user.value, withoutAll: true },
                      detailed: true
                    }, null, 8 /* PROPS */, ["src", "limit", "args", "detailed"])
                  ])
                ])
              ]))
            : (tab.value === 'raw')
              ? (_openBlock(), _createElementBlock("div", {
                key: 5,
                class: "_gaps_m"
              }, [
                (info.value && _unref($i).isAdmin)
                  ? (_openBlock(), _createBlock(MkObjectView, {
                    key: 0,
                    tall: "",
                    value: info.value
                  }, null, 8 /* PROPS */, ["value"]))
                  : _createCommentVNode("v-if", true),
                _createVNode(MkObjectView, {
                  tall: "",
                  value: user.value
                }, null, 8 /* PROPS */, ["value"])
              ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "tab"]))
}
}

})
