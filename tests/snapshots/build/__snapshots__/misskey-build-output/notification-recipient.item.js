import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "ti ti-settings" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "ti ti-trash" })
import { entities } from 'misskey-js'
import { computed, toRefs } from 'vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'notification-recipient.item',
  props: {
    entity: { type: null, required: true }
  },
  emits: ["edit", "id", "delete", "id"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { entity } = toRefs(props);
const method = computed(() => entity.value.method);
const user = computed(() => entity.value.user);
const systemWebhook = computed(() => entity.value.systemWebhook);
const methodIcon = computed(() => {
	switch (entity.value.method) {
		case 'email':
			return 'ti-mail';
		case 'webhook':
			return 'ti-webhook';
		default:
			return 'ti-help';
	}
});
const methodName = computed(() => {
	switch (entity.value.method) {
		case 'email':
			return i18n.ts._abuseReport._notificationRecipient._recipientType.mail;
		case 'webhook':
			return i18n.ts._abuseReport._notificationRecipient._recipientType.webhook;
		default:
			return '不明';
	}
});
function onEditButtonClicked() {
	emit('edit', entity.value.id);
}
function onDeleteButtonClicked() {
	emit('delete', entity.value.id);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_panel _gaps_s", _ctx.$style.root])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.rightDivider),
        style: "width: 80px;"
      }, [ _createElementVNode("span", {
          class: _normalizeClass(`ti ${methodIcon.value}`)
        }, null, 2 /* CLASS */), _createTextVNode(" " + _toDisplayString(methodName.value), 1 /* TEXT */) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.rightDivider),
        style: "flex: 0.5"
      }, _toDisplayString(_unref(entity).name), 1 /* TEXT */), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.rightDivider),
        style: "flex: 1"
      }, [ (method.value === 'email' && user.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(`${_unref(i18n).ts._abuseReport._notificationRecipient.notifiedUser}: ` + ((user.value.name) ? `${user.value.name}(${user.value.username})` : user.value.username)), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (method.value === 'webhook' && systemWebhook.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(`${_unref(i18n).ts._abuseReport._notificationRecipient.notifiedWebhook}: ` + systemWebhook.value.name), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.recipientButtons),
        style: "margin-left: auto"
      }, [ _createElementVNode("button", {
          class: _normalizeClass(_ctx.$style.recipientButton),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (onEditButtonClicked()))
        }, [ _hoisted_1 ]), _createElementVNode("button", {
          class: _normalizeClass(_ctx.$style.recipientButton),
          onClick: _cache[1] || (_cache[1] = ($event: any) => (onDeleteButtonClicked()))
        }, [ _hoisted_2 ]) ]) ]))
}
}

})
