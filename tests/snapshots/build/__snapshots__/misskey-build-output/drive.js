import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-folder" })
const _hoisted_2 = { class: "_beta" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-copyright" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cloud-cog" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-ipad-horizontal" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cloud-cog" })
import { computed, defineAsyncComponent, ref } from 'vue'
import * as Misskey from 'misskey-js'
import tinycolor from 'tinycolor2'
import XWatermarkItem from './drive.WatermarkItem.vue'
import XImageFrameItem from './drive.ImageFrameItem.vue'
import type { WatermarkPreset } from '@/utility/watermark/WatermarkRenderer.js'
import type { ImageFramePreset } from '@/utility/image-frame-renderer/ImageFrameRenderer.js'
import FormLink from '@/components/form/link.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkSelect from '@/components/MkSelect.vue'
import FormSection from '@/components/form/section.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import FormSplit from '@/components/form/split.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import bytes from '@/filters/bytes.js'
import MkChart from '@/components/MkChart.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { ensureSignin } from '@/i.js'
import { prefer } from '@/preferences.js'
import MkPreferenceContainer from '@/components/MkPreferenceContainer.vue'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'
import { selectDriveFolder } from '@/utility/drive.js'
import MkFolder from '@/components/MkFolder.vue'
import MkButton from '@/components/MkButton.vue'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'drive',
  setup(__props) {

const $i = ensureSignin();
const fetching = ref(true);
const usage = ref<number | null>(null);
const capacity = ref<number | null>(null);
const uploadFolder = ref<Misskey.entities.DriveFolder | null>(null);
const alwaysMarkNsfw = ref($i.alwaysMarkNsfw);
const autoSensitive = ref($i.autoSensitive);
const meterStyle = computed(() => {
	if (!capacity.value || !usage.value) return {};
	return {
		width: `${usage.value / capacity.value * 100}%`,
		background: tinycolor({
			h: 180 - (usage.value / capacity.value * 180),
			s: 0.7,
			l: 0.5,
		}).toHslString(),
	};
});
const keepOriginalFilename = prefer.model('keepOriginalFilename');
const defaultWatermarkPresetId = prefer.model('defaultWatermarkPresetId');
const defaultImageCompressionLevel = prefer.model('defaultImageCompressionLevel');
const defaultVideoCompressionLevel = prefer.model('defaultVideoCompressionLevel');
const watermarkPresetsSyncEnabled = ref(prefer.isSyncEnabled('watermarkPresets'));
function changeWatermarkPresetsSyncEnabled(value: boolean) {
	if (value) {
		prefer.enableSync('watermarkPresets').then((res) => {
			if (res == null) return;
			if (res.enabled) watermarkPresetsSyncEnabled.value = true;
		});
	} else {
		prefer.disableSync('watermarkPresets');
		watermarkPresetsSyncEnabled.value = false;
	}
}
const imageFramePresetsSyncEnabled = ref(prefer.isSyncEnabled('imageFramePresets'));
function changeImageFramePresetsSyncEnabled(value: boolean) {
	if (value) {
		prefer.enableSync('imageFramePresets').then((res) => {
			if (res == null) return;
			if (res.enabled) imageFramePresetsSyncEnabled.value = true;
		});
	} else {
		prefer.disableSync('imageFramePresets');
		imageFramePresetsSyncEnabled.value = false;
	}
}
misskeyApi('drive').then(info => {
	capacity.value = info.capacity;
	usage.value = info.usage;
	fetching.value = false;
});
if (prefer.s.uploadFolder) {
	misskeyApi('drive/folders/show', {
		folderId: prefer.s.uploadFolder,
	}).then(response => {
		uploadFolder.value = response;
	});
}
function chooseUploadFolder() {
	selectDriveFolder(null).then(async ({ canceled, folders }) => {
		if (canceled) return;
		prefer.commit('uploadFolder', folders[0] ? folders[0].id : null);
		os.success();
		if (prefer.s.uploadFolder) {
			uploadFolder.value = await misskeyApi('drive/folders/show', {
				folderId: prefer.s.uploadFolder,
			});
		} else {
			uploadFolder.value = null;
		}
	});
}
async function addWatermarkPreset() {
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkWatermarkEditorDialog.vue').then(x => x.default), {
		presetEditMode: true,
		preset: null,
		layers: [],
	}, {
		presetOk: (preset) => {
			prefer.commit('watermarkPresets', [...prefer.s.watermarkPresets, preset]);
		},
		closed: () => dispose(),
	});
}
function onUpdateWatermarkPreset(id: string, preset: WatermarkPreset) {
	const index = prefer.s.watermarkPresets.findIndex(p => p.id === id);
	if (index !== -1) {
		prefer.commit('watermarkPresets', [
			...prefer.s.watermarkPresets.slice(0, index),
			preset,
			...prefer.s.watermarkPresets.slice(index + 1),
		]);
	}
}
function onDeleteWatermarkPreset(id: string) {
	const index = prefer.s.watermarkPresets.findIndex(p => p.id === id);
	if (index !== -1) {
		prefer.commit('watermarkPresets', [
			...prefer.s.watermarkPresets.slice(0, index),
			...prefer.s.watermarkPresets.slice(index + 1),
		]);
		if (prefer.s.defaultWatermarkPresetId === id) {
			prefer.commit('defaultWatermarkPresetId', null);
		}
	}
}
function onUpdateImageFramePreset(id: string, preset: ImageFramePreset) {
	const index = prefer.s.imageFramePresets.findIndex(p => p.id === id);
	if (index !== -1) {
		prefer.commit('imageFramePresets', [
			...prefer.s.imageFramePresets.slice(0, index),
			preset,
			...prefer.s.imageFramePresets.slice(index + 1),
		]);
	}
}
function onDeleteImageFramePreset(id: string) {
	const index = prefer.s.imageFramePresets.findIndex(p => p.id === id);
	if (index !== -1) {
		prefer.commit('imageFramePresets', [
			...prefer.s.imageFramePresets.slice(0, index),
			...prefer.s.imageFramePresets.slice(index + 1),
		]);
	}
}
async function addImageFramePreset() {
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkImageFrameEditorDialog.vue').then(x => x.default), {
		presetEditMode: true,
		preset: null,
		params: null,
	}, {
		presetOk: (preset) => {
			prefer.commit('imageFramePresets', [...prefer.s.imageFramePresets, preset]);
		},
		closed: () => dispose(),
	});
}
function saveProfile() {
	misskeyApi('i/update', {
		alwaysMarkNsfw: !!alwaysMarkNsfw.value,
		autoSensitive: !!autoSensitive.value,
	}).catch(err => {
		os.alert({
			type: 'error',
			title: i18n.ts.error,
			text: err.message,
		});
		alwaysMarkNsfw.value = true;
	});
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.drive,
	icon: 'ti ti-cloud',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/drive",
      label: _unref(i18n).ts.drive,
      keywords: ['drive'],
      icon: "ti ti-cloud"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/cloud_3d.png",
            color: "#0059ff"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.driveBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(_component_SearchMarker, { keywords: ['capacity', 'usage'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, { first: "" }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.usageAmount), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  (!fetching.value)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "_gaps_m"
                    }, [
                      _createElementVNode("div", null, [
                        _createElementVNode("div", {
                          class: _normalizeClass(_ctx.$style.meter)
                        }, [
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.meterValue),
                            style: _normalizeStyle(meterStyle.value)
                          }, null, 4 /* STYLE */)
                        ])
                      ]),
                      _createVNode(FormSplit, null, {
                        default: _withCtx(() => [
                          _createVNode(MkKeyValue, null, {
                            key: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.capacity), 1 /* TEXT */)
                            ]),
                            value: _withCtx(() => [
                              _createTextVNode(_toDisplayString(bytes(capacity.value, 1)), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }),
                          _createVNode(MkKeyValue, null, {
                            key: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.inUse), 1 /* TEXT */)
                            ]),
                            value: _withCtx(() => [
                              _createTextVNode(_toDisplayString(bytes(usage.value, 1)), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          })
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]))
                    : _createCommentVNode("v-if", true)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['statistics', 'usage'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.statistics), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createVNode(MkChart, {
                    src: "per-user-drive",
                    args: { user: _unref($i) },
                    span: "day",
                    limit: 7 * 5,
                    bar: true,
                    stacked: true,
                    detailed: false,
                    aspectRatio: 6
                  }, null, 8 /* PROPS */, ["args", "limit", "bar", "stacked", "detailed", "aspectRatio"])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['general'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.general), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(_component_SearchMarker, { keywords: ['default', 'upload', 'folder'] }, {
                      default: _withCtx(() => [
                        _createVNode(FormLink, {
                          onClick: _cache[0] || (_cache[0] = ($event: any) => (chooseUploadFolder()))
                        }, {
                          suffix: _withCtx(() => [
                            _createTextVNode(_toDisplayString(uploadFolder.value ? uploadFolder.value.name : '-'), 1 /* TEXT */)
                          ]),
                          icon: _withCtx(() => [
                            _hoisted_1
                          ]),
                          default: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.uploadFolder), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(FormLink, { to: "/settings/drive/cleaner" }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.drivecleaner), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(_component_SearchMarker, { keywords: ['keep', 'original', 'filename'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkPreferenceContainer, { k: "keepOriginalFilename" }, {
                          default: _withCtx(() => [
                            _createVNode(MkSwitch, {
                              modelValue: _unref(keepOriginalFilename),
                              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((keepOriginalFilename).value = $event))
                            }, {
                              label: _withCtx(() => [
                                _createVNode(_component_SearchLabel, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts.keepOriginalFilename), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                })
                              ]),
                              caption: _withCtx(() => [
                                _createVNode(_component_SearchText, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts.keepOriginalFilenameDescription), 1 /* TEXT */)
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
                    _createVNode(_component_SearchMarker, { keywords: ['always', 'default', 'mark', 'nsfw', 'sensitive', 'media', 'file'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkSwitch, {
                          "onUpdate:modelValue": [($event: any) => (saveProfile()), ($event: any) => ((alwaysMarkNsfw).value = $event)],
                          modelValue: alwaysMarkNsfw.value
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.alwaysMarkSensitive), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['auto', 'nsfw', 'sensitive', 'media', 'file'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkSwitch, {
                          "onUpdate:modelValue": [($event: any) => (saveProfile()), ($event: any) => ((autoSensitive).value = $event)],
                          modelValue: autoSensitive.value
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.enableAutoSensitive), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }),
                            _createElementVNode("span", _hoisted_2, _toDisplayString(_unref(i18n).ts.beta), 1 /* TEXT */)
                          ]),
                          caption: _withCtx(() => [
                            _createVNode(_component_SearchText, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.enableAutoSensitiveDescription), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"])
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['image'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.image), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(_component_SearchMarker, { keywords: ['watermark', 'credit'] }, {
                      default: _withCtx(() => [
                        (_unref($i).policies.watermarkAvailable)
                          ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                            icon: _withCtx(() => [
                              _hoisted_3
                            ]),
                            label: _withCtx(() => [
                              _createVNode(_component_SearchLabel, null, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString(_unref(i18n).ts.watermark), 1 /* TEXT */)
                                ]),
                                _: 1 /* STABLE */
                              })
                            ]),
                            caption: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.tip), 1 /* TEXT */)
                            ]),
                            default: _withCtx(() => [
                              _createElementVNode("div", { class: "_gaps" }, [
                                _createElementVNode("div", { class: "_gaps_s" }, [
                                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(prefer).r.watermarkPresets.value, (preset, i) => {
                                    return (_openBlock(), _createBlock(XWatermarkItem, {
                                      key: preset.id,
                                      preset: preset,
                                      onUpdatePreset: _cache[2] || (_cache[2] = ($event: any) => (onUpdateWatermarkPreset(preset.id, $event))),
                                      onDel: _cache[3] || (_cache[3] = ($event: any) => (onDeleteWatermarkPreset(preset.id)))
                                    }, null, 8 /* PROPS */, ["preset"]))
                                  }), 128 /* KEYED_FRAGMENT */)),
                                  _createVNode(MkButton, {
                                    iconOnly: "",
                                    rounded: "",
                                    style: "margin: 0 auto;",
                                    onClick: addWatermarkPreset
                                  }, {
                                    default: _withCtx(() => [
                                      _hoisted_4
                                    ]),
                                    _: 1 /* STABLE */
                                  }),
                                  _createVNode(_component_SearchMarker, { keywords: ['sync', 'watermark', 'preset', 'devices'] }, {
                                    default: _withCtx(() => [
                                      _createVNode(MkSwitch, {
                                        modelValue: watermarkPresetsSyncEnabled.value,
                                        "onUpdate:modelValue": changeWatermarkPresetsSyncEnabled
                                      }, {
                                        label: _withCtx(() => [
                                          _hoisted_5,
                                          _createTextVNode(" "),
                                          _createVNode(_component_SearchLabel, null, {
                                            default: _withCtx(() => [
                                              _createTextVNode(_toDisplayString(_unref(i18n).ts.syncBetweenDevices), 1 /* TEXT */)
                                            ]),
                                            _: 1 /* STABLE */
                                          })
                                        ]),
                                        _: 1 /* STABLE */
                                      }, 8 /* PROPS */, ["modelValue"])
                                    ]),
                                    _: 1 /* STABLE */
                                  }, 8 /* PROPS */, ["keywords"])
                                ]),
                                _hoisted_6,
                                _createVNode(_component_SearchMarker, { keywords: ['default', 'watermark', 'preset'] }, {
                                  default: _withCtx(() => [
                                    _createVNode(MkPreferenceContainer, { k: "defaultWatermarkPresetId" }, {
                                      default: _withCtx(() => [
                                        _createVNode(MkSelect, {
                                          items: [{
  	label: _unref(i18n).ts.none,
  	value: null
  }, ..._unref(prefer).r.watermarkPresets.value.map((p) => ({
  	label: p.name || _unref(i18n).ts.noName,
  	value: p.id
  }))],
                                          modelValue: _unref(defaultWatermarkPresetId),
                                          "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((defaultWatermarkPresetId).value = $event))
                                        }, {
                                          label: _withCtx(() => [
                                            _createVNode(_component_SearchLabel, null, {
                                              default: _withCtx(() => [
                                                _createTextVNode(_toDisplayString(_unref(i18n).ts.defaultPreset), 1 /* TEXT */)
                                              ]),
                                              _: 1 /* STABLE */
                                            })
                                          ]),
                                          _: 1 /* STABLE */
                                        }, 8 /* PROPS */, ["items", "modelValue"])
                                      ]),
                                      _: 1 /* STABLE */
                                    })
                                  ]),
                                  _: 1 /* STABLE */
                                }, 8 /* PROPS */, ["keywords"])
                              ])
                            ]),
                            _: 1 /* STABLE */
                          }))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['label', 'frame', 'credit', 'metadata'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkFolder, null, {
                          icon: _withCtx(() => [
                            _hoisted_7
                          ]),
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.frame), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          caption: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.tip), 1 /* TEXT */)
                          ]),
                          default: _withCtx(() => [
                            _createElementVNode("div", { class: "_gaps" }, [
                              _createElementVNode("div", { class: "_gaps_s" }, [
                                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(prefer).r.imageFramePresets.value, (preset, i) => {
                                  return (_openBlock(), _createBlock(XImageFrameItem, {
                                    key: preset.id,
                                    preset: preset,
                                    onUpdatePreset: _cache[5] || (_cache[5] = ($event: any) => (onUpdateImageFramePreset(preset.id, $event))),
                                    onDel: _cache[6] || (_cache[6] = ($event: any) => (onDeleteImageFramePreset(preset.id)))
                                  }, null, 8 /* PROPS */, ["preset"]))
                                }), 128 /* KEYED_FRAGMENT */)),
                                _createVNode(MkButton, {
                                  iconOnly: "",
                                  rounded: "",
                                  style: "margin: 0 auto;",
                                  onClick: addImageFramePreset
                                }, {
                                  default: _withCtx(() => [
                                    _hoisted_8
                                  ]),
                                  _: 1 /* STABLE */
                                }),
                                _createVNode(_component_SearchMarker, { keywords: ['sync', 'frame', 'label', 'preset', 'devices'] }, {
                                  default: _withCtx(() => [
                                    _createVNode(MkSwitch, {
                                      modelValue: imageFramePresetsSyncEnabled.value,
                                      "onUpdate:modelValue": changeImageFramePresetsSyncEnabled
                                    }, {
                                      label: _withCtx(() => [
                                        _hoisted_9,
                                        _createTextVNode(" "),
                                        _createVNode(_component_SearchLabel, null, {
                                          default: _withCtx(() => [
                                            _createTextVNode(_toDisplayString(_unref(i18n).ts.syncBetweenDevices), 1 /* TEXT */)
                                          ]),
                                          _: 1 /* STABLE */
                                        })
                                      ]),
                                      _: 1 /* STABLE */
                                    }, 8 /* PROPS */, ["modelValue"])
                                  ]),
                                  _: 1 /* STABLE */
                                }, 8 /* PROPS */, ["keywords"])
                              ])
                            ])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['default', 'image', 'compression'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkPreferenceContainer, { k: "defaultImageCompressionLevel" }, {
                          default: _withCtx(() => [
                            _createVNode(MkSelect, {
                              items: [
  									{ label: _unref(i18n).ts.none, value: 0 },
  									{ label: `${_unref(i18n).ts.low} (${_unref(i18n).ts._compression._quality.high}; ${_unref(i18n).ts._compression._size.large})`, value: 1 },
  									{ label: `${_unref(i18n).ts.medium} (${_unref(i18n).ts._compression._quality.medium}; ${_unref(i18n).ts._compression._size.medium})`, value: 2 },
  									{ label: `${_unref(i18n).ts.high} (${_unref(i18n).ts._compression._quality.low}; ${_unref(i18n).ts._compression._size.small})`, value: 3 },
  								],
                              modelValue: _unref(defaultImageCompressionLevel),
                              "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((defaultImageCompressionLevel).value = $event))
                            }, {
                              label: _withCtx(() => [
                                _createVNode(_component_SearchLabel, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts.defaultCompressionLevel), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                })
                              ]),
                              caption: _withCtx(() => [
                                _createElementVNode("div", { innerHTML: _unref(i18n).ts.defaultCompressionLevel_description }, null, 8 /* PROPS */, ["innerHTML"])
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["items", "modelValue"])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"])
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['video'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.video), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(_component_SearchMarker, { keywords: ['default', 'video', 'compression'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkPreferenceContainer, { k: "defaultVideoCompressionLevel" }, {
                          default: _withCtx(() => [
                            _createVNode(MkSelect, {
                              items: [
  									{ label: _unref(i18n).ts.none, value: 0 },
  									{ label: `${_unref(i18n).ts.low} (${_unref(i18n).ts._compression._quality.high}; ${_unref(i18n).ts._compression._size.large})`, value: 1 },
  									{ label: `${_unref(i18n).ts.medium} (${_unref(i18n).ts._compression._quality.medium}; ${_unref(i18n).ts._compression._size.medium})`, value: 2 },
  									{ label: `${_unref(i18n).ts.high} (${_unref(i18n).ts._compression._quality.low}; ${_unref(i18n).ts._compression._size.small})`, value: 3 },
  								],
                              modelValue: _unref(defaultVideoCompressionLevel),
                              "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((defaultVideoCompressionLevel).value = $event))
                            }, {
                              label: _withCtx(() => [
                                _createVNode(_component_SearchLabel, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts.defaultCompressionLevel), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                })
                              ]),
                              caption: _withCtx(() => [
                                _createElementVNode("div", { innerHTML: _unref(i18n).ts.defaultCompressionLevel_description }, null, 8 /* PROPS */, ["innerHTML"])
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["items", "modelValue"])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"])
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
