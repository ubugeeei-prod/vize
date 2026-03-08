import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plug" })
const _hoisted_3 = { style: "margin-left: 1em; opacity: 0.7;" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-refresh" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-terminal-2" })
const _hoisted_8 = { class: "_monospace" }
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
import { nextTick, ref, computed } from 'vue'
import { isSafeMode } from '@@/js/config.js'
import type { Plugin } from '@/plugin.js'
import FormLink from '@/components/form/link.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import FormSection from '@/components/form/section.vue'
import MkButton from '@/components/MkButton.vue'
import MkCode from '@/components/MkCode.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkFeatureBanner from '@/components/MkFeatureBanner.vue'
import MkInfo from '@/components/MkInfo.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { changePluginActive, configPlugin, pluginLogs, uninstallPlugin, reloadPlugin } from '@/plugin.js'
import { prefer } from '@/preferences.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'plugin',
  setup(__props) {

const plugins = prefer.r.plugins;
async function uninstall(plugin: Plugin) {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.removeAreYouSure({ x: plugin.name }),
	});
	if (canceled) return;
	await uninstallPlugin(plugin);
	os.success();
}
function reload(plugin: Plugin) {
	reloadPlugin(plugin);
}
async function config(plugin: Plugin) {
	await configPlugin(plugin);
}
function changeActive(plugin: Plugin, active: boolean) {
	changePluginActive(plugin, active);
}
function timeToHhMmSs(unixtime: number) {
	return new Date(unixtime).toTimeString().split(' ')[0];
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.plugins,
	icon: 'ti ti-plug',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/plugin",
      label: _unref(i18n).ts.plugins,
      keywords: ['plugin', 'addon', 'extension'],
      icon: "ti ti-plug"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/electric_plug_3d.png",
            color: "#ffbb00"
          }, {
            default: _withCtx(() => [
              _createVNode(_component_SearchText, null, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._settings.pluginBanner), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          (_unref(isSafeMode))
            ? (_openBlock(), _createBlock(MkInfo, {
              key: 0,
              warn: ""
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.pluginsAreDisabledBecauseSafeMode), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : (_openBlock(), _createBlock(FormLink, {
              key: 1,
              to: "/settings/plugin/install"
            }, {
              icon: _withCtx(() => [
                _hoisted_1
              ]),
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._plugin.install), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })),
          _createVNode(FormSection, null, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.manage), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_s" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(plugins), (plugin) => {
                  return (_openBlock(), _createBlock(MkFolder, { key: plugin.installId }, {
                    icon: _withCtx(() => [
                      _hoisted_2
                    ]),
                    suffix: _withCtx(() => [
                      (plugin.active)
                        ? (_openBlock(), _createElementBlock("i", {
                          key: 0,
                          class: "ti ti-player-play",
                          style: "color: var(--MI_THEME-success);"
                        }))
                        : (_openBlock(), _createElementBlock("i", {
                          key: 1,
                          class: "ti ti-player-pause",
                          style: "opacity: 0.7;"
                        }))
                    ]),
                    label: _withCtx(() => [
                      _createElementVNode("div", {
                        style: _normalizeStyle(plugin.active ? '' : 'opacity: 0.7;')
                      }, [
                        _createTextVNode(_toDisplayString(plugin.name) + "\n\t\t\t\t\t\t\t", 1 /* TEXT */),
                        _createElementVNode("span", _hoisted_3, "v" + _toDisplayString(plugin.version), 1 /* TEXT */)
                      ], 4 /* STYLE */)
                    ]),
                    caption: _withCtx(() => [
                      _createTextVNode(_toDisplayString(plugin.description), 1 /* TEXT */)
                    ]),
                    footer: _withCtx(() => [
                      _createElementVNode("div", { class: "_buttons" }, [
                        _createVNode(MkButton, {
                          disabled: !plugin.active,
                          onClick: _cache[0] || (_cache[0] = ($event: any) => (reload(plugin)))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_4,
                            _createTextVNode(" "),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.reload), 1 /* TEXT */)
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 8 /* PROPS */, ["disabled"]),
                        _createVNode(MkButton, {
                          danger: "",
                          onClick: _cache[1] || (_cache[1] = ($event: any) => (uninstall(plugin)))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_5,
                            _createTextVNode(" "),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.uninstall), 1 /* TEXT */)
                          ]),
                          _: 2 /* DYNAMIC */
                        }),
                        (plugin.config)
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            style: "margin-left: auto;",
                            onClick: _cache[2] || (_cache[2] = ($event: any) => (config(plugin)))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_6,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.settings), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }))
                          : _createCommentVNode("v-if", true)
                      ])
                    ]),
                    default: _withCtx(() => [
                      _createElementVNode("div", { class: "_gaps_m" }, [
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          _createVNode(MkSwitch, {
                            modelValue: plugin.active,
                            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => (changeActive(plugin, $event)))
                          }, {
                            default: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.makeActive), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 8 /* PROPS */, ["modelValue"])
                        ]),
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          _createVNode(MkKeyValue, null, {
                            key: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.author), 1 /* TEXT */)
                            ]),
                            value: _withCtx(() => [
                              _createTextVNode(_toDisplayString(plugin.author), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }),
                          _createVNode(MkKeyValue, null, {
                            key: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.description), 1 /* TEXT */)
                            ]),
                            value: _withCtx(() => [
                              _createTextVNode(_toDisplayString(plugin.description), 1 /* TEXT */)
                            ]),
                            _: 2 /* DYNAMIC */
                          }),
                          _createVNode(MkKeyValue, null, {
                            key: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.permission), 1 /* TEXT */)
                            ]),
                            value: _withCtx(() => [
                              _createElementVNode("ul", { style: "margin-top: 0; margin-bottom: 0;" }, [
                                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(plugin.permissions, (permission) => {
                                  return (_openBlock(), _createElementBlock("li", { key: permission }, _toDisplayString(_unref(i18n).ts._permissions[permission] ?? permission), 1 /* TEXT */))
                                }), 128 /* KEYED_FRAGMENT */)),
                                (!plugin.permissions || plugin.permissions.length === 0)
                                  ? (_openBlock(), _createElementBlock("li", { key: 0 }, _toDisplayString(_unref(i18n).ts.none), 1 /* TEXT */))
                                  : _createCommentVNode("v-if", true)
                              ])
                            ]),
                            _: 2 /* DYNAMIC */
                          })
                        ]),
                        _createElementVNode("div", { class: "_gaps_s" }, [
                          _createVNode(MkFolder, null, {
                            icon: _withCtx(() => [
                              _hoisted_7
                            ]),
                            label: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.logs), 1 /* TEXT */)
                            ]),
                            default: _withCtx(() => [
                              _createElementVNode("div", null, [
                                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(pluginLogs).get(plugin.installId), (log) => {
                                  return (_openBlock(), _createElementBlock("div", { class: _normalizeClass([_ctx.$style.log, { [_ctx.$style.isSystemLog]: log.isSystem }]) }, [
                                    _createElementVNode("div", _hoisted_8, _toDisplayString(timeToHhMmSs(log.at)) + " " + _toDisplayString(log.message), 1 /* TEXT */)
                                  ], 2 /* CLASS */))
                                }), 256 /* UNKEYED_FRAGMENT */))
                              ])
                            ]),
                            _: 2 /* DYNAMIC */
                          }),
                          _createVNode(MkFolder, { withSpacer: false }, {
                            icon: _withCtx(() => [
                              _hoisted_9
                            ]),
                            label: _withCtx(() => [
                              _createTextVNode(_toDisplayString(_unref(i18n).ts._plugin.viewSource), 1 /* TEXT */)
                            ]),
                            default: _withCtx(() => [
                              _createElementVNode("div", { class: "_gaps_s" }, [
                                _createVNode(MkCode, {
                                  code: plugin.src ?? '',
                                  lang: "ais"
                                }, null, 8 /* PROPS */, ["code"])
                              ])
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 8 /* PROPS */, ["withSpacer"])
                        ])
                      ])
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 1024 /* DYNAMIC_SLOTS */))
                }), 128 /* KEYED_FRAGMENT */))
              ])
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
