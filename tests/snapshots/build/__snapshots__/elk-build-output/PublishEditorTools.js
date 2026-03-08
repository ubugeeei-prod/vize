import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:font-size-2": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:code-s-slash-line": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:bold": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:italic": "true" })
import type { Editor } from '@tiptap/core'

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishEditorTools',
  props: {
    editor: { type: null, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_VDropdown = _resolveComponent("VDropdown")

  return (_openBlock(), _createBlock(_component_CommonTooltip, {
      placement: "top",
      content: _ctx.$t('tooltip.open_editor_tools')
    }, {
      default: _withCtx(() => [
        (__props.editor)
          ? (_openBlock(), _createBlock(_component_VDropdown, {
            key: 0,
            placement: "bottom"
          }, {
            popper: _withCtx(() => [
              _createElementVNode("div", {
                flex: "",
                "gap-1": ""
              }, [
                _createVNode(_component_CommonTooltip, {
                  placement: "top",
                  content: _ctx.$t('tooltip.toggle_code_block')
                }, {
                  default: _withCtx(() => [
                    _createElementVNode("button", {
                      "btn-action-icon": "",
                      "aria-label": _ctx.$t('tooltip.toggle_code_block'),
                      class: _normalizeClass(__props.editor.isActive('codeBlock') ? 'text-primary' : ''),
                      onClick: _cache[0] || (_cache[0] = ($event: any) => (__props.editor?.chain().focus().toggleCodeBlock().run()))
                    }, [
                      _hoisted_2
                    ], 10 /* CLASS, PROPS */, ["aria-label"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["content"]),
                _createVNode(_component_CommonTooltip, {
                  placement: "top",
                  content: _ctx.$t('tooltip.toggle_bold')
                }, {
                  default: _withCtx(() => [
                    _createElementVNode("button", {
                      "btn-action-icon": "",
                      "aria-label": _ctx.$t('tooltip.toggle_bold'),
                      class: _normalizeClass(__props.editor.isActive('bold') ? 'text-primary' : ''),
                      onClick: _cache[1] || (_cache[1] = ($event: any) => (__props.editor?.chain().focus().toggleBold().run()))
                    }, [
                      _hoisted_3
                    ], 10 /* CLASS, PROPS */, ["aria-label"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["content"]),
                _createVNode(_component_CommonTooltip, {
                  placement: "top",
                  content: _ctx.$t('tooltip.toggle_italic')
                }, {
                  default: _withCtx(() => [
                    _createElementVNode("button", {
                      "btn-action-icon": "",
                      "aria-label": _ctx.$t('tooltip.toggle_italic'),
                      class: _normalizeClass(__props.editor.isActive('italic') ? 'text-primary' : ''),
                      onClick: _cache[2] || (_cache[2] = ($event: any) => (__props.editor?.chain().focus().toggleItalic().run()))
                    }, [
                      _hoisted_4
                    ], 10 /* CLASS, PROPS */, ["aria-label"])
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["content"])
              ])
            ]),
            default: _withCtx(() => [
              _createElementVNode("button", {
                "btn-action-icon": "",
                "aria-label": _ctx.$t('tooltip.open_editor_tools')
              }, [
                _hoisted_1
              ], 8 /* PROPS */, ["aria-label"])
            ]),
            _: 1 /* STABLE */
          }))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["content"]))
}
}

})
