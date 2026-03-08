import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelRadio as _vModelRadio, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { class: "before" }
const _hoisted_2 = { class: "after" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "crater crater--1" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "crater crater--2" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "crater crater--3" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "star star--1" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "star star--2" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("span", { class: "star star--3" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "star star--4" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "star star--5" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", { class: "star star--6" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-sun" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-moon" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cloud-cog" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-tool" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-world" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-paint" })
import { computed, ref, watch } from 'vue'
import JSON5 from 'json5'
import defaultLightTheme from '@@/themes/l-light.json5'
import defaultDarkTheme from '@@/themes/d-green-lime.json5'
import { isSafeMode } from '@@/js/config.js'
import type { Theme } from '@/theme.js'
import * as os from '@/os.js'
import MkSwitch from '@/components/MkSwitch.vue'
import FormSection from '@/components/form/section.vue'
import FormLink from '@/components/form/link.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkThemePreview from '@/components/MkThemePreview.vue'
import MkInfo from '@/components/MkInfo.vue'
import { getBuiltinThemesRef, getThemesRef, installTheme, parseThemeCode, removeTheme } from '@/theme.js'
import { isDeviceDarkmode } from '@/utility/is-device-darkmode.js'
import { store } from '@/store.js'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { uniqueBy } from '@/utility/array.js'
import { definePage } from '@/page.js'
import { prefer } from '@/preferences.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { checkDragDataType, getDragData, getPlainDragData, setDragData, setPlainDragData } from '@/drag-and-drop.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'theme',
  setup(__props) {

const installedThemes = getThemesRef();
const builtinThemes = getBuiltinThemesRef();
const instanceDarkTheme = computed<Theme | null>(() => instance.defaultDarkTheme ? JSON5.parse(instance.defaultDarkTheme) : null);
const installedDarkThemes = computed(() => installedThemes.value.filter(t => t.base === 'dark' || t.kind === 'dark'));
const builtinDarkThemes = computed(() => builtinThemes.value.filter(t => t.base === 'dark' || t.kind === 'dark'));
const instanceLightTheme = computed<Theme | null>(() => instance.defaultLightTheme ? JSON5.parse(instance.defaultLightTheme) : null);
const installedLightThemes = computed(() => installedThemes.value.filter(t => t.base === 'light' || t.kind === 'light'));
const builtinLightThemes = computed(() => builtinThemes.value.filter(t => t.base === 'light' || t.kind === 'light'));
const themes = computed(() => uniqueBy([instanceDarkTheme.value, instanceLightTheme.value, ...builtinThemes.value, ...installedThemes.value].filter(x => x != null), theme => theme.id));
const darkTheme = prefer.r.darkTheme;
const darkThemeName = computed(() => darkTheme.value?.name ?? defaultDarkTheme.name);
const darkThemeId = computed({
	get() {
		return darkTheme.value ? darkTheme.value.id : defaultDarkTheme.id;
	},
	set(id) {
		const t = themes.value.find(x => x.id === id);
		if (t) { // テーマエディタでテーマを作成したときなどは、themesに反映されないため undefined になる
			prefer.commit('darkTheme', t);
		}
	},
});
const lightTheme = prefer.r.lightTheme;
const lightThemeName = computed(() => lightTheme.value?.name ?? defaultLightTheme.name);
const lightThemeId = computed({
	get() {
		return lightTheme.value ? lightTheme.value.id : defaultLightTheme.id;
	},
	set(id) {
		const t = themes.value.find(x => x.id === id);
		if (t) { // テーマエディタでテーマを作成したときなどは、themesに反映されないため undefined になる
			prefer.commit('lightTheme', t);
		}
	},
});
const syncDeviceDarkMode = prefer.model('syncDeviceDarkMode');
const themesCount = installedThemes.value.length;
watch(syncDeviceDarkMode, () => {
	if (syncDeviceDarkMode.value) {
		store.set('darkMode', isDeviceDarkmode());
	}
});
async function toggleDarkMode() {
	const value = !store.r.darkMode.value;
	if (syncDeviceDarkMode.value) {
		const { canceled } = await os.confirm({
			type: 'question',
			text: i18n.tsx.switchDarkModeManuallyWhenSyncEnabledConfirm({ x: i18n.ts.syncDeviceDarkMode }),
		});
		if (canceled) return;
		syncDeviceDarkMode.value = false;
		store.set('darkMode', value);
	} else {
		store.set('darkMode', value);
	}
}
const themesSyncEnabled = ref(prefer.isSyncEnabled('themes'));
function changeThemesSyncEnabled(value: boolean) {
	if (value) {
		prefer.enableSync('themes').then((res) => {
			if (res == null) return;
			if (res.enabled) themesSyncEnabled.value = true;
		});
	} else {
		prefer.disableSync('themes');
		themesSyncEnabled.value = false;
	}
}
function onThemeContextmenu(theme: Theme, ev: PointerEvent) {
	os.contextMenu([{
		type: 'label',
		text: theme.name,
	}, {
		icon: 'ti ti-clipboard',
		text: i18n.ts._theme.copyThemeCode,
		action: () => {
			copyToClipboard(JSON5.stringify(theme, null, '\t'));
		},
	}, {
		icon: 'ti ti-trash',
		text: i18n.ts.delete,
		danger: true,
		action: () => {
			removeTheme(theme);
		},
	}], ev);
}
function onThemeDragstart(ev: DragEvent, theme: Theme) {
	if (!ev.dataTransfer) return;
	ev.dataTransfer.effectAllowed = 'copy';
	setPlainDragData(ev, JSON5.stringify(theme, null, '\t'));
}
function onDragover(ev: DragEvent) {
	if (!ev.dataTransfer) return;
	if (ev.dataTransfer.types[0] === 'text/plain') {
		ev.dataTransfer.dropEffect = 'copy';
	} else {
		ev.dataTransfer.dropEffect = 'none';
	}
	return false;
}
async function onDrop(ev: DragEvent) {
	if (!ev.dataTransfer) return;
	const code = getPlainDragData(ev);
	if (code != null) {
		try {
			await installTheme(code);
		} catch (err) {
			// nop
		}
	}
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.theme,
	icon: 'ti ti-palette',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _directive_adaptive_border = _resolveDirective("adaptive-border")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/theme",
      label: _unref(i18n).ts.theme,
      keywords: ['theme'],
      icon: "ti ti-palette"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_gaps_m",
          onDragover: _withModifiers(onDragover, ["prevent","stop"]),
          onDrop: _withModifiers(onDrop, ["prevent","stop"])
        }, [
          _createElementVNode("div", { class: "rfqxtzch _panel" }, [
            _createElementVNode("div", { class: "toggle" }, [
              _createElementVNode("div", { class: "toggleWrapper" }, [
                _createElementVNode("div", {
                  class: _normalizeClass(["toggle", _unref(store).r.darkMode.value ? 'checked' : null]),
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleDarkMode()))
                }, [
                  _createElementVNode("span", _hoisted_1, _toDisplayString(_unref(i18n).ts.light), 1 /* TEXT */),
                  _createElementVNode("span", _hoisted_2, _toDisplayString(_unref(i18n).ts.dark), 1 /* TEXT */),
                  _createElementVNode("span", { class: "toggle__handler" }, [
                    _hoisted_3,
                    _hoisted_4,
                    _hoisted_5
                  ]),
                  _hoisted_6,
                  _hoisted_7,
                  _hoisted_8,
                  _hoisted_9,
                  _hoisted_10,
                  _hoisted_11
                ], 2 /* CLASS */)
              ])
            ]),
            _createElementVNode("div", { class: "sync" }, [
              _createVNode(_component_SearchMarker, { keywords: ['sync', 'device', 'dark', 'light', 'mode'] }, {
                default: _withCtx(() => [
                  _createVNode(MkSwitch, {
                    modelValue: _unref(syncDeviceDarkMode),
                    "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((syncDeviceDarkMode).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.syncDeviceDarkMode), 1 /* TEXT */)
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
          (_unref(isSafeMode))
            ? (_openBlock(), _createBlock(MkInfo, {
              key: 0,
              warn: ""
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.themeIsDefaultBecauseSafeMode), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "_gaps"
            }, [
              (!_unref(store).r.darkMode.value)
                ? (_openBlock(), _createBlock(_component_SearchMarker, {
                  key: 0,
                  keywords: ['light', 'theme']
                }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, {
                      defaultOpen: true,
                      "max-height": 500
                    }, {
                      icon: _withCtx(() => [
                        _hoisted_12
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.themeForLightMode), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      caption: _withCtx(() => [
                        _createTextVNode(_toDisplayString(lightThemeName.value), 1 /* TEXT */)
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps_m" }, [
                          (instanceLightTheme.value != null)
                            ? (_openBlock(), _createBlock(FormSection, {
                              key: 0,
                              first: ""
                            }, {
                              label: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.instanceTheme), 1 /* TEXT */)
                              ]),
                              default: _withCtx(() => [
                                _createElementVNode("div", {
                                  class: _normalizeClass(_ctx.$style.themeSelect)
                                }, [
                                  _createElementVNode("div", {
                                    class: _normalizeClass(_ctx.$style.themeItemOuter)
                                  }, [
                                    _withDirectives(_createElementVNode("input", {
                                      id: `themeRadio_${instanceLightTheme.value.id}`,
                                      "onUpdate:modelValue": [($event: any) => ((lightThemeId).value = $event), ($event: any) => ((lightThemeId).value = $event)],
                                      type: "radio",
                                      name: "lightTheme",
                                      class: _normalizeClass(_ctx.$style.themeRadio),
                                      value: instanceLightTheme.value.id
                                    }, null, 8 /* PROPS */, ["id", "value"]), [
                                      [_vModelRadio, lightThemeId.value]
                                    ]),
                                    _createElementVNode("label", {
                                      for: `themeRadio_${instanceLightTheme.value.id}`,
                                      class: _normalizeClass(["_button", _ctx.$style.themeItemRoot]),
                                      draggable: "true",
                                      onDragstart: _cache[2] || (_cache[2] = ($event: any) => (onThemeDragstart($event, instanceLightTheme.value))),
                                      onContextmenu: _cache[3] || (_cache[3] = _withModifiers(($event: any) => (onThemeContextmenu(instanceLightTheme.value, $event)), ["prevent","stop"]))
                                    }, [
                                      _createVNode(MkThemePreview, {
                                        theme: instanceLightTheme.value,
                                        class: _normalizeClass(_ctx.$style.themeItemPreview)
                                      }, null, 8 /* PROPS */, ["theme"]),
                                      _createElementVNode("div", {
                                        class: _normalizeClass(_ctx.$style.themeItemCaption)
                                      }, _toDisplayString(instanceLightTheme.value.name), 1 /* TEXT */)
                                    ], 40 /* PROPS, NEED_HYDRATION */, ["for"])
                                  ])
                                ])
                              ]),
                              _: 1 /* STABLE */
                            }))
                            : _createCommentVNode("v-if", true),
                          (installedLightThemes.value.length > 0)
                            ? (_openBlock(), _createBlock(FormSection, {
                              key: 0,
                              first: instanceLightTheme.value == null
                            }, {
                              label: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.installedThemes), 1 /* TEXT */)
                              ]),
                              default: _withCtx(() => [
                                _createElementVNode("div", {
                                  class: _normalizeClass(_ctx.$style.themeSelect)
                                }, [
                                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(installedLightThemes.value, (theme) => {
                                    return (_openBlock(), _createElementBlock("div", { class: _normalizeClass(_ctx.$style.themeItemOuter) }, [
                                      _withDirectives(_createElementVNode("input", {
                                        id: `themeRadio_${theme.id}`,
                                        "onUpdate:modelValue": [($event: any) => ((lightThemeId).value = $event), ($event: any) => ((lightThemeId).value = $event)],
                                        type: "radio",
                                        name: "lightTheme",
                                        class: _normalizeClass(_ctx.$style.themeRadio),
                                        value: theme.id
                                      }, null, 8 /* PROPS */, ["id", "value"]), [
                                        [_vModelRadio, lightThemeId.value]
                                      ]),
                                      _createElementVNode("label", {
                                        for: `themeRadio_${theme.id}`,
                                        class: _normalizeClass(["_button", _ctx.$style.themeItemRoot]),
                                        draggable: "true",
                                        onDragstart: _cache[4] || (_cache[4] = ($event: any) => (onThemeDragstart($event, theme))),
                                        onContextmenu: _cache[5] || (_cache[5] = _withModifiers(($event: any) => (onThemeContextmenu(theme, $event)), ["prevent","stop"]))
                                      }, [
                                        _createVNode(MkThemePreview, {
                                          theme: theme,
                                          class: _normalizeClass(_ctx.$style.themeItemPreview)
                                        }, null, 8 /* PROPS */, ["theme"]),
                                        _createElementVNode("div", {
                                          class: _normalizeClass(_ctx.$style.themeItemCaption)
                                        }, _toDisplayString(theme.name), 1 /* TEXT */)
                                      ], 40 /* PROPS, NEED_HYDRATION */, ["for"])
                                    ]))
                                  }), 256 /* UNKEYED_FRAGMENT */))
                                ])
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["first"]))
                            : _createCommentVNode("v-if", true),
                          _createVNode(FormSection, { first: installedLightThemes.value.length === 0 && instanceLightTheme.value == null }, {
                            label: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.builtinThemes), 1 /* TEXT */)
                            ]),
                            default: _withCtx(() => [
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.themeSelect)
                              }, [
                                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(builtinLightThemes.value, (theme) => {
                                  return (_openBlock(), _createElementBlock("div", { class: _normalizeClass(_ctx.$style.themeItemOuter) }, [
                                    _withDirectives(_createElementVNode("input", {
                                      id: `themeRadio_${theme.id}`,
                                      "onUpdate:modelValue": [($event: any) => ((lightThemeId).value = $event), ($event: any) => ((lightThemeId).value = $event)],
                                      type: "radio",
                                      name: "lightTheme",
                                      class: _normalizeClass(_ctx.$style.themeRadio),
                                      value: theme.id
                                    }, null, 8 /* PROPS */, ["id", "value"]), [
                                      [_vModelRadio, lightThemeId.value]
                                    ]),
                                    _createElementVNode("label", {
                                      for: `themeRadio_${theme.id}`,
                                      class: _normalizeClass(["_button", _ctx.$style.themeItemRoot]),
                                      draggable: "true",
                                      onDragstart: _cache[6] || (_cache[6] = ($event: any) => (onThemeDragstart($event, theme))),
                                      onContextmenu: _cache[7] || (_cache[7] = _withModifiers(($event: any) => (onThemeContextmenu(theme, $event)), ["prevent","stop"]))
                                    }, [
                                      _createVNode(MkThemePreview, {
                                        theme: theme,
                                        class: _normalizeClass(_ctx.$style.themeItemPreview)
                                      }, null, 8 /* PROPS */, ["theme"]),
                                      _createElementVNode("div", {
                                        class: _normalizeClass(_ctx.$style.themeItemCaption)
                                      }, _toDisplayString(theme.name), 1 /* TEXT */)
                                    ], 40 /* PROPS, NEED_HYDRATION */, ["for"])
                                  ]))
                                }), 256 /* UNKEYED_FRAGMENT */))
                              ])
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["first"])
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["defaultOpen", "max-height"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]))
                : (_openBlock(), _createBlock(_component_SearchMarker, {
                  key: 1,
                  keywords: ['dark', 'theme']
                }, {
                  default: _withCtx(() => [
                    _createVNode(MkFolder, {
                      defaultOpen: true,
                      "max-height": 500
                    }, {
                      icon: _withCtx(() => [
                        _hoisted_13
                      ]),
                      label: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.themeForDarkMode), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      caption: _withCtx(() => [
                        _createTextVNode(_toDisplayString(darkThemeName.value), 1 /* TEXT */)
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "_gaps_m" }, [
                          (instanceDarkTheme.value != null)
                            ? (_openBlock(), _createBlock(FormSection, {
                              key: 0,
                              first: ""
                            }, {
                              label: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.instanceTheme), 1 /* TEXT */)
                              ]),
                              default: _withCtx(() => [
                                _createElementVNode("div", {
                                  class: _normalizeClass(_ctx.$style.themeSelect)
                                }, [
                                  _createElementVNode("div", {
                                    class: _normalizeClass(_ctx.$style.themeItemOuter)
                                  }, [
                                    _withDirectives(_createElementVNode("input", {
                                      id: `themeRadio_${instanceDarkTheme.value.id}`,
                                      "onUpdate:modelValue": [($event: any) => ((darkThemeId).value = $event), ($event: any) => ((darkThemeId).value = $event)],
                                      type: "radio",
                                      name: "darkTheme",
                                      class: _normalizeClass(_ctx.$style.themeRadio),
                                      value: instanceDarkTheme.value.id
                                    }, null, 8 /* PROPS */, ["id", "value"]), [
                                      [_vModelRadio, darkThemeId.value]
                                    ]),
                                    _createElementVNode("label", {
                                      for: `themeRadio_${instanceDarkTheme.value.id}`,
                                      class: _normalizeClass(["_button", _ctx.$style.themeItemRoot]),
                                      draggable: "true",
                                      onDragstart: _cache[8] || (_cache[8] = ($event: any) => (onThemeDragstart($event, instanceDarkTheme.value))),
                                      onContextmenu: _cache[9] || (_cache[9] = _withModifiers(($event: any) => (onThemeContextmenu(instanceDarkTheme.value, $event)), ["prevent","stop"]))
                                    }, [
                                      _createVNode(MkThemePreview, {
                                        theme: instanceDarkTheme.value,
                                        class: _normalizeClass(_ctx.$style.themeItemPreview)
                                      }, null, 8 /* PROPS */, ["theme"]),
                                      _createElementVNode("div", {
                                        class: _normalizeClass(_ctx.$style.themeItemCaption)
                                      }, _toDisplayString(instanceDarkTheme.value.name), 1 /* TEXT */)
                                    ], 40 /* PROPS, NEED_HYDRATION */, ["for"])
                                  ])
                                ])
                              ]),
                              _: 1 /* STABLE */
                            }))
                            : _createCommentVNode("v-if", true),
                          (installedDarkThemes.value.length > 0)
                            ? (_openBlock(), _createBlock(FormSection, {
                              key: 0,
                              first: instanceDarkTheme.value == null
                            }, {
                              label: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.installedThemes), 1 /* TEXT */)
                              ]),
                              default: _withCtx(() => [
                                _createElementVNode("div", {
                                  class: _normalizeClass(_ctx.$style.themeSelect)
                                }, [
                                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(installedDarkThemes.value, (theme) => {
                                    return (_openBlock(), _createElementBlock("div", { class: _normalizeClass(_ctx.$style.themeItemOuter) }, [
                                      _withDirectives(_createElementVNode("input", {
                                        id: `themeRadio_${theme.id}`,
                                        "onUpdate:modelValue": [($event: any) => ((darkThemeId).value = $event), ($event: any) => ((darkThemeId).value = $event)],
                                        type: "radio",
                                        name: "darkTheme",
                                        class: _normalizeClass(_ctx.$style.themeRadio),
                                        value: theme.id
                                      }, null, 8 /* PROPS */, ["id", "value"]), [
                                        [_vModelRadio, darkThemeId.value]
                                      ]),
                                      _createElementVNode("label", {
                                        for: `themeRadio_${theme.id}`,
                                        class: _normalizeClass(["_button", _ctx.$style.themeItemRoot]),
                                        draggable: "true",
                                        onDragstart: _cache[10] || (_cache[10] = ($event: any) => (onThemeDragstart($event, theme))),
                                        onContextmenu: _cache[11] || (_cache[11] = _withModifiers(($event: any) => (onThemeContextmenu(theme, $event)), ["prevent","stop"]))
                                      }, [
                                        _createVNode(MkThemePreview, {
                                          theme: theme,
                                          class: _normalizeClass(_ctx.$style.themeItemPreview)
                                        }, null, 8 /* PROPS */, ["theme"]),
                                        _createElementVNode("div", {
                                          class: _normalizeClass(_ctx.$style.themeItemCaption)
                                        }, _toDisplayString(theme.name), 1 /* TEXT */)
                                      ], 40 /* PROPS, NEED_HYDRATION */, ["for"])
                                    ]))
                                  }), 256 /* UNKEYED_FRAGMENT */))
                                ])
                              ]),
                              _: 1 /* STABLE */
                            }, 8 /* PROPS */, ["first"]))
                            : _createCommentVNode("v-if", true),
                          _createVNode(FormSection, { first: installedDarkThemes.value.length === 0 && instanceDarkTheme.value == null }, {
                            label: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.builtinThemes), 1 /* TEXT */)
                            ]),
                            default: _withCtx(() => [
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.themeSelect)
                              }, [
                                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(builtinDarkThemes.value, (theme) => {
                                  return (_openBlock(), _createElementBlock("div", { class: _normalizeClass(_ctx.$style.themeItemOuter) }, [
                                    _withDirectives(_createElementVNode("input", {
                                      id: `themeRadio_${theme.id}`,
                                      "onUpdate:modelValue": [($event: any) => ((darkThemeId).value = $event), ($event: any) => ((darkThemeId).value = $event)],
                                      type: "radio",
                                      name: "darkTheme",
                                      class: _normalizeClass(_ctx.$style.themeRadio),
                                      value: theme.id
                                    }, null, 8 /* PROPS */, ["id", "value"]), [
                                      [_vModelRadio, darkThemeId.value]
                                    ]),
                                    _createElementVNode("label", {
                                      for: `themeRadio_${theme.id}`,
                                      class: _normalizeClass(["_button", _ctx.$style.themeItemRoot]),
                                      draggable: "true",
                                      onDragstart: _cache[12] || (_cache[12] = ($event: any) => (onThemeDragstart($event, theme))),
                                      onContextmenu: _cache[13] || (_cache[13] = _withModifiers(($event: any) => (onThemeContextmenu(theme, $event)), ["prevent","stop"]))
                                    }, [
                                      _createVNode(MkThemePreview, {
                                        theme: theme,
                                        class: _normalizeClass(_ctx.$style.themeItemPreview)
                                      }, null, 8 /* PROPS */, ["theme"]),
                                      _createElementVNode("div", {
                                        class: _normalizeClass(_ctx.$style.themeItemCaption)
                                      }, _toDisplayString(theme.name), 1 /* TEXT */)
                                    ], 40 /* PROPS, NEED_HYDRATION */, ["for"])
                                  ]))
                                }), 256 /* UNKEYED_FRAGMENT */))
                              ])
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["first"])
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["defaultOpen", "max-height"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]))
            ])),
          _createVNode(_component_SearchMarker, { keywords: ['sync', 'themes', 'devices'] }, {
            default: _withCtx(() => [
              _createVNode(MkSwitch, {
                modelValue: themesSyncEnabled.value,
                "onUpdate:modelValue": changeThemesSyncEnabled
              }, {
                label: _withCtx(() => [
                  _hoisted_14,
                  _createTextVNode(" "),
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.enableSyncThemesBetweenDevices), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(FormSection, null, {
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_formLinksGrid" }, [
                _createVNode(FormLink, { to: "/settings/theme/manage" }, {
                  icon: _withCtx(() => [
                    _hoisted_15
                  ]),
                  suffix: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(themesCount)), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.manage), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(FormLink, {
                  to: "https://assets.misskey.io/theme/list",
                  external: ""
                }, {
                  icon: _withCtx(() => [
                    _hoisted_16
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.explore), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(FormLink, { to: "/settings/theme/install" }, {
                  icon: _withCtx(() => [
                    _hoisted_17
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.install), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(FormLink, { to: "/theme-editor" }, {
                  icon: _withCtx(() => [
                    _hoisted_18
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.make), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ]),
            _: 1 /* STABLE */
          })
        ], 32 /* NEED_HYDRATION */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
