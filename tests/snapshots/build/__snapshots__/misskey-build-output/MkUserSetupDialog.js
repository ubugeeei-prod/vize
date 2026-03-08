import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-edit" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-plus" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bell-plus" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-confetti", style: "display: block; margin: auto; font-size: 3em; color: var(--MI_THEME-accent);" })
const _hoisted_6 = { style: "font-size: 120%;" }
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bell-ringing-2", style: "display: block; margin: auto; font-size: 3em; color: var(--MI_THEME-accent);" })
const _hoisted_15 = { style: "font-size: 120%;" }
const _hoisted_16 = { style: "padding: 0 16px;" }
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check", style: "display: block; margin: auto; font-size: 3em; color: var(--MI_THEME-accent);" })
const _hoisted_20 = { style: "font-size: 120%;" }
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
import { ref, useTemplateRef, watch, nextTick, defineAsyncComponent } from 'vue'
import { host } from '@@/js/config.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkButton from '@/components/MkButton.vue'
import XProfile from '@/components/MkUserSetupDialog.Profile.vue'
import XFollow from '@/components/MkUserSetupDialog.Follow.vue'
import XPrivacy from '@/components/MkUserSetupDialog.Privacy.vue'
import MkAnimBg from '@/components/MkAnimBg.vue'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import MkPushNotificationAllowButton from '@/components/MkPushNotificationAllowButton.vue'
import { store } from '@/store.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserSetupDialog',
  emits: ["closed"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const dialog = useTemplateRef('dialog');
const page = ref(store.s.accountSetupWizard);
watch(page, () => {
	store.set('accountSetupWizard', page.value);
});
async function close(skip: boolean) {
	if (skip) {
		const { canceled } = await os.confirm({
			type: 'warning',
			text: i18n.ts._initialAccountSetting.skipAreYouSure,
		});
		if (canceled) return;
	}
	dialog.value?.close();
	store.set('accountSetupWizard', -1);
}
function setupComplete() {
	store.set('accountSetupWizard', -1);
	dialog.value?.close();
}
function launchTutorial() {
	setupComplete();
	nextTick(async () => {
		const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkTutorialDialog.vue').then(x => x.default), {
			initialPage: 1,
		}, {
			closed: () => dispose(),
		});
	});
}
async function later(later: boolean) {
	if (later) {
		const { canceled } = await os.confirm({
			type: 'warning',
			text: i18n.ts._initialAccountSetting.laterAreYouSure,
		});
		if (canceled) return;
	}
	dialog.value?.close();
	store.set('accountSetupWizard', 0);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 500,
      height: 550,
      "data-cy-user-setup": "",
      onClose: _cache[0] || (_cache[0] = ($event: any) => (close(true))),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (page.value === 1) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts._initialAccountSetting.profileSetting), 1 /* TEXT */)
          ]),
          key: "0"
        } : (page.value === 2) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_2,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts._initialAccountSetting.privacySetting), 1 /* TEXT */)
          ]),
          key: "1"
        } : (page.value === 3) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_3,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.follow), 1 /* TEXT */)
          ]),
          key: "2"
        } : (page.value === 4) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_4,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.pushNotification), 1 /* TEXT */)
          ]),
          key: "3"
        } : (page.value === 5) ? {
          name: "header",
          fn: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.done), 1 /* TEXT */)
          ]),
          key: "4"
        } : {
        name: "header",
        fn: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.initialAccountSetting), 1 /* TEXT */)
        ]),
        key: "5"
      } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["width", "height"]))
}
}

})
