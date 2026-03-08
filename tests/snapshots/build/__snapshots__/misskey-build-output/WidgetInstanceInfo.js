import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"

import { host } from '@@/js/config.js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import { instance } from '@/instance.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'instanceInfo';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetInstanceInfo',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
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
  const _component_MkA = _resolveComponent("MkA")

  return (_openBlock(), _createElementBlock("div", { class: "_panel" }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.container),
        style: _normalizeStyle({ backgroundImage: _unref(instance).bannerUrl ? `url(${ _unref(instance).bannerUrl })` : undefined })
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.iconContainer)
        }, [ _createElementVNode("img", {
            src: _unref(instance).iconUrl ?? '/favicon.ico',
            alt: "",
            class: _normalizeClass(_ctx.$style.icon)
          }, null, 8 /* PROPS */, ["src"]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.bodyContainer)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.body)
          }, [ _createVNode(_component_MkA, {
              class: _normalizeClass(_ctx.$style.name),
              to: "/about",
              behavior: "window"
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(instance).name), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.host)
            }, _toDisplayString(_unref(host)), 1 /* TEXT */) ]) ]) ], 4 /* STYLE */) ]))
}
}

})
