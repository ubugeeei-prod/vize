import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Suspense as _Suspense, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-reload" })
import { computed, ref } from 'vue'
import XSound from './sounds.sound.vue'
import type { Ref } from 'vue'
import type { SoundType, OperationType } from '@/utility/sound.js'
import type { SoundStore } from '@/preferences/def.js'
import { prefer } from '@/preferences.js'
import MkRange from '@/components/MkRange.vue'
import MkButton from '@/components/MkButton.vue'
import FormSection from '@/components/form/section.vue'
import MkFolder from '@/components/MkFolder.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { operationTypes } from '@/utility/sound.js'
import MkSwitch from '@/components/MkSwitch.vue'
import MkPreferenceContainer from '@/components/MkPreferenceContainer.vue'
import { PREF_DEF } from '@/preferences/def.js'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'
import { getInitialPrefValue } from '@/preferences/manager.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'sounds',
  setup(__props) {

const notUseSound = prefer.model('sound.notUseSound');
const useSoundOnlyWhenActive = prefer.model('sound.useSoundOnlyWhenActive');
const masterVolume = prefer.model('sound.masterVolume');
const sounds = ref<Record<OperationType, Ref<SoundStore>>>({
	note: prefer.r['sound.on.note'],
	noteMy: prefer.r['sound.on.noteMy'],
	notification: prefer.r['sound.on.notification'],
	reaction: prefer.r['sound.on.reaction'],
	chatMessage: prefer.r['sound.on.chatMessage'],
});
function getSoundTypeName(f: SoundType): string {
	switch (f) {
		case null:
			return i18n.ts.none;
		case '_driveFile_':
			return i18n.ts._soundSettings.driveFile;
		default:
			return f;
	}
}
async function updated(type: keyof typeof sounds.value, sound: { type: SoundType; fileId?: string; fileUrl?: string; volume: number; }) {
	const v: SoundStore = sound.type === '_driveFile_' ? {
		type: sound.type,
		fileId: sound.fileId!,
		fileUrl: sound.fileUrl!,
		volume: sound.volume,
	} : {
		type: sound.type,
		volume: sound.volume,
	};
	prefer.commit(`sound.on.${type}`, v);
	sounds.value[type] = v;
}
function reset() {
	for (const sound of Object.keys(sounds.value) as Array<keyof typeof sounds.value>) {
		const v = getInitialPrefValue(`sound.on.${sound}`);
		prefer.commit(`sound.on.${sound}`, v);
		sounds.value[sound] = v;
	}
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.sounds,
	icon: 'ti ti-music',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/sounds",
      label: _unref(i18n).ts.sounds,
      keywords: ['sounds'],
      icon: "ti ti-music"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/speaker_high_volume_3d.png",
            color: "#ff006f"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.soundsBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(_component_SearchMarker, { keywords: ['mute'] }, {
            default: _withCtx(() => [
              _createVNode(MkPreferenceContainer, { k: "sound.notUseSound" }, {
                default: _withCtx(() => [
                  _createVNode(MkSwitch, {
                    modelValue: _unref(notUseSound),
                    "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((notUseSound).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.notUseSound), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['active', 'mute'] }, {
            default: _withCtx(() => [
              _createVNode(MkPreferenceContainer, { k: "sound.useSoundOnlyWhenActive" }, {
                default: _withCtx(() => [
                  _createVNode(MkSwitch, {
                    modelValue: _unref(useSoundOnlyWhenActive),
                    "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((useSoundOnlyWhenActive).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.useSoundOnlyWhenActive), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['volume', 'master'] }, {
            default: _withCtx(() => [
              _createVNode(MkPreferenceContainer, { k: "sound.masterVolume" }, {
                default: _withCtx(() => [
                  _createVNode(MkRange, {
                    min: 0,
                    max: 1,
                    step: 0.05,
                    textConverter: (v) => `${Math.floor(v * 100)}%`,
                    modelValue: _unref(masterVolume),
                    "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((masterVolume).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.masterVolume), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(FormSection, null, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.sounds), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_s" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(operationTypes), (type) => {
                  return (_openBlock(), _createBlock(MkFolder, { key: type }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._sfx[type]), 1 /* TEXT */)
                    ]),
                    suffix: _withCtx(() => [
                      _createTextVNode(_toDisplayString(getSoundTypeName(sounds.value[type].type)), 1 /* TEXT */)
                    ]),
                    default: _withCtx(() => [
                      _createVNode(_Suspense, null, {
                        default: _withCtx(() => [
                          _createVNode(XSound, {
                            def: sounds.value[type],
                            onUpdate: _cache[3] || (_cache[3] = (res) => updated(type, res))
                          }, null, 8 /* PROPS */, ["def"])
                        ]),
                        fallback: _withCtx(() => [
                          _createVNode(_component_MkLoading)
                        ]),
                        _: 2 /* DYNAMIC */
                      })
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 1024 /* DYNAMIC_SLOTS */))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            danger: "",
            onClick: _cache[4] || (_cache[4] = ($event: any) => (reset()))
          }, {
            default: _withCtx(() => [
              _hoisted_1,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.default), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
