import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"


const _hoisted_1 = { class: "font-mono text-xs text-fg-muted" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:circle-alert w-3 h-3" })
import { getOutdatedTooltip, getVersionClass } from '~/utils/npm/outdated-dependencies'
import type { RouteLocationRaw } from 'vue-router'

export default /*@__PURE__*/_defineComponent({
  __name: 'InstallScripts',
  props: {
    packageName: { type: String, required: true },
    version: { type: String, required: true },
    installScripts: { type: Object, required: true }
  },
  setup(__props: any) {

const props = __props
function getCodeLink(filePath: string): RouteLocationRaw {
  const split = props.packageName.split('/')
  return {
    name: 'code',
    params: {
      org: split.length === 2 ? split[0] : null,
      packageName: split.length === 2 ? split[1]! : split[0]!,
      version: props.version,
      filePath: filePath,
    },
  }
}
const scriptParts = computed(() => {
  const parts: Record<
    string,
    { prefix: string | null; filePath: string | null; link: RouteLocationRaw }
  > = {}
  for (const scriptName of props.installScripts.scripts) {
    const content = props.installScripts.content?.[scriptName]
    if (!content) continue
    const parsed = parseNodeScript(content)
    if (parsed) {
      parts[scriptName] = {
        prefix: parsed.prefix,
        filePath: parsed.filePath,
        link: getCodeLink(parsed.filePath),
      }
    } else {
      parts[scriptName] = { prefix: null, filePath: null, link: getCodeLink('package.json') }
    }
  }
  return parts
})
const outdatedNpxDeps = useOutdatedDependencies(() => props.installScripts.npxDependencies)
const hasNpxDeps = computed(() => Object.keys(props.installScripts.npxDependencies).length > 0)
const sortedNpxDeps = computed(() => {
  return Object.entries(props.installScripts.npxDependencies).sort(([a], [b]) => a.localeCompare(b))
})
const isExpanded = shallowRef(false)

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_CollapsibleSection = _resolveComponent("CollapsibleSection")

  return (_openBlock(), _createBlock(_component_CollapsibleSection, {
      title: _ctx.$t('package.install_scripts.title'),
      id: "installScripts",
      icon: "i-lucide:circle-alert w-3 h-3 text-yellow-500"
    }, {
      default: _withCtx(() => [
        _createElementVNode("dl", { class: "space-y-2 m-0" }, [
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.installScripts.scripts, (scriptName) => {
            return (_openBlock(), _createElementBlock("div", { key: scriptName }, [
              _createElementVNode("dt", _hoisted_1, _toDisplayString(scriptName), 1 /* TEXT */),
              _createElementVNode("dd", {
                class: "font-mono text-sm text-fg-subtle m-0 truncate",
                title: __props.installScripts.content?.[scriptName]
              }, [
                (__props.installScripts.content?.[scriptName] && scriptParts.value[scriptName])
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    (scriptParts.value[scriptName].prefix)
                      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                        _toDisplayString(scriptParts.value[scriptName].prefix),
                        _createVNode(_component_LinkBase, { to: scriptParts.value[scriptName].link }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(scriptParts.value[scriptName].filePath), 1 /* TEXT */)
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 8 /* PROPS */, ["to"])
                      ], 64 /* STABLE_FRAGMENT */))
                      : (_openBlock(), _createBlock(_component_LinkBase, {
                        key: 1,
                        to: scriptParts.value[scriptName].link
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(__props.installScripts.content[scriptName]), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["to"]))
                  ], 64 /* STABLE_FRAGMENT */))
                  : (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    tabindex: "0",
                    class: "cursor-help"
                  }, _toDisplayString(_ctx.$t('package.install_scripts.script_label')), 1 /* TEXT */))
              ], 8 /* PROPS */, ["title"])
            ]))
          }), 128 /* KEYED_FRAGMENT */))
        ]),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    "),
        (hasNpxDeps.value)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "mt-3"
          }, [
            _createElementVNode("button", {
              type: "button",
              class: "flex items-center gap-1.5 text-xs text-fg-muted hover:text-fg transition-colors duration-200 focus-visible:outline-accent/70 rounded",
              "aria-expanded": isExpanded.value,
              "aria-controls": "npx-packages-details",
              onClick: _cache[0] || (_cache[0] = ($event: any) => (isExpanded.value = !isExpanded.value))
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(["i-lucide:chevron-right rtl-flip w-3 h-3 transition-transform duration-200", { 'rotate-90': isExpanded.value }]),
                "aria-hidden": "true"
              }, null, 2 /* CLASS */),
              _createTextVNode("\n        " + _toDisplayString(_ctx.$t(
              'package.install_scripts.npx_packages',
              { count: sortedNpxDeps.value.length },
              sortedNpxDeps.value.length,
            )), 1 /* TEXT */)
            ], 8 /* PROPS */, ["aria-expanded"]),
            _withDirectives(_createElementVNode("ul", {
              id: "npx-packages-details",
              class: "mt-2 space-y-1 list-none m-0 p-0 ps-4"
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(sortedNpxDeps.value, ([dep, version]) => {
                return (_openBlock(), _createElementBlock("li", {
                  key: dep,
                  class: "flex items-center justify-between py-0.5 text-sm gap-2"
                }, [
                  _createVNode(_component_LinkBase, {
                    to: _ctx.packageRoute(dep),
                    class: "font-mono text-fg-muted hover:text-fg transition-colors duration-200 truncate min-w-0"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(dep), 1 /* TEXT */)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to"]),
                  _createElementVNode("span", { class: "flex items-center gap-1" }, [
                    (
                  _unref(outdatedNpxDeps)[dep] &&
                  _unref(outdatedNpxDeps)[dep].resolved !== _unref(outdatedNpxDeps)[dep].latest
                )
                      ? (_openBlock(), _createBlock(_component_TooltipApp, {
                        key: 0,
                        class: _normalizeClass(["shrink-0 p-2 -m-2", _unref(getVersionClass)(_unref(outdatedNpxDeps)[dep])]),
                        "aria-hidden": "true",
                        text: _unref(getOutdatedTooltip)(_unref(outdatedNpxDeps)[dep], _ctx.$t)
                      }, {
                        default: _withCtx(() => [
                          _hoisted_2
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 10 /* CLASS, PROPS */, ["text"]))
                      : _createCommentVNode("v-if", true),
                    _createElementVNode("span", {
                      class: _normalizeClass(["font-mono text-xs text-end truncate", _unref(getVersionClass)(_unref(outdatedNpxDeps)[dep])]),
                      title: 
                  _unref(outdatedNpxDeps)[dep]
                    ? _unref(outdatedNpxDeps)[dep].resolved === _unref(outdatedNpxDeps)[dep].latest
                      ? _ctx.$t('package.install_scripts.currently', {
                          version: _unref(outdatedNpxDeps)[dep].latest,
                        })
                      : _unref(getOutdatedTooltip)(_unref(outdatedNpxDeps)[dep], _ctx.$t)
                    : __props.version
              
                    }, _toDisplayString(__props.version), 11 /* TEXT, CLASS, PROPS */, ["title"])
                  ])
                ]))
              }), 128 /* KEYED_FRAGMENT */))
            ], 512 /* NEED_PATCH */), [
              [_vShow, isExpanded.value]
            ])
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["title"]))
}
}

})
