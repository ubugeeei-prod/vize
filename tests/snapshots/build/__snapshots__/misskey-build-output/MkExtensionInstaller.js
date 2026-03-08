import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-info-circle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-info-circle" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
import { computed } from 'vue'
import MkButton from '@/components/MkButton.vue'
import FormSplit from '@/components/form/split.vue'
import MkCode from '@/components/MkCode.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import { i18n } from '@/i18n.js'

import * as Misskey from 'misskey-js';

export type Extension = {
	type: 'plugin';
	raw: string;
	meta: {
		name: string;
		version: string;
		author: string;
		description?: string;
		permissions?: (typeof Misskey.permissions)[number][];
		config?: Record<string, unknown>;
	};
} | {
	type: 'theme';
	raw: string;
	meta: {
		name: string;
		author: string;
		base?: 'light' | 'dark';
	};
};

export default /*@__PURE__*/_defineComponent({
  __name: 'MkExtensionInstaller',
  props: {
    extension: { type: Object, required: true }
  },
  emits: ["confirm", "cancel"],
  setup(__props: any, { emit: __emit }) {

const emits = __emit
const props = __props
const isPlugin = computed(() => props.extension.type === 'plugin');
const isTheme = computed(() => props.extension.type === 'theme');

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_gaps_m", _ctx.$style.extInstallerRoot])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.extInstallerIconWrapper)
      }, [ (isPlugin.value) ? (_openBlock(), _createElementBlock("i", {
            key: 0,
            class: "ti ti-plug"
          })) : (isTheme.value) ? (_openBlock(), _createElementBlock("i", {
              key: 1,
              class: "ti ti-palette"
            })) : (_openBlock(), _createElementBlock("i", {
            key: 2,
            class: "ti ti-download"
          })), _createTextVNode("\n\t\t" + "\n\t\t") ]), (isPlugin.value) ? (_openBlock(), _createElementBlock("h2", {
          key: 0,
          class: _normalizeClass(_ctx.$style.extInstallerTitle)
        }, _toDisplayString(_unref(i18n).ts._externalResourceInstaller._plugin.title), 1 /* TEXT */)) : (isTheme.value) ? (_openBlock(), _createElementBlock("h2", {
            key: 1,
            class: _normalizeClass(_ctx.$style.extInstallerTitle)
          }, _toDisplayString(_unref(i18n).ts._externalResourceInstaller._theme.title), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createVNode(MkInfo, { warn: true }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._externalResourceInstaller.checkVendorBeforeInstall), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["warn"]), (__props.extension.type === 'plugin') ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps_s"
        }, [ _createVNode(MkFolder, { defaultOpen: true }, {
            icon: _withCtx(() => [
              _hoisted_1
            ]),
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.metadata), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", { class: "_gaps_s" }, [
                _createVNode(FormSplit, null, {
                  default: _withCtx(() => [
                    _createVNode(MkKeyValue, null, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.name), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        _createTextVNode(_toDisplayString(__props.extension.meta.name), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkKeyValue, null, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.author), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        _createTextVNode(_toDisplayString(__props.extension.meta.author), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkKeyValue, null, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.description), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(__props.extension.meta.description ?? _unref(i18n).ts.none), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkKeyValue, null, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.version), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(__props.extension.meta.version), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkKeyValue, null, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.permission), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    (__props.extension.meta.permissions && __props.extension.meta.permissions.length > 0)
                      ? (_openBlock(), _createElementBlock("ul", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.extInstallerKVList)
                      }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.extension.meta.permissions, (permission) => {
                          return (_openBlock(), _createElementBlock("li", { key: permission }, _toDisplayString(_unref(i18n).ts._permissions[permission] ?? permission), 1 /* TEXT */))
                        }), 128 /* KEYED_FRAGMENT */))
                      ]))
                      : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                        _toDisplayString(_unref(i18n).ts.none)
                      ], 64 /* STABLE_FRAGMENT */))
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["defaultOpen"]), _createVNode(MkFolder, { withSpacer: false }, {
            icon: _withCtx(() => [
              _hoisted_2
            ]),
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._plugin.viewSource), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createVNode(MkCode, { code: __props.extension.raw }, null, 8 /* PROPS */, ["code"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["withSpacer"]) ])) : (__props.extension.type === 'theme') ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: "_gaps_s"
          }, [ _createVNode(MkFolder, { defaultOpen: true }, {
              icon: _withCtx(() => [
                _hoisted_3
              ]),
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.metadata), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", { class: "_gaps_s" }, [
                  _createVNode(FormSplit, null, {
                    default: _withCtx(() => [
                      _createVNode(MkKeyValue, null, {
                        key: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.name), 1 /* TEXT */)
                        ]),
                        value: _withCtx(() => [
                          _createTextVNode(_toDisplayString(__props.extension.meta.name), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      _createVNode(MkKeyValue, null, {
                        key: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.author), 1 /* TEXT */)
                        ]),
                        value: _withCtx(() => [
                          _createTextVNode(_toDisplayString(__props.extension.meta.author), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(MkKeyValue, null, {
                    key: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._externalResourceInstaller._meta.base), 1 /* TEXT */)
                    ]),
                    value: _withCtx(() => [
                      _createTextVNode(_toDisplayString({ light: _unref(i18n).ts.light, dark: _unref(i18n).ts.dark, none: _unref(i18n).ts.none }[__props.extension.meta.base ?? 'none']), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["defaultOpen"]), _createVNode(MkFolder, { withSpacer: false }, {
              icon: _withCtx(() => [
                _hoisted_4
              ]),
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.code), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createVNode(MkCode, { code: __props.extension.raw }, null, 8 /* PROPS */, ["code"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["withSpacer"]) ])) : _createCommentVNode("v-if", true), _renderSlot(_ctx.$slots, "additionalInfo"), _createElementVNode("div", { class: "_buttonsCenter" }, [ _createVNode(MkButton, {
          danger: "",
          rounded: "",
          large: "",
          onClick: _cache[0] || (_cache[0] = ($event: any) => (emits('cancel')))
        }, {
          default: _withCtx(() => [
            _hoisted_5,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.cancel), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkButton, {
          gradate: "",
          rounded: "",
          large: "",
          onClick: _cache[1] || (_cache[1] = ($event: any) => (emits('confirm')))
        }, {
          default: _withCtx(() => [
            _hoisted_6,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.install), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }) ]) ]))
}
}

})
