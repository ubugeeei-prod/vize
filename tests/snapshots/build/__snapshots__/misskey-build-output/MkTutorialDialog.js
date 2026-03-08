import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mood-smile" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-home" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil-plus" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-confetti", style: "display: block; margin: auto; font-size: 3em; color: var(--MI_THEME-accent);" })
const _hoisted_7 = { style: "font-size: 120%;" }
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check", style: "display: block; margin: auto; font-size: 3em; color: var(--MI_THEME-accent);" })
const _hoisted_20 = { style: "font-size: 120%;" }
const _hoisted_21 = { href: "https://misskey-hub.net/docs/for-users/", target: "_blank", class: "_link" }
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-left" })
import { ref, useTemplateRef, watch } from 'vue'
import { host } from '@@/js/config.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkButton from '@/components/MkButton.vue'
import XNote from '@/components/MkTutorialDialog.Note.vue'
import XTimeline from '@/components/MkTutorialDialog.Timeline.vue'
import XPostNote from '@/components/MkTutorialDialog.PostNote.vue'
import XSensitive from '@/components/MkTutorialDialog.Sensitive.vue'
import MkAnimBg from '@/components/MkAnimBg.vue'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { claimAchievement } from '@/utility/achievements.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkTutorialDialog',
  props: {
    initialPage: { type: Number, required: false }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const dialog = useTemplateRef('dialog');
// eslint-disable-next-line vue/no-setup-props-reactivity-loss
const page = ref(props.initialPage ?? 0);
watch(page, (to) => {
	// チュートリアルの枚数を増やしたら必ず変更すること！！
	if (to === 6) {
		claimAchievement('tutorialCompleted');
	}
});
const isReactionTutorialPushed = ref<boolean>(false);
const isSensitiveTutorialSucceeded = ref<boolean>(false);
async function close(skip: boolean) {
	if (skip) {
		const { canceled } = await os.confirm({
			type: 'warning',
			text: i18n.ts._initialTutorial.skipAreYouSure,
		});
		if (canceled) return;
	}
	dialog.value?.close();
}

return (_ctx: any,_cache: any) => {
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 600,
      height: 650,
      onClose: _cache[0] || (_cache[0] = ($event: any) => (close(true))),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (page.value === 1) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial._note.title), 1 /* TEXT */)
          ]),
          key: "0"
        } : (page.value === 2) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_2,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial._reaction.title), 1 /* TEXT */)
          ]),
          key: "1"
        } : (page.value === 3) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_3,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial._timeline.title), 1 /* TEXT */)
          ]),
          key: "2"
        } : (page.value === 4) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_4,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial._postNote.title), 1 /* TEXT */)
          ]),
          key: "3"
        } : (page.value === 5) ? {
          name: "header",
          fn: _withCtx(() => [
            _hoisted_5,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial._howToMakeAttachmentsSensitive.title), 1 /* TEXT */)
          ]),
          key: "4"
        } : {
        name: "header",
        fn: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._initialTutorial.title), 1 /* TEXT */)
        ]),
        key: "5"
      } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["width", "height"]))
}
}

})
