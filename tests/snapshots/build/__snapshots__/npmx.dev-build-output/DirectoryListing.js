import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "w-full flex justify-self-stretch items-center gap-2" }, " .. ")
const _hoisted_2 = { class: "flex-1" }
import type { PackageFileTree } from '#shared/types'
import type { RouteLocationRaw } from 'vue-router'
import type { RouteNamedMap } from 'vue-router/auto-routes'
import { ADDITIONAL_ICONS, getFileIcon } from '~/utils/file-icons'

export default /*@__PURE__*/_defineComponent({
  __name: 'DirectoryListing',
  props: {
    tree: { type: Array, required: true },
    currentPath: { type: String, required: true },
    baseUrl: { type: String, required: true },
    baseRoute: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
// Get the current directory's contents
const currentContents = computed(() => {
  if (!props.currentPath) {
    return props.tree
  }

  const parts = props.currentPath.split('/')
  let current: PackageFileTree[] | undefined = props.tree

  for (const part of parts) {
    const found: PackageFileTree | undefined = current?.find(n => n.name === part)
    if (!found || found.type === 'file') {
      return []
    }
    current = found.children
  }

  return current ?? []
})
// Get parent directory path
const parentPath = computed(() => {
  if (!props.currentPath) return null
  const parts = props.currentPath.split('/')
  if (parts.length <= 1) return ''
  return parts.slice(0, -1).join('/')
})
// Build route object for a path
function getCodeRoute(nodePath?: string): RouteLocationRaw {
  return {
    name: 'code',
    params: {
      org: props.baseRoute.params.org,
      packageName: props.baseRoute.params.packageName,
      version: props.baseRoute.params.version,
      filePath: nodePath ?? '',
    },
  }
}
const bytesFormatter = useBytesFormatter()

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")

  return (_openBlock(), _createElementBlock("div", { class: "directory-listing" }, [ (currentContents.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "py-20 text-center text-fg-muted"
        }, [ _createElementVNode("p", null, _toDisplayString(_ctx.$t('code.no_files')), 1 /* TEXT */) ])) : (_openBlock(), _createElementBlock("table", {
          key: 1,
          class: "w-full"
        }, [ _createElementVNode("thead", { class: "sr-only" }, [ _createElementVNode("tr", null, [ _createElementVNode("th", null, _toDisplayString(_ctx.$t('code.table.name')), 1 /* TEXT */), _createElementVNode("th", null, _toDisplayString(_ctx.$t('code.table.size')), 1 /* TEXT */) ]) ]), _createElementVNode("tbody", null, [ (parentPath.value !== null) ? (_openBlock(), _createElementBlock("tr", {
                key: 0,
                class: "border-b border-border hover:bg-bg-subtle transition-colors"
              }, [ _createElementVNode("td", { colspan: "2" }, [ _createVNode(_component_LinkBase, {
                    to: getCodeRoute(parentPath.value || undefined),
                    class: "py-2 px-4 font-mono text-sm w-full",
                    "no-underline": ""
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("svg", {
                        class: "size-[1em] me-1 shrink-0 text-yellow-600",
                        viewBox: "0 0 16 16",
                        fill: "currentColor",
                        "aria-hidden": "true"
                      }, [
                        _createElementVNode("use", { href: `/file-tree-sprite.svg#${_unref(ADDITIONAL_ICONS)['folder']}` }, null, 8 /* PROPS */, ["href"])
                      ]),
                      _hoisted_1
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"]) ]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n        " + "\n        "), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(currentContents.value, (node) => {
              return (_openBlock(), _createElementBlock("tr", {
                key: node.path,
                class: "border-b border-border hover:bg-bg-subtle transition-colors"
              }, [
                _createElementVNode("td", { colspan: "2" }, [
                  _createVNode(_component_LinkBase, {
                    to: getCodeRoute(node.path),
                    class: "py-2 px-4 font-mono text-sm w-full",
                    "no-underline": ""
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("svg", {
                        viewBox: "0 0 16 16",
                        fill: "currentColor",
                        class: _normalizeClass(["size-[1em] me-1 shrink-0", node.type === 'directory' ? 'text-yellow-600' : undefined]),
                        "aria-hidden": "true"
                      }, [
                        _createElementVNode("use", { href: `/file-tree-sprite.svg#${node.type === 'directory' ? _unref(ADDITIONAL_ICONS)['folder'] : _unref(getFileIcon)(node.name)}` }, null, 8 /* PROPS */, ["href"])
                      ], 2 /* CLASS */),
                      _createElementVNode("span", { class: "w-full flex justify-self-stretch items-center gap-2" }, [
                        _createElementVNode("span", _hoisted_2, _toDisplayString(node.name), 1 /* TEXT */),
                        (typeof node.size === 'number')
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "text-end text-xs text-fg-subtle"
                          }, _toDisplayString(_unref(bytesFormatter).format(node.size)), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true)
                      ])
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["to"])
                ])
              ]))
            }), 128 /* KEYED_FRAGMENT */)) ]) ])), _createTextVNode("\n\n    " + "\n    ") ]))
}
}

})
