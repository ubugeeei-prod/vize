import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText, vModelSelect as _vModelSelect } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide-lightbulb w-3.5 h-3.5" })
const _hoisted_2 = { class: "text-green-500" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_4 = { class: "text-red-500" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle" }, "/")
const _hoisted_6 = { class: "text-yellow-500" }
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:triangle-alert w-3.5 h-3.5 text-yellow-500 shrink-0 mt-0.5" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chevron-right w-3.5 h-3.5 transition-transform group-open:rotate-90" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:file-text w-3.5 h-3.5" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:chevron-right w-3.5 h-3.5 transition-transform group-open:rotate-90" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", { class: "absolute inset-is-2 top-1/2 -translate-y-1/2 i-lucide:search w-3 h-3 text-fg-subtle pointer-events-none" })
const _hoisted_12 = { value: "all" }
const _hoisted_13 = { value: "added" }
const _hoisted_14 = { value: "removed" }
const _hoisted_15 = { value: "modified" }
import type { CompareResponse, FileChange } from '#shared/types'
import { packageRoute } from '~/utils/router'

export default /*@__PURE__*/_defineComponent({
  __name: 'SidebarPanel',
  props: {
    compare: { type: null, required: true },
    groupedDeps: { type: Map, required: true },
    allChanges: { type: Array, required: true },
    showSettings: { type: Boolean, required: false },
    "selectedFile": { default: null },
    "fileFilter": {
  default: 'all',
}
  },
  emits: ["file-select", "update:selectedFile", "update:fileFilter"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const selectedFile = _useModel(__props, "selectedFile")
const fileFilter = _useModel(__props, "fileFilter")
const sectionOrder = ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies']
const { t } = useI18n()
const sectionMeta = computed<Record<string, { label: string; icon: string }>>(() => ({
  dependencies: { label: t('compare.dependencies'), icon: 'i-lucide:box' },
  devDependencies: { label: t('compare.dev_dependencies'), icon: 'i-lucide:wrench' },
  peerDependencies: { label: t('compare.peer_dependencies'), icon: 'i-lucide:users' },
  optionalDependencies: { label: t('compare.optional_dependencies'), icon: 'i-lucide:circle-help' },
}))
const sectionList = computed(() => {
  const entries = Array.from(props.groupedDeps.entries())
  return entries
    .map(([key, changes]) => ({
      key,
      changes,
      label: sectionMeta.value[key]?.label ?? key,
      icon: sectionMeta.value[key]?.icon ?? 'i-lucide:box',
      order: sectionOrder.indexOf(key) === -1 ? sectionOrder.length + 1 : sectionOrder.indexOf(key),
    }))
    .sort((a, b) => a.order - b.order)
})
const fileSearch = ref('')
const filteredChanges = computed(() => {
  let files = props.allChanges
  if (fileFilter.value !== 'all') {
    files = files.filter(f => f.type === fileFilter.value)
  }
  if (fileSearch.value.trim()) {
    const query = fileSearch.value.trim().toLowerCase()
    files = files.filter(f => f.path.toLowerCase().includes(query))
  }
  return files
})
function getSemverBadgeClass(semverDiff: string | null | undefined): string {
  switch (semverDiff) {
    case 'major':
      return 'bg-red-500/10 text-red-500'
    case 'minor':
      return 'bg-yellow-500/10 text-yellow-500'
    case 'patch':
      return 'bg-green-500/10 text-green-500'
    case 'prerelease':
      return 'bg-purple-500/10 text-purple-500'
    default:
      return 'bg-bg-muted text-fg-subtle'
  }
}
function handleFileSelect(file: FileChange) {
  selectedFile.value = file
  emit('file-select', file)
}

return (_ctx: any,_cache: any) => {
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_DiffFileTree = _resolveComponent("DiffFileTree")

  return (_openBlock(), _createElementBlock("div", { class: "flex flex-col min-h-0" }, [ _createElementVNode("div", { class: "border-b border-border shrink-0" }, [ _createElementVNode("div", { class: "px-3 py-2.5 border-b border-border" }, [ _createElementVNode("div", { class: "flex flex-wrap items-center justify-between gap-2" }, [ _createElementVNode("span", { class: "text-xs font-medium flex items-center gap-1.5" }, [ _hoisted_1, _createTextVNode("\n            " + _toDisplayString(_ctx.$t('compare.summary')), 1 /* TEXT */) ]), _createElementVNode("div", { class: "flex items-center gap-3 font-mono text-3xs" }, [ _createElementVNode("span", { class: "flex items-center gap-1" }, [ _createElementVNode("span", _hoisted_2, "+" + _toDisplayString(__props.compare.stats.filesAdded), 1 /* TEXT */), _hoisted_3, _createElementVNode("span", _hoisted_4, "-" + _toDisplayString(__props.compare.stats.filesRemoved), 1 /* TEXT */), _hoisted_5, _createElementVNode("span", _hoisted_6, "~" + _toDisplayString(__props.compare.stats.filesModified), 1 /* TEXT */) ]), (__props.compare.dependencyChanges.length > 0) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: "text-fg-muted"
                }, _toDisplayString(_ctx.$t('compare.deps_count', { count: __props.compare.dependencyChanges.length })), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]) ]) ]), (__props.compare.meta.warnings?.length) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "px-3 py-2 bg-yellow-500/5 border-b border-border"
          }, [ _createElementVNode("div", { class: "flex items-start gap-2" }, [ _hoisted_7, _createElementVNode("div", { class: "text-3xs text-fg-muted" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.compare.meta.warnings, (warning) => {
                  return (_openBlock(), _createElementBlock("p", { key: warning }, _toDisplayString(warning), 1 /* TEXT */))
                }), 128 /* KEYED_FRAGMENT */)) ]) ]) ])) : _createCommentVNode("v-if", true), (__props.compare.dependencyChanges.length > 0) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "px-3 py-2.5 space-y-2"
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(sectionList.value, (section) => {
              return (_openBlock(), _createElementBlock("details", {
                key: section.key,
                class: "group"
              }, [
                _createElementVNode("summary", { class: "cursor-pointer list-none flex items-center gap-2 text-xs font-medium mb-2 hover:text-fg transition-colors" }, [
                  _hoisted_8,
                  _createElementVNode("span", {
                    class: _normalizeClass(["w-3.5 h-3.5", section.icon])
                  }, null, 2 /* CLASS */),
                  _createTextVNode("\n            " + _toDisplayString(section.label) + " (" + _toDisplayString(section.changes.length) + ")\n          ", 1 /* TEXT */)
                ]),
                _createElementVNode("div", { class: "space-y-1 ms-5 max-h-40 overflow-y-auto" }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(section.changes, (dep) => {
                    return (_openBlock(), _createElementBlock("div", {
                      key: dep.name,
                      class: "flex items-center gap-2 text-xs py-0.5"
                    }, [
                      _createElementVNode("span", {
                        class: _normalizeClass([
                    'w-3 h-3 shrink-0',
                    dep.type === 'added'
                      ? 'i-lucide:plus text-green-500'
                      : dep.type === 'removed'
                        ? 'i-lucide:minus text-red-500'
                        : 'i-lucide:arrow-left-right text-yellow-500',
                  ])
                      }, null, 2 /* CLASS */),
                      _createVNode(_component_NuxtLink, {
                        to: _unref(packageRoute)(dep.name),
                        class: "font-mono hover:text-fg transition-colors truncate min-w-0"
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(dep.name), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["to"]),
                      _createElementVNode("div", { class: "flex items-center gap-1.5 text-fg-muted font-mono text-3xs ms-auto shrink-0" }, [
                        (dep.from)
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: _normalizeClass({ 'line-through opacity-50': dep.type === 'updated' })
                          }, _toDisplayString(dep.from), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true),
                        (dep.type === 'updated')
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "i-lucide:arrow-right w-2.5 h-2.5"
                          }))
                          : _createCommentVNode("v-if", true),
                        (dep.to)
                          ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(dep.to), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true)
                      ]),
                      (dep.semverDiff)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          class: _normalizeClass(["text-4xs px-1.5 py-0.5 rounded font-medium shrink-0", getSemverBadgeClass(dep.semverDiff)])
                        }, _toDisplayString(dep.semverDiff), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ]))
                  }), 128 /* KEYED_FRAGMENT */))
                ])
              ]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (__props.compare.dependencyChanges.length === 0 && !__props.compare.meta.warnings?.length) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "px-3 py-2 text-3xs text-fg-muted text-center"
          }, _toDisplayString(_ctx.$t('compare.no_dependency_changes')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("details", {
        class: "flex-1 flex flex-col open:flex-1 group min-h-0",
        open: ""
      }, [ _createElementVNode("summary", { class: "border-b border-border px-3 py-2 shrink-0 cursor-pointer list-none flex items-center justify-between gap-2" }, [ _createElementVNode("span", { class: "text-xs font-medium flex items-center gap-1.5" }, [ _hoisted_9, _createTextVNode("\n          " + _toDisplayString(_ctx.$t('compare.file_changes')), 1 /* TEXT */) ]), _hoisted_10 ]), _createElementVNode("div", { class: "border-b border-border px-3 py-2 shrink-0 space-y-2" }, [ _createElementVNode("div", { class: "relative" }, [ _hoisted_11, _withDirectives(_createElementVNode("input", {
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((fileSearch).value = $event)),
              type: "search",
              placeholder: _ctx.$t('compare.search_files_placeholder'),
              "aria-label": _ctx.$t('compare.search_files_placeholder'),
              class: "w-full text-2xs ps-6.5 pe-2 py-1 bg-bg-subtle border border-border rounded font-mono placeholder:text-fg-subtle transition-colors hover:border-border-hover focus:border-accent focus:outline-none"
            }, null, 8 /* PROPS */, ["placeholder", "aria-label"]), [ [_vModelText, fileSearch.value] ]) ]), _createElementVNode("div", { class: "flex items-center justify-end" }, [ _withDirectives(_createElementVNode("select", {
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((fileFilter).value = $event)),
              "aria-label": _ctx.$t('compare.filter_files_label'),
              class: "text-3xs px-2 py-1 bg-bg-subtle border border-border rounded font-mono cursor-pointer hover:border-border-hover transition-colors"
            }, [ _createElementVNode("option", _hoisted_12, _toDisplayString(_ctx.$t('compare.file_filter_option.all', { count: __props.allChanges.length })), 1 /* TEXT */), _createElementVNode("option", _hoisted_13, _toDisplayString(_ctx.$t('compare.file_filter_option.added', { count: __props.compare.stats.filesAdded })), 1 /* TEXT */), _createElementVNode("option", _hoisted_14, _toDisplayString(_ctx.$t('compare.file_filter_option.removed', { count: __props.compare.stats.filesRemoved })), 1 /* TEXT */), _createElementVNode("option", _hoisted_15, _toDisplayString(_ctx.$t('compare.file_filter_option.modified', { count: __props.compare.stats.filesModified })), 1 /* TEXT */) ], 8 /* PROPS */, ["aria-label"]), [ [_vModelSelect, fileFilter.value] ]) ]) ]), _createElementVNode("div", { class: "flex-1 overflow-y-auto min-h-0" }, [ (filteredChanges.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "p-8 text-center text-xs text-fg-muted"
            }, _toDisplayString(fileSearch.value.trim() ? _ctx.$t('compare.no_files_search', { query: fileSearch.value.trim() }) : fileFilter.value === 'all' ? _ctx.$t('compare.no_files_all') : fileFilter.value === 'added' ? _ctx.$t('compare.no_files_filtered', { filter: _ctx.$t('compare.filter.added') }) : fileFilter.value === 'removed' ? _ctx.$t('compare.no_files_filtered', { filter: _ctx.$t('compare.filter.removed') }) : _ctx.$t('compare.no_files_filtered', { filter: _ctx.$t('compare.filter.modified') })), 1 /* TEXT */)) : (_openBlock(), _createBlock(_component_DiffFileTree, {
              key: 1,
              files: filteredChanges.value,
              "selected-path": selectedFile.value?.path ?? null,
              onSelect: handleFileSelect
            }, null, 8 /* PROPS */, ["files", "selected-path"])) ]) ]) ]))
}
}

})
