import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"

import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import { ensureSignin } from '@/i.js'
import { userPage } from '@/filters/user.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'profile';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetProfile',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
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
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkAcct = _resolveComponent("MkAcct")

  return (_openBlock(), _createElementBlock("div", { class: "_panel" }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.container),
        style: _normalizeStyle({ backgroundImage: _unref($i).bannerUrl ? `url(${ _unref($i).bannerUrl })` : undefined })
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.avatarContainer)
        }, [ _createVNode(_component_MkAvatar, {
            class: _normalizeClass(_ctx.$style.avatar),
            user: _unref($i)
          }, null, 8 /* PROPS */, ["user"]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.bodyContainer)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.body)
          }, [ _createVNode(_component_MkA, {
              class: _normalizeClass(_ctx.$style.name),
              to: _unref(userPage)(_unref($i))
            }, {
              default: _withCtx(() => [
                _createVNode(_component_MkUserName, { user: _unref($i) }, null, 8 /* PROPS */, ["user"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.username)
            }, [ _createVNode(_component_MkAcct, {
                user: _unref($i),
                detail: ""
              }, null, 8 /* PROPS */, ["user"]) ]) ]) ]) ], 4 /* STYLE */) ]))
}
}

})
