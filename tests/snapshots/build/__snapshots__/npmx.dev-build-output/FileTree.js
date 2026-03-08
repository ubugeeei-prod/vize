import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue"


const _hoisted_1 = { class: "truncate" }
const _hoisted_2 = { class: "truncate" }
import type { FileChange } from '#shared/types'

interface DiffTreeNode {
  name: string
  path: string
  type: 'file' | 'directory'
  changeType?: 'added' | 'removed' | 'modified'
  children?: DiffTreeNode[]
}

export default /*@__PURE__*/_defineComponent({
  __name: 'FileTree',
  props: {
    files: { type: Array, required: true },
    selectedPath: { type: String, required: true },
    treeNodes: { type: Array, required: false },
    depth: { type: Number, required: false }
  },
  emits: ["select"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const depth = computed(() => props.depth ?? 0)
// Sort: directories first, then alphabetically
function sortTree(nodes: DiffTreeNode[]): DiffTreeNode[] {
  return nodes
    .map(n => ({
      ...n,
      children: n.children ? sortTree(n.children) : undefined,
    }))
    .sort((a, b) => {
      if (a.type !== b.type) return a.type === 'directory' ? -1 : 1
      return a.name.localeCompare(b.name)
    })
}
// Build tree structure from flat file list (only at root level)
function buildTree(files: FileChange[]): DiffTreeNode[] {
  const root: DiffTreeNode[] = []
  for (const file of files) {
    const parts = file.path.split('/')
    let current = root
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]!
      const isFile = i === parts.length - 1
      const path = parts.slice(0, i + 1).join('/')
      let node = current.find(n => n.name === part)
      if (!node) {
        node = {
          name: part,
          path,
          type: isFile ? 'file' : 'directory',
          changeType: isFile ? file.type : undefined,
          children: isFile ? undefined : [],
        }
        current.push(node)
      }
      if (!isFile) {
        current = node.children!
      }
    }
  }
  return sortTree(root)
}
// Use provided tree nodes or build from files
const tree = computed(() => props.treeNodes ?? buildTree(props.files))
// Check if a node or any of its children is currently selected
function isNodeActive(node: DiffTreeNode): boolean {
  if (props.selectedPath === node.path) return true
  if (props.selectedPath?.startsWith(node.path + '/')) return true
  return false
}
const expandedDirs = ref<Set<string>>(new Set())
function collectDirs(nodes: DiffTreeNode[]) {
  for (const node of nodes) {
    if (node.type === 'directory') {
      expandedDirs.value.add(node.path)
      if (node.children) collectDirs(node.children)
    }
  }
}
// Auto-expand all directories eagerly (runs on both SSR and client)
watchEffect(() => {
  if (props.depth === undefined || props.depth === 0) {
    collectDirs(tree.value)
  }
})
function toggleDir(path: string) {
  if (expandedDirs.value.has(path)) {
    expandedDirs.value.delete(path)
  } else {
    expandedDirs.value.add(path)
  }
}
function isExpanded(path: string): boolean {
  return expandedDirs.value.has(path)
}
function getChangeIcon(type: 'added' | 'removed' | 'modified') {
  switch (type) {
    case 'added':
      return 'i-lucide:file-plus text-green-500'
    case 'removed':
      return 'i-lucide:file-minus text-red-500'
    case 'modified':
      return 'i-lucide:file-diff text-yellow-500'
  }
}
function handleFileClick(node: DiffTreeNode) {
  const file = props.files.find(f => f.path === node.path)
  if (file) emit('select', file)
}

return (_ctx: any,_cache: any) => {
  const _component_FileTree = _resolveComponent("FileTree")

  return (_openBlock(), _createElementBlock("ul", {
      class: _normalizeClass(["list-none m-0 p-0", depth.value === 0 ? 'py-2' : ''])
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(tree.value, (node) => {
        return (_openBlock(), _createElementBlock("li", { key: node.path }, [
          (node.type === 'directory')
            ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
              _createElementVNode("button", {
                type: "button",
                class: _normalizeClass(["w-full flex items-center gap-1.5 py-1.5 px-3 text-start font-mono text-sm transition-colors hover:bg-bg-muted", isNodeActive(node) ? 'text-fg' : 'text-fg-muted']),
                style: _normalizeStyle({ paddingLeft: `${depth.value * 12 + 12}px` }),
                onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleDir(node.path)))
              }, [
                _createElementVNode("span", {
                  class: _normalizeClass(["w-4 h-4 shrink-0 transition-transform", [isExpanded(node.path) ? 'i-lucide:chevron-down' : 'i-lucide:chevron-right']])
                }, null, 2 /* CLASS */),
                _createElementVNode("span", {
                  class: _normalizeClass(["w-4 h-4 shrink-0", 
                isExpanded(node.path)
                  ? 'i-lucide:folder-open text-yellow-500'
                  : 'i-lucide:folder text-yellow-600'
              ])
                }, null, 2 /* CLASS */),
                _createElementVNode("span", _hoisted_1, _toDisplayString(node.name), 1 /* TEXT */)
              ], 6 /* CLASS, STYLE */),
              (isExpanded(node.path) && node.children)
                ? (_openBlock(), _createBlock(_component_FileTree, {
                  key: 0,
                  files: __props.files,
                  "tree-nodes": node.children,
                  "selected-path": __props.selectedPath,
                  depth: depth.value + 1,
                  onSelect: _cache[1] || (_cache[1] = ($event: any) => (emit('select', $event)))
                }, null, 8 /* PROPS */, ["files", "tree-nodes", "selected-path", "depth"]))
                : _createCommentVNode("v-if", true)
            ], 64 /* STABLE_FRAGMENT */))
            : (_openBlock(), _createElementBlock("button", {
              key: 1,
              type: "button",
              class: _normalizeClass(["w-full flex items-center gap-1.5 py-1.5 px-3 font-mono text-sm transition-colors hover:bg-bg-muted text-start", __props.selectedPath === node.path ? 'bg-bg-muted text-fg' : 'text-fg-muted']),
              style: _normalizeStyle({ paddingLeft: `${depth.value * 12 + 32}px` }),
              onClick: _cache[2] || (_cache[2] = ($event: any) => (handleFileClick(node)))
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(["w-4 h-4 shrink-0", getChangeIcon(node.changeType)])
              }, null, 2 /* CLASS */),
              _createElementVNode("span", _hoisted_2, _toDisplayString(node.name), 1 /* TEXT */)
            ])),
          _createTextVNode("\n\n      " + "\n      ")
        ]))
      }), 128 /* KEYED_FRAGMENT */)) ], 2 /* CLASS */))
}
}

})
