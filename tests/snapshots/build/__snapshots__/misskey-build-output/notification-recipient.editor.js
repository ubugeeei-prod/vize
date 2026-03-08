import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { computed, onMounted, ref, useTemplateRef, toRefs } from 'vue'
import { entities } from 'misskey-js'
import type { MkSystemWebhookResult } from '@/components/MkSystemWebhookEditor.impl.js'
import MkButton from '@/components/MkButton.vue'
import MkModalWindow from '@/components/MkModalWindow.vue'
import { i18n } from '@/i18n.js'
import MkInput from '@/components/MkInput.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import MkSelect from '@/components/MkSelect.vue'
import { showSystemWebhookEditorDialog } from '@/components/MkSystemWebhookEditor.impl.js'
import MkSwitch from '@/components/MkSwitch.vue'
import MkDivider from '@/components/MkDivider.vue'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'notification-recipient.editor',
  props: {
    mode: { type: String, required: true },
    id: { type: String, required: false }
  },
  emits: ["submitted", "canceled", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { mode, id } = toRefs(props);
const dialogEl = useTemplateRef('dialogEl');
const loading = ref<number>(0);
const title = ref<string>('');
const {
	model: method,
	def: methodDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts._abuseReport._notificationRecipient._recipientType.mail, value: 'email' },
		{ label: i18n.ts._abuseReport._notificationRecipient._recipientType.webhook, value: 'webhook' },
	],
	initialValue: 'email',
});
const {
	model: userId,
	def: userIdDef,
} = useMkSelect({
	items: computed(() => moderators.value.map(u => ({ label: u.name ? `${u.name}(${u.username})` : u.username, value: u.id as string | null }))),
});
const {
	model: systemWebhookId,
	def: systemWebhookIdDef,
} = useMkSelect({
	items: computed(() => systemWebhooks.value.map(w => ({ label: w.name, value: w.id }))),
});
const isActive = ref<boolean>(true);
const moderators = ref<entities.User[]>([]);
const systemWebhooks = ref<(entities.SystemWebhook | { id: null, name: string })[]>([]);
const methodCaption = computed(() => {
	switch (method.value) {
		case 'email': {
			return i18n.ts._abuseReport._notificationRecipient._recipientType._captions.mail;
		}
		case 'webhook': {
			return i18n.ts._abuseReport._notificationRecipient._recipientType._captions.webhook;
		}
		default: {
			return '';
		}
	}
});
const disableSubmitButton = computed(() => {
	if (!title.value) {
		return true;
	}

	switch (method.value) {
		case 'email': {
			return userId.value === null;
		}
		case 'webhook': {
			return systemWebhookId.value === null;
		}
		default: {
			return true;
		}
	}
});
async function onSubmitClicked() {
	await loadingScope(async () => {
		const _userId = (method.value === 'email') ? userId.value : null;
		const _systemWebhookId = (method.value === 'webhook') ? systemWebhookId.value : null;
		const params = {
			isActive: isActive.value,
			name: title.value,
			method: method.value,
			userId: _userId ?? undefined,
			systemWebhookId: _systemWebhookId ?? undefined,
		};
		try {
			switch (mode.value) {
				case 'create': {
					await misskeyApi('admin/abuse-report/notification-recipient/create', params);
					break;
				}
				case 'edit': {
					// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
					await misskeyApi('admin/abuse-report/notification-recipient/update', { id: id.value!, ...params });
					break;
				}
			}
			dialogEl.value?.close();
			emit('submitted');
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
		} catch (ex: any) {
			const msg = ex.message ?? i18n.ts.internalServerErrorDescription;
			await os.alert({ type: 'error', title: i18n.ts.error, text: msg });
			dialogEl.value?.close();
			emit('canceled');
		}
	});
}
function onCancelClicked() {
	dialogEl.value?.close();
	emit('canceled');
}
async function onEditSystemWebhookClicked() {
	let result: MkSystemWebhookResult | null;
	if (systemWebhookId.value === null) {
		result = await showSystemWebhookEditorDialog({
			mode: 'create',
		});
	} else {
		result = await showSystemWebhookEditorDialog({
			mode: 'edit',
			id: systemWebhookId.value,
		});
	}
	if (!result) {
		return;
	}
	await fetchSystemWebhooks();
	systemWebhookId.value = result.id ?? null;
}
async function fetchSystemWebhooks() {
	await loadingScope(async () => {
		systemWebhooks.value = [
			{ id: null, name: i18n.ts.createNew },
			...await misskeyApi('admin/system-webhook/list', { }),
		];
	});
}
async function fetchModerators() {
	await loadingScope(async () => {
		const users = Array.of<entities.User>();
		for (; ;) {
			const res = await misskeyApi('admin/show-users', {
				limit: 100,
				state: 'adminOrModerator',
				origin: 'local',
				offset: users.length,
			});
			if (res.length === 0) {
				break;
			}
			users.push(...res);
		}
		moderators.value = users;
	});
}
async function loadingScope<T>(fn: () => Promise<T>): Promise<T> {
	loading.value++;
	try {
		return await fn();
	} finally {
		loading.value--;
	}
}
onMounted(async () => {
	await loadingScope(async () => {
		await fetchModerators();
		await fetchSystemWebhooks();
		if (mode.value === 'edit') {
			if (!id.value) {
				throw new Error('id is required');
			}
			try {
				const res = await misskeyApi('admin/abuse-report/notification-recipient/show', { id: id.value });
				title.value = res.name;
				method.value = res.method;
				userId.value = res.userId ?? null;
				systemWebhookId.value = res.systemWebhookId ?? null;
				isActive.value = res.isActive;
				// eslint-disable-next-line
			} catch (ex: any) {
				const msg = ex.message ?? i18n.ts.internalServerErrorDescription;
				await os.alert({ type: 'error', title: i18n.ts.error, text: msg });
				dialogEl.value?.close();
				emit('canceled');
			}
		} else {
			userId.value = moderators.value[0]?.id ?? null;
			systemWebhookId.value = systemWebhooks.value[0]?.id ?? null;
		}
	});
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialogEl", ref: dialogEl,
      width: 400,
      height: 490,
      withOkButton: false,
      okButtonDisabled: false,
      onClose: onCancelClicked,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(mode) === 'create' ? _unref(i18n).ts._abuseReport._notificationRecipient.createRecipient : _unref(i18n).ts._abuseReport._notificationRecipient.modifyRecipient), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        (loading.value === 0)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            style: "display: flex; flex-direction: column; min-height: 100%;"
          }, [
            _createElementVNode("div", {
              class: "_spacer",
              style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px; flex-grow: 1;"
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(["_gaps_m", _ctx.$style.root])
              }, [
                _createVNode(MkInput, {
                  modelValue: title.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((title).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.title), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                _createVNode(MkSelect, {
                  items: _unref(methodDef),
                  modelValue: _unref(method),
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((method).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseReport._notificationRecipient.recipientType), 1 /* TEXT */)
                  ]),
                  caption: _withCtx(() => [
                    _createTextVNode(_toDisplayString(methodCaption.value), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["items", "modelValue"]),
                _createElementVNode("div", null, [
                  (_unref(method) === 'email')
                    ? (_openBlock(), _createBlock(MkSelect, {
                      key: 0,
                      items: _unref(userIdDef),
                      modelValue: _unref(userId),
                      "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((userId).value = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseReport._notificationRecipient.notifiedUser), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["items", "modelValue"]))
                    : (_unref(method) === 'webhook')
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 1,
                        class: _normalizeClass(_ctx.$style.systemWebhook)
                      }, [
                        _createVNode(MkSelect, {
                          items: _unref(systemWebhookIdDef),
                          style: "flex: 1",
                          modelValue: _unref(systemWebhookId),
                          "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((systemWebhookId).value = $event))
                        }, {
                          label: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseReport._notificationRecipient.notifiedWebhook), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["items", "modelValue"]),
                        _createVNode(MkButton, {
                          rounded: "",
                          class: _normalizeClass(_ctx.$style.systemWebhookEditButton),
                          onClick: onEditSystemWebhookClicked
                        }, {
                          default: _withCtx(() => [
                            (_unref(systemWebhookId) === null)
                              ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                class: "ti ti-plus",
                                style: "line-height: normal"
                              }))
                              : (_openBlock(), _createElementBlock("span", {
                                key: 1,
                                class: "ti ti-settings",
                                style: "line-height: normal"
                              }))
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]))
                    : _createCommentVNode("v-if", true)
                ]),
                _createVNode(MkDivider),
                _createVNode(MkSwitch, {
                  modelValue: isActive.value,
                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((isActive).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.enable), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"])
              ])
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(["_buttonsCenter", _ctx.$style.footer])
            }, [
              _createVNode(MkButton, {
                primary: "",
                rounded: "",
                disabled: disableSubmitButton.value,
                onClick: onSubmitClicked
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.ok), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["disabled"]),
              _createVNode(MkButton, {
                rounded: "",
                onClick: onCancelClicked
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.cancel), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          ]))
          : (_openBlock(), _createElementBlock("div", { key: 1 }, [
            _createVNode(_component_MkLoading)
          ]))
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "height", "withOkButton", "okButtonDisabled"]))
}
}

})
