import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, watch } from 'vue'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import FormSlot from '@/components/form/slot.vue'
import MkInfo from '@/components/MkInfo.vue'
import * as os from '@/os.js'
import { ensureSignin } from '@/i.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserSetupDialog.Profile',
  setup(__props) {

const $i = ensureSignin();
const name = ref($i.name ?? '');
const description = ref($i.description ?? '');
watch(name, () => {
	os.apiWithDialog('i/update', {
		// 空文字列をnullにしたいので??は使うな
		name: name.value || null,
	}, undefined, {
		'0b3f9f6a-2f4d-4b1f-9fb4-49d3a2fd7191': {
			title: i18n.ts.yourNameContainsProhibitedWords,
			text: i18n.ts.yourNameContainsProhibitedWordsDescription,
		},
	});
});
watch(description, () => {
	os.apiWithDialog('i/update', {
		// 空文字列をnullにしたいので??は使うな
		description: description.value || null,
	});
});
async function setAvatar(ev: PointerEvent) {
	const files = await os.chooseFileFromPc({ multiple: false });
	const file = files[0];
	let originalOrCropped = file;
	const { canceled } = await os.confirm({
		type: 'question',
		text: i18n.ts.cropImageAsk,
		okText: i18n.ts.cropYes,
		cancelText: i18n.ts.cropNo,
	});
	if (!canceled) {
		originalOrCropped = await os.cropImageFile(file, {
			aspectRatio: 1,
		});
	}
	const driveFile = (await os.launchUploader([originalOrCropped], { multiple: false }))[0];
	const i = await os.apiWithDialog('i/update', {
		avatarId: driveFile.id,
	});
	$i.avatarId = i.avatarId;
	$i.avatarUrl = i.avatarUrl;
}

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _directive_adaptive_bg = _resolveDirective("adaptive-bg")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createVNode(MkInfo, null, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._initialAccountSetting.theseSettingsCanEditLater), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), _createVNode(FormSlot, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.avatar), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", {
            class: _normalizeClass(["_panel", _ctx.$style.avatarSection])
          }, [
            _createVNode(_component_MkAvatar, {
              class: _normalizeClass(_ctx.$style.avatar),
              user: _unref($i),
              onClick: setAvatar
            }, null, 8 /* PROPS */, ["user"]),
            _createElementVNode("div", { style: "margin-top: 16px;" }, [
              _createVNode(MkButton, {
                primary: "",
                rounded: "",
                inline: "",
                onClick: setAvatar
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.changeAvatar), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          ])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkInput, {
        max: 30,
        manualSave: "",
        "data-cy-user-setup-user-name": "",
        modelValue: name.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((name).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.name), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["max", "modelValue"]), _createVNode(MkTextarea, {
        max: 500,
        tall: "",
        manualSave: "",
        "data-cy-user-setup-user-description": "",
        modelValue: description.value,
        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((description).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.description), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["max", "modelValue"]), _createVNode(MkInfo, null, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._initialAccountSetting.youCanEditMoreSettingsInSettingsPageLater), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
