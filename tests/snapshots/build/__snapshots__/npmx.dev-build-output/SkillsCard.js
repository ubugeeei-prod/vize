import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-custom:agent-skills w-4 h-4 shrink-0 text-fg-muted", "aria-hidden": "true" })
const _hoisted_2 = { class: "text-fg-muted" }
import type { SkillListItem } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'SkillsCard',
  props: {
    skills: { type: Array, required: true },
    packageName: { type: String, required: true },
    version: { type: String, required: false }
  },
  setup(__props: any) {

const skillsModal = useModal('skills-modal')

return (_ctx: any,_cache: any) => {
  const _component_CollapsibleSection = _resolveComponent("CollapsibleSection")

  return (__props.skills.length)
      ? (_openBlock(), _createBlock(_component_CollapsibleSection, {
        key: 0,
        title: _ctx.$t('package.skills.title'),
        id: "skills"
      }, {
        default: _withCtx(() => [
          _createElementVNode("button", {
            type: "button",
            class: "w-full flex items-center gap-2 px-3 py-2 text-sm font-mono bg-bg-muted border border-border rounded-md hover:border-border-hover hover:bg-bg-elevated focus-visible:outline-accent/70 transition-colors duration-200",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(skillsModal).open()))
          }, [
            _hoisted_1,
            _createElementVNode("span", _hoisted_2, _toDisplayString(_ctx.$t('package.skills.skills_available', { count: __props.skills.length }, __props.skills.length)), 1 /* TEXT */)
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["title"]))
      : _createCommentVNode("v-if", true)
}
}

})
