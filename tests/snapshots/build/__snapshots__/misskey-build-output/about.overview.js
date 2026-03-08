import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-info-circle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user-shield" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-checkup-list" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-license" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-shield-lock" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-message" })
import { host, version } from '@@/js/config.js'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import number from '@/filters/number.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import FormLink from '@/components/form/link.vue'
import FormSection from '@/components/form/section.vue'
import FormSplit from '@/components/form/split.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkLink from '@/components/MkLink.vue'
import MkInfo from '@/components/MkInfo.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'about.overview',
  setup(__props) {

const initStats = () => misskeyApi('stats', {});

return (_ctx: any,_cache: any) => {
  const _component_MkSuspense = _resolveComponent("MkSuspense")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.banner),
        style: _normalizeStyle({ backgroundImage: `url(${ _unref(instance).bannerUrl })` })
      }, [ _createElementVNode("div", { style: "overflow: clip;" }, [ _createElementVNode("img", {
            src: _unref(instance).iconUrl ?? '/favicon.ico',
            alt: "",
            class: _normalizeClass(_ctx.$style.bannerIcon)
          }, null, 8 /* PROPS */, ["src"]), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.bannerName)
          }, [ _createElementVNode("b", null, _toDisplayString(_unref(instance).name ?? _unref(host)), 1 /* TEXT */) ]) ]) ], 4 /* STYLE */), _createVNode(MkKeyValue, null, {
        key: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.description), 1 /* TEXT */)
        ]),
        value: _withCtx(() => [
          _createElementVNode("div", { innerHTML: _unref(instance).description }, null, 8 /* PROPS */, ["innerHTML"])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(FormSection, null, {
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(MkKeyValue, { copy: _unref(version) }, {
              key: _withCtx(() => [
                _createTextVNode("Misskey")
              ]),
              value: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(version)), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["copy"]),
            _createElementVNode("div", { innerHTML: _unref(i18n).tsx.poweredByMisskeyDescription({ name: _unref(instance).name ?? _unref(host) }) }, null, 8 /* PROPS */, ["innerHTML"]),
            _createVNode(FormLink, { to: "/about-misskey" }, {
              icon: _withCtx(() => [
                _hoisted_1
              ]),
              default: _withCtx(() => [
                _createTextVNode("\n\t\t\t\t"),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.aboutMisskey), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            (_unref(instance).repositoryUrl || _unref(instance).providesTarball)
              ? (_openBlock(), _createBlock(FormLink, {
                key: 0,
                to: _unref(instance).repositoryUrl || `/tarball/misskey-${_unref(version)}.tar.gz`,
                external: ""
              }, {
                icon: _withCtx(() => [
                  _hoisted_2
                ]),
                default: _withCtx(() => [
                  _createTextVNode("\n\t\t\t\t"),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.sourceCode), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["to"]))
              : (_openBlock(), _createBlock(MkInfo, {
                key: 1,
                warn: ""
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.sourceCodeIsNotYetProvided), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
          ])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(FormSection, null, {
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(FormSplit, null, {
              default: _withCtx(() => [
                _createVNode(MkKeyValue, { copy: _unref(instance).maintainerName }, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.administrator), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    (_unref(instance).maintainerName)
                      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                        _toDisplayString(_unref(instance).maintainerName)
                      ], 64 /* STABLE_FRAGMENT */))
                      : (_openBlock(), _createElementBlock("span", {
                        key: 1,
                        style: "opacity: 0.7;"
                      }, "(" + _toDisplayString(_unref(i18n).ts.none) + ")", 1 /* TEXT */))
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["copy"]),
                _createVNode(MkKeyValue, { copy: _unref(instance).maintainerEmail }, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.contact), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    (_unref(instance).maintainerEmail)
                      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                        _toDisplayString(_unref(instance).maintainerEmail)
                      ], 64 /* STABLE_FRAGMENT */))
                      : (_openBlock(), _createElementBlock("span", {
                        key: 1,
                        style: "opacity: 0.7;"
                      }, "(" + _toDisplayString(_unref(i18n).ts.none) + ")", 1 /* TEXT */))
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["copy"]),
                _createVNode(MkKeyValue, null, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.inquiry), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    (_unref(instance).inquiryUrl)
                      ? (_openBlock(), _createBlock(MkLink, {
                        key: 0,
                        url: _unref(instance).inquiryUrl,
                        target: "_blank"
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(instance).inquiryUrl), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["url"]))
                      : (_openBlock(), _createElementBlock("span", {
                        key: 1,
                        style: "opacity: 0.7;"
                      }, "(" + _toDisplayString(_unref(i18n).ts.none) + ")", 1 /* TEXT */))
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _: 1 /* STABLE */
            }),
            _createElementVNode("div", { class: "_gaps_s" }, [
              (_unref(instance).impressumUrl)
                ? (_openBlock(), _createBlock(FormLink, {
                  key: 0,
                  to: _unref(instance).impressumUrl,
                  external: ""
                }, {
                  icon: _withCtx(() => [
                    _hoisted_3
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.impressum), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"]))
                : _createCommentVNode("v-if", true),
              (_unref(instance).serverRules.length > 0)
                ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                  icon: _withCtx(() => [
                    _hoisted_4
                  ]),
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.serverRules), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("ol", {
                      class: _normalizeClass(["_gaps_s", _ctx.$style.rules])
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(instance).serverRules, (item) => {
                        return (_openBlock(), _createElementBlock("li", {
                          key: item,
                          class: _normalizeClass(_ctx.$style.rule)
                        }, [
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.ruleText),
                            innerHTML: item
                          }, null, 8 /* PROPS */, ["innerHTML"])
                        ]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              (_unref(instance).tosUrl)
                ? (_openBlock(), _createBlock(FormLink, {
                  key: 0,
                  to: _unref(instance).tosUrl,
                  external: ""
                }, {
                  icon: _withCtx(() => [
                    _hoisted_5
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.termsOfService), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"]))
                : _createCommentVNode("v-if", true),
              (_unref(instance).privacyPolicyUrl)
                ? (_openBlock(), _createBlock(FormLink, {
                  key: 0,
                  to: _unref(instance).privacyPolicyUrl,
                  external: ""
                }, {
                  icon: _withCtx(() => [
                    _hoisted_6
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.privacyPolicy), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"]))
                : _createCommentVNode("v-if", true),
              (_unref(instance).feedbackUrl)
                ? (_openBlock(), _createBlock(FormLink, {
                  key: 0,
                  to: _unref(instance).feedbackUrl,
                  external: ""
                }, {
                  icon: _withCtx(() => [
                    _hoisted_7
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.feedback), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"]))
                : _createCommentVNode("v-if", true)
            ])
          ])
        ]),
        _: 1 /* STABLE */
      }), _createVNode(_component_MkSuspense, { p: initStats }, {
        default: _withCtx(({ result: stats }) => [
          _createVNode(FormSection, null, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.statistics), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createVNode(FormSplit, null, {
                default: _withCtx(() => [
                  _createVNode(MkKeyValue, null, {
                    key: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.users), 1 /* TEXT */)
                    ]),
                    value: _withCtx(() => [
                      _createTextVNode(_toDisplayString(number(stats.originalUsersCount)), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }),
                  _createVNode(MkKeyValue, null, {
                    key: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.notes), 1 /* TEXT */)
                    ]),
                    value: _withCtx(() => [
                      _createTextVNode(_toDisplayString(number(stats.originalNotesCount)), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          })
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["p"]), _createVNode(FormSection, null, {
        label: _withCtx(() => [
          _createTextVNode("Well-known resources")
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(FormLink, {
              to: "/.well-known/host-meta",
              external: ""
            }, {
              default: _withCtx(() => [
                _createTextVNode("host-meta")
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(FormLink, {
              to: "/.well-known/host-meta.json",
              external: ""
            }, {
              default: _withCtx(() => [
                _createTextVNode("host-meta.json")
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(FormLink, {
              to: "/.well-known/nodeinfo",
              external: ""
            }, {
              default: _withCtx(() => [
                _createTextVNode("nodeinfo")
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(FormLink, {
              to: "/robots.txt",
              external: ""
            }, {
              default: _withCtx(() => [
                _createTextVNode("robots.txt")
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(FormLink, {
              to: "/manifest.json",
              external: ""
            }, {
              default: _withCtx(() => [
                _createTextVNode("manifest.json")
              ]),
              _: 1 /* STABLE */
            })
          ])
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
