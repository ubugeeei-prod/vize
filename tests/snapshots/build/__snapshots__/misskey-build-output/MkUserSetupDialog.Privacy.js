import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-off" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world-x" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo-shield" })
import { ref, watch } from 'vue'
import { i18n } from '@/i18n.js'
import MkSwitch from '@/components/MkSwitch.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkFolder from '@/components/MkFolder.vue'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserSetupDialog.Privacy',
  setup(__props) {

const isLocked = ref(false);
const hideOnlineStatus = ref(false);
const noCrawle = ref(false);
const preventAiLearning = ref(true);
watch([isLocked, hideOnlineStatus, noCrawle, preventAiLearning], () => {
	misskeyApi('i/update', {
		isLocked: !!isLocked.value,
		hideOnlineStatus: !!hideOnlineStatus.value,
		noCrawle: !!noCrawle.value,
		preventAiLearning: !!preventAiLearning.value,
	});
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createVNode(MkInfo, null, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._initialAccountSetting.theseSettingsCanEditLater), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkFolder, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.makeFollowManuallyApprove), 1 /* TEXT */)
        ]),
        icon: _withCtx(() => [
          _hoisted_1
        ]),
        suffix: _withCtx(() => [
          _createTextVNode(_toDisplayString(isLocked.value ? _unref(i18n).ts.on : _unref(i18n).ts.off), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(MkSwitch, {
            modelValue: isLocked.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((isLocked).value = $event))
          }, {
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.lockedAccountInfo), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.makeFollowManuallyApprove), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkFolder, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.hideOnlineStatus), 1 /* TEXT */)
        ]),
        icon: _withCtx(() => [
          _hoisted_2
        ]),
        suffix: _withCtx(() => [
          _createTextVNode(_toDisplayString(hideOnlineStatus.value ? _unref(i18n).ts.on : _unref(i18n).ts.off), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(MkSwitch, {
            modelValue: hideOnlineStatus.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((hideOnlineStatus).value = $event))
          }, {
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.hideOnlineStatusDescription), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.hideOnlineStatus), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkFolder, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.noCrawle), 1 /* TEXT */)
        ]),
        icon: _withCtx(() => [
          _hoisted_3
        ]),
        suffix: _withCtx(() => [
          _createTextVNode(_toDisplayString(noCrawle.value ? _unref(i18n).ts.on : _unref(i18n).ts.off), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(MkSwitch, {
            modelValue: noCrawle.value,
            "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((noCrawle).value = $event))
          }, {
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.noCrawleDescription), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.noCrawle), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkFolder, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.preventAiLearning), 1 /* TEXT */)
        ]),
        icon: _withCtx(() => [
          _hoisted_4
        ]),
        suffix: _withCtx(() => [
          _createTextVNode(_toDisplayString(preventAiLearning.value ? _unref(i18n).ts.on : _unref(i18n).ts.off), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(MkSwitch, {
            modelValue: preventAiLearning.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((preventAiLearning).value = $event))
          }, {
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.preventAiLearningDescription), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.preventAiLearning), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkInfo, null, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._initialAccountSetting.youCanEditMoreSettingsInSettingsPageLater), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
