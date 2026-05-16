import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'Compatibility',
  props: {
    engines: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const engineNames: Record<string, string> = {
  bun: 'Bun',
  node: 'Node.js',
  npm: 'npm',
}
// Map engine name to icon class
const engineIcons: Record<string, string> = {
  bun: 'i-simple-icons:bun',
  node: 'i-simple-icons:nodedotjs',
  npm: 'i-simple-icons:npm',
  pnpm: 'i-simple-icons:pnpm',
  yarn: 'i-simple-icons:yarn',
}
function getName(engine: string): string {
  return engineNames[engine] || engine
}
function getIcon(engine: string): string {
  return engineIcons[engine] || 'i-lucide:code'
}
const sortedEngines = computed(() => {
  const entries = Object.entries(props.engines ?? {})
  return entries.sort(([a], [b]) => a.localeCompare(b))
})

return (_ctx: any,_cache: any) => {
  const _component_CollapsibleSection = _resolveComponent("CollapsibleSection")

  return (sortedEngines.value.length)
      ? (_openBlock(), _createBlock(_component_CollapsibleSection, {
        key: 0,
        title: _ctx.$t('package.compatibility'),
        id: "compatibility"
      }, {
        default: _withCtx(() => [
          _createElementVNode("dl", { class: "space-y-2" }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(sortedEngines.value, ([engine, version]) => {
              return (_openBlock(), _createElementBlock("div", {
                key: engine,
                class: "flex justify-between gap-4 py-1"
              }, [
                _createElementVNode("dt", { class: "flex items-center gap-2 text-fg-muted text-sm shrink-0" }, [
                  _createElementVNode("span", {
                    class: _normalizeClass([getIcon(engine), 'inline-block w-4 h-4 shrink-0 text-fg-muted']),
                    "aria-hidden": "true"
                  }),
                  _createTextVNode("\n          " + _toDisplayString(getName(engine)), 1 /* TEXT */)
                ]),
                _createElementVNode("dd", {
                  class: "font-mono text-sm text-fg text-end",
                  title: version,
                  dir: "ltr"
                }, _toDisplayString(version), 9 /* TEXT, PROPS */, ["title"])
              ]))
            }), 128 /* KEYED_FRAGMENT */))
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["title"]))
      : _createCommentVNode("v-if", true)
}
}

})
