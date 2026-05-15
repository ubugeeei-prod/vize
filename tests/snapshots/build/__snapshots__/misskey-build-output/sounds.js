import { Fragment as _Fragment, Suspense as _Suspense, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-reload" });
import { computed, ref } from "vue";
import XSound from "./sounds.sound.vue";
import { prefer } from "@/preferences.js";
import MkRange from "@/components/MkRange.vue";
import MkButton from "@/components/MkButton.vue";
import FormSection from "@/components/form/section.vue";
import MkFolder from "@/components/MkFolder.vue";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import { operationTypes } from "@/utility/sound.js";
import MkSwitch from "@/components/MkSwitch.vue";
import MkPreferenceContainer from "@/components/MkPreferenceContainer.vue";
import MkFeatureBanner from "@/components/MkFeatureBanner.vue";
import { getInitialPrefValue } from "@/preferences/manager.js";
export default {
  __name: "sounds",
  setup(__props) {
    const notUseSound = prefer.model("sound.notUseSound");
    const useSoundOnlyWhenActive = prefer.model("sound.useSoundOnlyWhenActive");
    const masterVolume = prefer.model("sound.masterVolume");
    const sounds = ref({
      note: prefer.r["sound.on.note"],
      noteMy: prefer.r["sound.on.noteMy"],
      notification: prefer.r["sound.on.notification"],
      reaction: prefer.r["sound.on.reaction"],
      chatMessage: prefer.r["sound.on.chatMessage"]
    });
    function getSoundTypeName(f) {
      switch (f) {
        case null: return i18n.ts.none;
        case "_driveFile_": return i18n.ts._soundSettings.driveFile;
        default: return f;
      }
    }
    async function updated(type, sound) {
      const v = sound.type === "_driveFile_" ? {
        type: sound.type,
        fileId: sound.fileId,
        fileUrl: sound.fileUrl,
        volume: sound.volume
      } : {
        type: sound.type,
        volume: sound.volume
      };
      prefer.commit(`sound.on.${type}`, v);
      sounds.value[type] = v;
    }
    function reset() {
      for (const sound of Object.keys(sounds.value)) {
        const v = getInitialPrefValue(`sound.on.${sound}`);
        prefer.commit(`sound.on.${sound}`, v);
        sounds.value[sound] = v;
      }
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.sounds,
      icon: "ti ti-music"
    }));
    return (_ctx, _cache) => {
      const _component_SearchText = _resolveComponent("SearchText");
      const _component_SearchLabel = _resolveComponent("SearchLabel");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      const _component_MkLoading = _resolveComponent("MkLoading");
      return _openBlock(), _createBlock(_component_SearchMarker, {
        path: "/settings/sounds",
        label: _unref(i18n).ts.sounds,
        keywords: ["sounds"],
        icon: "ti ti-music"
      }, {
        default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/speaker_high_volume_3d.png",
            color: "#ff006f"
          }, {
            default: _withCtx(() => [_createVNode(_component_SearchText, null, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._settings.soundsBanner),
                1
                /* TEXT */
              )]),
              _: 1
            })]),
            _: 1
          }),
          _createVNode(_component_SearchMarker, { keywords: ["mute"] }, {
            default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "sound.notUseSound" }, {
              default: _withCtx(() => [_createVNode(MkSwitch, {
                modelValue: _unref(notUseSound),
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => notUseSound.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.notUseSound),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            })]),
            _: 1
          }, 8, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ["active", "mute"] }, {
            default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "sound.useSoundOnlyWhenActive" }, {
              default: _withCtx(() => [_createVNode(MkSwitch, {
                modelValue: _unref(useSoundOnlyWhenActive),
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => useSoundOnlyWhenActive.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.useSoundOnlyWhenActive),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            })]),
            _: 1
          }, 8, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ["volume", "master"] }, {
            default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "sound.masterVolume" }, {
              default: _withCtx(() => [_createVNode(MkRange, {
                min: 0,
                max: 1,
                step: .05,
                textConverter: (v) => `${Math.floor(v * 100)}%`,
                modelValue: _unref(masterVolume),
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => masterVolume.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.masterVolume),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, [
                "min",
                "max",
                "step",
                "textConverter",
                "modelValue"
              ])]),
              _: 1
            })]),
            _: 1
          }, 8, ["keywords"]),
          _createVNode(FormSection, null, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.sounds),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(_unref(operationTypes), (type) => {
                return _openBlock(), _createBlock(
                  MkFolder,
                  { key: type },
                  {
                    label: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts._sfx[type]),
                      1
                      /* TEXT */
                    )]),
                    suffix: _withCtx(() => [_createTextVNode(
                      _toDisplayString(getSoundTypeName(sounds.value[type].type)),
                      1
                      /* TEXT */
                    )]),
                    default: _withCtx(() => [_createVNode(_Suspense, null, {
                      default: _withCtx(() => [_createVNode(XSound, {
                        def: sounds.value[type],
                        onUpdate: (res) => updated(type, res)
                      }, null, 8, ["def", "onUpdate"])]),
                      fallback: _withCtx(() => [_createVNode(_component_MkLoading)]),
                      _: 2
                    })]),
                    _: 2
                  },
                  1024
                  /* DYNAMIC_SLOTS */
                );
              }),
              128
              /* KEYED_FRAGMENT */
            ))])]),
            _: 1
          }),
          _createVNode(MkButton, {
            danger: "",
            onClick: _cache[3] || (_cache[3] = ($event) => reset())
          }, {
            default: _withCtx(() => [
              _hoisted_1,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.default),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })
        ])]),
        _: 1
      }, 8, ["label", "keywords"]);
    };
  }
};
