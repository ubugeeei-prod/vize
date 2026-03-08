import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("code", { class: "font-mono text-fg" }, "skills-npm")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-right w-3 h-3" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "w-2.5 h-2.5 rounded-full bg-fg-subtle" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle select-none" }, "$ ")
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg" }, "npx ")
const _hoisted_9 = { class: "text-fg-muted" }
const _hoisted_10 = { "aria-live": "polite" }
const _hoisted_11 = { class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_12 = { class: "text-xs text-fg-subtle/60" }
const _hoisted_13 = { class: "font-mono text-sm text-fg-muted" }
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:circle-alert w-3.5 h-3.5 text-amber-500" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:file-code size-3 align-[-2px] me-0.5" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:file-text size-3 align-[-2px] me-0.5" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:image size-3 align-[-2px] me-0.5" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:circle-alert size-3 align-[-2px] me-0.5" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:code size-3" })
import type { SkillListItem } from '#shared/types'

type InstallMethod = 'skills-npm' | 'skills-cli'

export default /*@__PURE__*/_defineComponent({
  __name: 'SkillsModal',
  props: {
    skills: { type: Array, required: true },
    packageName: { type: String, required: true },
    version: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
function getSkillSourceUrl(skill: SkillListItem): string {
  const base = `/package-code/${props.packageName}`
  const versionPath = props.version ? `/v/${props.version}` : ''
  return `${base}${versionPath}/skills/${skill.dirName}/SKILL.md`
}
const expandedSkills = ref<Set<string>>(new Set())
function toggleSkill(dirName: string) {
  if (expandedSkills.value.has(dirName)) {
    expandedSkills.value.delete(dirName)
  } else {
    expandedSkills.value.add(dirName)
  }
  expandedSkills.value = new Set(expandedSkills.value)
}
const selectedMethod = ref<InstallMethod>('skills-npm')
const baseUrl = computed(() =>
  typeof window !== 'undefined' ? window.location.origin : 'https://npmx.dev',
)
const installCommand = computed(() => {
  if (!props.skills.length) return null
  return `npx skills add ${baseUrl.value}/${props.packageName}`
})
const { copied, copy } = useClipboard({ copiedDuring: 2000 })
const copyCommand = () => installCommand.value && copy(installCommand.value)
function getWarningTooltip(skill: SkillListItem): string | undefined {
  if (!skill.warnings?.length) return undefined
  return skill.warnings.map(w => w.message).join(', ')
}

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_Modal = _resolveComponent("Modal")

  return (_openBlock(), _createBlock(_component_Modal, {
      "modal-title": _ctx.$t('package.skills.title'),
      id: "skills-modal",
      class: "sm:max-w-2xl"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "flex flex-wrap items-center justify-between gap-2 mb-3" }, [
          _createElementVNode("h3", _hoisted_1, _toDisplayString(_ctx.$t('package.skills.install')), 1 /* TEXT */),
          _createElementVNode("div", {
            class: "flex items-center gap-1 p-0.5 bg-bg-subtle border border-border-subtle rounded-md",
            role: "tablist",
            "aria-label": _ctx.$t('package.skills.installation_method')
          }, [
            _createElementVNode("button", {
              role: "tab",
              "aria-selected": selectedMethod.value === 'skills-npm',
              tabindex: selectedMethod.value === 'skills-npm' ? 0 : -1,
              type: "button",
              class: _normalizeClass(["px-2 py-1 font-mono text-xs rounded transition-colors duration-150 border border-solid focus-visible:outline-accent/70", 
              selectedMethod.value === 'skills-npm'
                ? 'bg-bg border-border shadow-sm text-fg'
                : 'border-transparent text-fg-subtle hover:text-fg'
            ]),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (selectedMethod.value = 'skills-npm'))
            }, "\n          skills-npm\n        ", 10 /* CLASS, PROPS */, ["aria-selected", "tabindex"]),
            _createElementVNode("button", {
              role: "tab",
              "aria-selected": selectedMethod.value === 'skills-cli',
              tabindex: selectedMethod.value === 'skills-cli' ? 0 : -1,
              type: "button",
              class: _normalizeClass(["px-2 py-1 font-mono text-xs rounded transition-colors duration-150 border border-solid focus-visible:outline-accent/70", 
              selectedMethod.value === 'skills-cli'
                ? 'bg-bg border-border shadow-sm text-fg'
                : 'border-transparent text-fg-subtle hover:text-fg'
            ]),
              onClick: _cache[1] || (_cache[1] = ($event: any) => (selectedMethod.value = 'skills-cli'))
            }, "\n          skills CLI\n        ", 10 /* CLASS, PROPS */, ["aria-selected", "tabindex"])
          ], 8 /* PROPS */, ["aria-label"])
        ]),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    "),
        (selectedMethod.value === 'skills-npm')
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "flex items-center justify-between gap-2 px-3 py-2.5 sm:px-4 bg-bg-subtle border border-border rounded-lg mb-5"
          }, [
            _createVNode(_component_i18n_t, {
              keypath: "package.skills.compatible_with",
              tag: "span",
              class: "text-sm text-fg-muted",
              scope: "global"
            }, {
              tool: _withCtx(() => [
                _hoisted_2
              ]),
              _: 1 /* STABLE */
            }),
            _createElementVNode("a", {
              href: "/package/skills-npm",
              class: "inline-flex items-center gap-1 text-xs text-fg-subtle hover:text-fg transition-colors shrink-0"
            }, [
              _createTextVNode(_toDisplayString(_ctx.$t('package.skills.learn_more')) + "\n        ", 1 /* TEXT */),
              _hoisted_3
            ])
          ]))
          : (installCommand.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "bg-bg-subtle border border-border rounded-lg overflow-hidden mb-5"
            }, [
              _createElementVNode("div", { class: "flex gap-1.5 px-3 pt-2 sm:px-4 sm:pt-3" }, [
                _hoisted_4,
                _hoisted_5,
                _hoisted_6
              ]),
              _createElementVNode("div", { class: "px-3 pt-2 pb-3 sm:px-4 sm:pt-3 sm:pb-4 overflow-x-auto" }, [
                _createElementVNode("div", { class: "relative group/cmd" }, [
                  _createElementVNode("code", { class: "font-mono text-sm whitespace-nowrap" }, [
                    _hoisted_7,
                    _hoisted_8,
                    _createElementVNode("span", _hoisted_9, "skills add " + _toDisplayString(baseUrl.value) + "/" + _toDisplayString(__props.packageName), 1 /* TEXT */)
                  ]),
                  _createElementVNode("button", {
                    type: "button",
                    class: "absolute top-0 inset-ie-0 px-2 py-0.5 font-mono text-xs text-fg-muted bg-bg-subtle/80 border border-border rounded transition-colors duration-200 opacity-0 group-hover/cmd:opacity-100 hover:(text-fg border-border-hover) active:scale-95 focus-visible:opacity-100 focus-visible:outline-accent/70",
                    "aria-label": _ctx.$t('package.get_started.copy_command'),
                    onClick: _withModifiers(copyCommand, ["stop"])
                  }, [
                    _createElementVNode("span", _hoisted_10, _toDisplayString(_unref(copied) ? _ctx.$t('common.copied') : _ctx.$t('common.copy')), 1 /* TEXT */)
                  ], 8 /* PROPS */, ["aria-label"])
                ])
              ])
            ]))
          : _createCommentVNode("v-if", true),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    "),
        _createTextVNode("\n\n    "),
        _createTextVNode("\n    "),
        _createElementVNode("div", { class: "flex items-baseline justify-between gap-2 mb-2" }, [
          _createElementVNode("h3", _hoisted_11, _toDisplayString(_ctx.$t('package.skills.available_skills')), 1 /* TEXT */),
          _createElementVNode("span", _hoisted_12, _toDisplayString(_ctx.$t('package.skills.click_to_expand')), 1 /* TEXT */)
        ]),
        _createElementVNode("ul", { class: "space-y-0.5 list-none m-0 p-0" }, [
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.skills, (skill) => {
            return (_openBlock(), _createElementBlock("li", { key: skill.dirName }, [
              _createElementVNode("button", {
                type: "button",
                class: "w-full flex items-center gap-2 py-1.5 text-start rounded transition-colors hover:bg-bg-subtle focus-visible:outline-accent/70",
                "aria-expanded": expandedSkills.value.has(skill.dirName),
                onClick: _cache[2] || (_cache[2] = ($event: any) => (toggleSkill(skill.dirName)))
              }, [
                _createElementVNode("span", {
                  class: _normalizeClass(["i-lucide:chevron-right w-3 h-3 text-fg-subtle shrink-0 transition-transform duration-200", { 'rotate-90': expandedSkills.value.has(skill.dirName) }]),
                  "aria-hidden": "true"
                }, null, 2 /* CLASS */),
                _createElementVNode("span", _hoisted_13, _toDisplayString(skill.name), 1 /* TEXT */),
                (skill.warnings?.length)
                  ? (_openBlock(), _createBlock(_component_TooltipApp, {
                    key: 0,
                    class: "shrink-0 p-2 -m-2",
                    "aria-hidden": "true",
                    text: getWarningTooltip(skill),
                    to: "#skills-modal",
                    defer: ""
                  }, {
                    default: _withCtx(() => [
                      _hoisted_14
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["text"]))
                  : _createCommentVNode("v-if", true)
              ], 8 /* PROPS */, ["aria-expanded"]),
              _createTextVNode("\n\n        " + "\n        "),
              _createElementVNode("div", {
                class: _normalizeClass(["grid transition-[grid-template-rows] duration-200 ease-out", expandedSkills.value.has(skill.dirName) ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'])
              }, [
                _createElementVNode("div", { class: "overflow-hidden" }, [
                  _createElementVNode("div", { class: "ps-5.5 pe-2 pb-2 pt-1 space-y-1.5" }, [
                    (skill.description)
                      ? (_openBlock(), _createElementBlock("p", {
                        key: 0,
                        class: "text-sm text-fg-subtle"
                      }, _toDisplayString(skill.description), 1 /* TEXT */))
                      : (_openBlock(), _createElementBlock("p", {
                        key: 1,
                        class: "text-sm text-fg-subtle/50 italic"
                      }, _toDisplayString(_ctx.$t('package.skills.no_description')), 1 /* TEXT */)),
                    _createTextVNode("\n\n              " + "\n              "),
                    _createElementVNode("div", { class: "flex flex-wrap items-center gap-x-3 gap-y-1 text-xs" }, [
                      (skill.fileCounts?.scripts)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          class: "text-fg-subtle"
                        }, [
                          _hoisted_15,
                          _toDisplayString(_ctx.$t(
                        'package.skills.file_counts.scripts',
                        { count: skill.fileCounts.scripts },
                        skill.fileCounts.scripts,
                      ))
                        ]))
                        : _createCommentVNode("v-if", true),
                      (skill.fileCounts?.references)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          class: "text-fg-subtle"
                        }, [
                          _hoisted_16,
                          _toDisplayString(_ctx.$t(
                        'package.skills.file_counts.refs',
                        { count: skill.fileCounts.references },
                        skill.fileCounts.references,
                      ))
                        ]))
                        : _createCommentVNode("v-if", true),
                      (skill.fileCounts?.assets)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          class: "text-fg-subtle"
                        }, [
                          _hoisted_17,
                          _toDisplayString(_ctx.$t(
                        'package.skills.file_counts.assets',
                        { count: skill.fileCounts.assets },
                        skill.fileCounts.assets,
                      ))
                        ]))
                        : _createCommentVNode("v-if", true),
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(skill.warnings, (warning) => {
                        return (_openBlock(), _createElementBlock("span", {
                          key: warning.message,
                          class: "text-amber-500"
                        }, [
                          _hoisted_18,
                          _createTextVNode(_toDisplayString(warning.message), 1 /* TEXT */)
                        ]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]),
                    _createTextVNode("\n\n              " + "\n              "),
                    _createVNode(_component_NuxtLink, {
                      to: getSkillSourceUrl(skill),
                      class: "inline-flex items-center gap-1 text-xs text-fg-subtle hover:text-fg transition-colors",
                      onClick: _withModifiers(() => {}, ["stop"])
                    }, {
                      default: _withCtx(() => [
                        _hoisted_19,
                        _createTextVNode(_toDisplayString(_ctx.$t('package.skills.view_source')), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to", "onClick"])
                  ])
                ])
              ], 2 /* CLASS */)
            ]))
          }), 128 /* KEYED_FRAGMENT */))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["modal-title"]))
}
}

})
