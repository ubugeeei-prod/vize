import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withCtx as _withCtx, unref as _unref } from "vue"

import { ref } from 'vue'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { definePage } from '@/page.js'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkLink from '@/components/MkLink.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkButton from '@/components/MkButton.vue'
import { useMkSelect } from '@/composables/use-mkselect.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'debug',
  setup(__props) {

const {
	model: resultType,
	def: resultTypeDef,
} = useMkSelect({
	items: [
		{ label: 'empty', value: 'empty' },
		{ label: 'notFound', value: 'notFound' },
		{ label: 'error', value: 'error' },
	],
	initialValue: 'empty',
});
const {
	model: iconType,
	def: iconTypeDef,
} = useMkSelect({
	items: [
		{ label: 'info', value: 'info' },
		{ label: 'question', value: 'question' },
		{ label: 'success', value: 'success' },
		{ label: 'warn', value: 'warn' },
		{ label: 'error', value: 'error' },
		{ label: 'waiting', value: 'waiting' },
	],
	initialValue: 'info',
});
definePage(() => ({
	title: 'DEBUG ROOM',
	icon: 'ti ti-help-circle',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkSystemIcon = _resolveComponent("MkSystemIcon")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 600px;"
        }, [
          _createElementVNode("div", { class: "_gaps_m" }, [
            (_unref(resultType) === 'empty')
              ? (_openBlock(), _createBlock(_component_MkResult, {
                key: 0,
                type: "empty"
              }))
              : _createCommentVNode("v-if", true),
            (_unref(resultType) === 'notFound')
              ? (_openBlock(), _createBlock(_component_MkResult, {
                key: 0,
                type: "notFound"
              }))
              : _createCommentVNode("v-if", true),
            (_unref(resultType) === 'error')
              ? (_openBlock(), _createBlock(_component_MkResult, {
                key: 0,
                type: "error"
              }))
              : _createCommentVNode("v-if", true),
            _createVNode(MkSelect, {
              items: _unref(resultTypeDef),
              modelValue: _unref(resultType),
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((resultType).value = $event))
            }, null, 8 /* PROPS */, ["items", "modelValue"]),
            (_unref(iconType) === 'info')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 0,
                type: "info",
                style: "width: 150px;"
              }))
              : _createCommentVNode("v-if", true),
            (_unref(iconType) === 'question')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 0,
                type: "question",
                style: "width: 150px;"
              }))
              : _createCommentVNode("v-if", true),
            (_unref(iconType) === 'success')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 0,
                type: "success",
                style: "width: 150px;"
              }))
              : _createCommentVNode("v-if", true),
            (_unref(iconType) === 'warn')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 0,
                type: "warn",
                style: "width: 150px;"
              }))
              : _createCommentVNode("v-if", true),
            (_unref(iconType) === 'error')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 0,
                type: "error",
                style: "width: 150px;"
              }))
              : _createCommentVNode("v-if", true),
            (_unref(iconType) === 'waiting')
              ? (_openBlock(), _createBlock(_component_MkSystemIcon, {
                key: 0,
                type: "waiting",
                style: "width: 150px;"
              }))
              : _createCommentVNode("v-if", true),
            _createVNode(MkSelect, {
              items: _unref(iconTypeDef),
              modelValue: _unref(iconType),
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((iconType).value = $event))
            }, null, 8 /* PROPS */, ["items", "modelValue"]),
            _createElementVNode("div", { class: "_buttons" }, [
              _createVNode(MkButton, {
                onClick: _cache[2] || (_cache[2] = ($event: any) => (os.alert({ type: 'error', title: 'Error', text: 'error' })))
              }, {
                default: _withCtx(() => [
                  _createTextVNode("Error")
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, {
                onClick: _cache[3] || (_cache[3] = ($event: any) => (os.alert({ type: 'warning', title: 'Warning', text: 'warning' })))
              }, {
                default: _withCtx(() => [
                  _createTextVNode("Warning")
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, {
                onClick: _cache[4] || (_cache[4] = ($event: any) => (os.alert({ type: 'info', title: 'Info', text: 'info' })))
              }, {
                default: _withCtx(() => [
                  _createTextVNode("Info")
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, {
                onClick: _cache[5] || (_cache[5] = ($event: any) => (os.alert({ type: 'success', title: 'Success', text: 'success' })))
              }, {
                default: _withCtx(() => [
                  _createTextVNode("Success")
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, {
                onClick: _cache[6] || (_cache[6] = ($event: any) => (os.alert({ type: 'question', title: 'Question', text: 'question' })))
              }, {
                default: _withCtx(() => [
                  _createTextVNode("Question")
                ]),
                _: 1 /* STABLE */
              })
            ])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
