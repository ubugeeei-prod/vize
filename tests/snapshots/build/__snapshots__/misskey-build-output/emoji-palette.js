import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cloud-cog" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye" })
import { computed, ref, watch } from 'vue'
import XPalette from './emoji-palette.palette.vue'
import type { MkSelectItem } from '@/components/MkSelect.vue'
import type { MkRadiosOption } from '@/components/MkRadios.vue'
import { genId } from '@/utility/id.js'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'
import MkRadios from '@/components/MkRadios.vue'
import MkButton from '@/components/MkButton.vue'
import FormSection from '@/components/form/section.vue'
import MkSelect from '@/components/MkSelect.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkFolder from '@/components/MkFolder.vue'
import { prefer } from '@/preferences.js'
import MkPreferenceContainer from '@/components/MkPreferenceContainer.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import { emojiPicker } from '@/utility/emoji-picker.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'emoji-palette',
  setup(__props) {

const emojiPaletteForReaction = prefer.model('emojiPaletteForReaction');
const emojiPaletteForReactionDef = computed<MkSelectItem[]>(() => [
	{ label: `(${i18n.ts.auto})`, value: null },
	...prefer.s.emojiPalettes.map(palette => ({
		label: palette.name === '' ? `(${i18n.ts.noName})` : palette.name,
		value: palette.id,
	})),
]);
const emojiPaletteForMain = prefer.model('emojiPaletteForMain');
const emojiPaletteForMainDef = computed<MkSelectItem[]>(() => [
	{ label: `(${i18n.ts.auto})`, value: null },
	...prefer.s.emojiPalettes.map(palette => ({
		label: palette.name === '' ? `(${i18n.ts.noName})` : palette.name,
		value: palette.id,
	})),
]);
const emojiPickerScale = prefer.model('emojiPickerScale');
const emojiPickerScaleDef = [
	{ label: i18n.ts.small, value: 1 },
	{ label: i18n.ts.medium, value: 2 },
	{ label: i18n.ts.large, value: 3 },
	{ label: i18n.ts.large + '+', value: 4 },
	{ label: i18n.ts.large + '++', value: 5 },
] as MkRadiosOption<number>[];
const emojiPickerWidth = prefer.model('emojiPickerWidth');
const emojiPickerWidthDef = [
	{ label: '5', value: 1 },
	{ label: '6', value: 2 },
	{ label: '7', value: 3 },
	{ label: '8', value: 4 },
	{ label: '9', value: 5 },
] as MkRadiosOption<number>[];
const emojiPickerHeight = prefer.model('emojiPickerHeight');
const emojiPickerHeightDef = [
	{ label: i18n.ts.small, value: 1 },
	{ label: i18n.ts.medium, value: 2 },
	{ label: i18n.ts.large, value: 3 },
	{ label: i18n.ts.large + '+', value: 4 },
] as MkRadiosOption<number>[];
const emojiPickerStyle = prefer.model('emojiPickerStyle');
const palettesSyncEnabled = ref(prefer.isSyncEnabled('emojiPalettes'));
function changePalettesSyncEnabled(value: boolean) {
	if (value) {
		prefer.enableSync('emojiPalettes').then((res) => {
			if (res == null) return;
			if (res.enabled) palettesSyncEnabled.value = true;
		});
	} else {
		prefer.disableSync('emojiPalettes');
		palettesSyncEnabled.value = false;
	}
}
function addPalette() {
	prefer.commit('emojiPalettes', [
		...prefer.s.emojiPalettes,
		{
			id: genId(),
			name: '',
			emojis: [],
		},
	]);
}
function updatePaletteEmojis(id: string, emojis: string[]) {
	prefer.commit('emojiPalettes', prefer.s.emojiPalettes.map(palette => {
		if (palette.id === id) {
			return {
				...palette,
				emojis,
			};
		} else {
			return palette;
		}
	}));
}
function updatePaletteName(id: string, name: string) {
	prefer.commit('emojiPalettes', prefer.s.emojiPalettes.map(palette => {
		if (palette.id === id) {
			return {
				...palette,
				name,
			};
		} else {
			return palette;
		}
	}));
}
function delPalette(id: string) {
	if (prefer.s.emojiPalettes.length === 1) {
		addPalette();
	}
	prefer.commit('emojiPalettes', prefer.s.emojiPalettes.filter(palette => palette.id !== id));
	if (prefer.s.emojiPaletteForMain === id) {
		prefer.commit('emojiPaletteForMain', null);
	}
	if (prefer.s.emojiPaletteForReaction === id) {
		prefer.commit('emojiPaletteForReaction', null);
	}
}
function getHTMLElement(ev: PointerEvent): HTMLElement {
	const target = ev.currentTarget ?? ev.target;
	return target as HTMLElement;
}
function previewPicker(ev: PointerEvent) {
	emojiPicker.show(getHTMLElement(ev));
}
definePage(() => ({
	title: i18n.ts.emojiPalette,
	icon: 'ti ti-mood-happy',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/emoji-palette",
      label: _unref(i18n).ts.emojiPalette,
      keywords: ['emoji', 'palette'],
      icon: "ti ti-mood-happy"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/artist_palette_3d.png",
            color: "#ff9100"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.emojiPaletteBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(FormSection, { first: "" }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._emojiPalette.palettes), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_s" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(prefer).r.emojiPalettes.value, (palette) => {
                  return (_openBlock(), _createBlock(XPalette, {
                    key: palette.id,
                    palette: palette,
                    onUpdateEmojis: _cache[0] || (_cache[0] = emojis => updatePaletteEmojis(palette.id, emojis)),
                    onUpdateName: _cache[1] || (_cache[1] = name => updatePaletteName(palette.id, name)),
                    onDel: _cache[2] || (_cache[2] = ($event: any) => (delPalette(palette.id)))
                  }, null, 8 /* PROPS */, ["palette"]))
                }), 128 /* KEYED_FRAGMENT */)),
                _createVNode(MkButton, {
                  primary: "",
                  rounded: "",
                  style: "margin: auto;",
                  onClick: addPalette
                }, {
                  default: _withCtx(() => [
                    _hoisted_1
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(FormSection, null, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(_component_SearchMarker, { keywords: ['sync', 'palettes', 'devices'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkSwitch, {
                      modelValue: palettesSyncEnabled.value,
                      "onUpdate:modelValue": changePalettesSyncEnabled
                    }, {
                      label: _withCtx(() => [
                        _hoisted_2,
                        _createTextVNode(" "),
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._emojiPalette.enableSyncBetweenDevicesForPalettes), 1 /* TEXT */)
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
          }),
          _createVNode(FormSection, null, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(_component_SearchMarker, { keywords: ['main', 'palette'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkPreferenceContainer, { k: "emojiPaletteForMain" }, {
                      default: _withCtx(() => [
                        _createVNode(MkSelect, {
                          items: emojiPaletteForMainDef.value,
                          modelValue: _unref(emojiPaletteForMain),
                          "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((emojiPaletteForMain).value = $event))
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._emojiPalette.paletteForMain), 1 /* TEXT */)
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
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(_component_SearchMarker, { keywords: ['reaction', 'palette'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkPreferenceContainer, { k: "emojiPaletteForReaction" }, {
                      default: _withCtx(() => [
                        _createVNode(MkSelect, {
                          items: emojiPaletteForReactionDef.value,
                          modelValue: _unref(emojiPaletteForReaction),
                          "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((emojiPaletteForReaction).value = $event))
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._emojiPalette.paletteForReaction), 1 /* TEXT */)
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
          }),
          _createVNode(_component_SearchMarker, { keywords: ['emoji', 'picker', 'display'] }, {
            default: _withCtx(() => [
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.emojiPickerDisplay), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(_component_SearchMarker, { keywords: ['emoji', 'picker', 'scale', 'size'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkPreferenceContainer, { k: "emojiPickerScale" }, {
                          default: _withCtx(() => [
                            _createVNode(MkRadios, {
                              options: _unref(emojiPickerScaleDef),
                              modelValue: _unref(emojiPickerScale),
                              "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((emojiPickerScale).value = $event))
                            }, {
                              label: _withCtx(() => [
                                _createVNode(_component_SearchLabel, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts.size), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                })
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["options", "modelValue"])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['emoji', 'picker', 'width', 'column', 'size'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkPreferenceContainer, { k: "emojiPickerWidth" }, {
                          default: _withCtx(() => [
                            _createVNode(MkRadios, {
                              options: _unref(emojiPickerWidthDef),
                              modelValue: _unref(emojiPickerWidth),
                              "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((emojiPickerWidth).value = $event))
                            }, {
                              label: _withCtx(() => [
                                _createVNode(_component_SearchLabel, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts.numberOfColumn), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                })
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["options", "modelValue"])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['emoji', 'picker', 'height', 'size'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkPreferenceContainer, { k: "emojiPickerHeight" }, {
                          default: _withCtx(() => [
                            _createVNode(MkRadios, {
                              options: _unref(emojiPickerHeightDef),
                              modelValue: _unref(emojiPickerHeight),
                              "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((emojiPickerHeight).value = $event))
                            }, {
                              label: _withCtx(() => [
                                _createVNode(_component_SearchLabel, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts.height), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                })
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["options", "modelValue"])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['emoji', 'picker', 'style'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkPreferenceContainer, { k: "emojiPickerStyle" }, {
                          default: _withCtx(() => [
                            _createVNode(MkSelect, {
                              items: [
  									{ label: _unref(i18n).ts.auto, value: 'auto' },
  									{ label: _unref(i18n).ts.popup, value: 'popup' },
  									{ label: _unref(i18n).ts.drawer, value: 'drawer' },
  								],
                              modelValue: _unref(emojiPickerStyle),
                              "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((emojiPickerStyle).value = $event))
                            }, {
                              label: _withCtx(() => [
                                _createVNode(_component_SearchLabel, null, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(_unref(i18n).ts.style), 1 /* TEXT */)
                                  ]),
                                  _: 1 /* STABLE */
                                })
                              ]),
                              caption: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.needReloadToApply), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["items", "modelValue"])
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(MkButton, { onClick: previewPicker }, {
                      default: _withCtx(() => [
                        _hoisted_3,
                        _createTextVNode(" "),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.preview), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
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
