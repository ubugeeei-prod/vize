import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkContainer from '@/components/MkContainer.vue'
import { i18n } from '@/i18n.js'
import MkChatHistories from '@/components/MkChatHistories.vue'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'chat';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetChat',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure, save } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      class: "mkw-chat"
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets.chat), 1 /* TEXT */)
      ]),
      func: _withCtx(({ buttonStyleClass }) => [
        _createElementVNode("button", {
          class: _normalizeClass(["_button", buttonStyleClass]),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(configure)()))
        }, [
          _hoisted_2
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", null, [
          _createVNode(MkChatHistories)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader"]))
}
}

})
