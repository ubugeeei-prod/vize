import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"

import { toRefs } from 'vue'
import type { Ref } from 'vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSwitch.button',
  props: {
    checked: { type: [Boolean, Object], required: true },
    disabled: { type: [Boolean, Object], required: false, default: false }
  },
  emits: ["toggle"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const checked = toRefs(props).checked;
const toggle = () => {
	emit('toggle');
};

return (_ctx: any,_cache: any) => {
  const _directive_tooltip = _resolveDirective("tooltip")

  return _withDirectives((_openBlock(), _createElementBlock("span", {
      class: _normalizeClass({
  		[_ctx.$style.button]: true,
  		[_ctx.$style.buttonChecked]: _unref(checked),
  		[_ctx.$style.buttonDisabled]: props.disabled
  	}),
      "data-cy-switch-toggle": "",
      onClick: _withModifiers(toggle, ["prevent","stop"])
    }, [ _createElementVNode("div", {
        class: _normalizeClass({ [_ctx.$style.knob]: true, [_ctx.$style.knobChecked]: _unref(checked) })
      }, null, 2 /* CLASS */) ], 2 /* CLASS */)), [ [_directive_tooltip, _unref(checked) ? _unref(i18n).ts.itsOn : _unref(i18n).ts.itsOff] ])
}
}

})
