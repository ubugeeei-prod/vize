import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "ti ti-plus" })
import { entities } from 'misskey-js'
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'
import XRecipient from './notification-recipient.item.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkInput from '@/components/MkInput.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import MkDivider from '@/components/MkDivider.vue'
import { i18n } from '@/i18n.js'
import { useMkSelect } from '@/composables/use-mkselect.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'notification-recipient',
  setup(__props) {

const recipients = ref<entities.AbuseReportNotificationRecipient[]>([]);
const {
	model: filterMethod,
	def: filterMethodDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: null },
		{ label: i18n.ts._abuseReport._notificationRecipient._recipientType.mail, value: 'email' },
		{ label: i18n.ts._abuseReport._notificationRecipient._recipientType.webhook, value: 'webhook' },
	],
	initialValue: null,
});
const filterText = ref<string>('');
const filteredRecipients = computed(() => {
	const method = filterMethod.value;
	const text = filterText.value.trim().length === 0 ? null : filterText.value;

	return recipients.value.filter(it => {
		if (method ?? text) {
			if (text) {
				const keywords = [it.name, it.systemWebhook?.name, it.user?.name, it.user?.username];
				if (keywords.filter(k => k?.includes(text)).length !== 0) {
					return true;
				}
			}

			if (method) {
				return it.method.includes(method);
			}

			return false;
		}

		return true;
	});
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
async function onAddButtonClicked() {
	await showEditor('create');
}
async function onEditButtonClicked(id: string) {
	await showEditor('edit', id);
}
async function onDeleteButtonClicked(id: string) {
	const res = await os.confirm({
		type: 'warning',
		title: i18n.ts._abuseReport._notificationRecipient.deleteConfirm,
	});
	if (!res.canceled) {
		await misskeyApi('admin/abuse-report/notification-recipient/delete', { id: id });
		await fetchRecipients();
	}
}
async function showEditor(mode: 'create' | 'edit', id?: string) {
	const { needLoad } = await new Promise<{ needLoad: boolean }>(async resolve => {
		const { dispose } = os.popup(
			defineAsyncComponent(() => import('./notification-recipient.editor.vue')),
			{
				mode,
				id,
			},
			{
				submitted: () => {
					resolve({ needLoad: true });
				},
				canceled: () => {
					resolve({ needLoad: false });
				},
				closed: () => {
					dispose();
				},
			},
		);
	});
	if (needLoad) {
		await fetchRecipients();
	}
}
async function fetchRecipients() {
	const result = await misskeyApi('admin/abuse-report/notification-recipient/list', {
		method: ['email', 'webhook'],
	});
	recipients.value = result.sort((a, b) => (a.method + a.id).localeCompare(b.method + b.id));
}
onMounted(async () => {
	await fetchRecipients();
});

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 900px;"
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(["_gaps_m", _ctx.$style.root])
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.addButton)
            }, [
              _createVNode(MkButton, {
                primary: "",
                onClick: onAddButtonClicked
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseReport._notificationRecipient.createRecipient), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(["_gaps_s", _ctx.$style.subMenus])
            }, [
              _createVNode(MkSelect, {
                items: _unref(filterMethodDef),
                style: "flex: 1",
                modelValue: _unref(filterMethod),
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((filterMethod).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseReport._notificationRecipient.recipientType), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"]),
              _createVNode(MkInput, {
                type: "search",
                style: "flex: 1",
                modelValue: filterText.value,
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((filterText).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseReport._notificationRecipient.keywords), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _createVNode(MkDivider),
            _createElementVNode("div", {
              class: _normalizeClass(["_gaps_s", _ctx.$style.recipients])
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(filteredRecipients.value, (r) => {
                return (_openBlock(), _createBlock(XRecipient, {
                  key: r.id,
                  entity: r,
                  onEdit: onEditButtonClicked,
                  onDelete: onDeleteButtonClicked
                }, null, 8 /* PROPS */, ["entity"]))
              }), 128 /* KEYED_FRAGMENT */))
            ])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
