import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, vModelSelect as _vModelSelect } from "vue"

import { NodeViewContent, nodeViewProps, NodeViewWrapper } from '@tiptap/vue-3'

export default /*@__PURE__*/_defineComponent({
  __name: 'TiptapCodeBlock',
  props: nodeViewProps,
  setup(__props: any) {

const languages = [
  'c',
  'cpp',
  'csharp',
  'css',
  'dart',
  'go',
  'html',
  'java',
  'javascript',
  'jsx',
  'kotlin',
  'python',
  'rust',
  'svelte',
  'swift',
  'tsx',
  'typescript',
  'vue',
]
const selectedLanguage = computed({
  get() {
    return __props.node.attrs.language
  },
  set(language) {
    __props.updateAttributes({ language })
  },
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(NodeViewWrapper, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          relative: "",
          my2: ""
        }, [
          _withDirectives(_createElementVNode("select", {
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((selectedLanguage).value = $event)),
            contenteditable: "false",
            absolute: "",
            "top-1": "",
            "right-1": "",
            rounded: "",
            px2: "",
            op0: "",
            "hover:op100": "",
            "focus:op100": "",
            transition: "",
            "outline-none": "",
            border: "~ base"
          }, [
            _createElementVNode("option", { value: null }, "\n          plain\n        ", 8 /* PROPS */, ["value"]),
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(languages, (language, index) => {
              return (_openBlock(), _createElementBlock("option", {
                key: index,
                value: language
              }, _toDisplayString(language), 9 /* TEXT, PROPS */, ["value"]))
            }), 128 /* KEYED_FRAGMENT */))
          ], 512 /* NEED_PATCH */), [
            [_vModelSelect, selectedLanguage.value]
          ]),
          _createElementVNode("pre", { class: "code-block" }, [
            _createElementVNode("code", null, [
              _createVNode(NodeViewContent)
            ])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
