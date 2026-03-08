import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { onMounted, onUnmounted, useTemplateRef } from 'vue'
import { useWidgetPropsManager } from './widget.js'
import { i18n } from '@/i18n.js'
import type { WidgetComponentProps, WidgetComponentEmits, WidgetComponentExpose } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'aichan';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetAichan',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	transparent: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.transparent,
		default: false,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const live2d = useTemplateRef('live2d');
const touched = () => {
	//if (this.live2d) this.live2d.changeExpression('gurugurume');
};
const onMousemove = (ev: MouseEvent) => {
	if (!live2d.value || !live2d.value.contentWindow) return;

	const iframeRect = live2d.value.getBoundingClientRect();
	live2d.value.contentWindow.postMessage({
		type: 'moveCursor',
		body: {
			x: ev.clientX - iframeRect.left,
			y: ev.clientY - iframeRect.top,
		},
	}, '*');
};
onMounted(() => {
	window.addEventListener('mousemove', onMousemove, { passive: true });
});
onUnmounted(() => {
	window.removeEventListener('mousemove', onMousemove);
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  const _component_MkContainer = _resolveComponent("MkContainer")

  return (_openBlock(), _createBlock(_component_MkContainer, {
      naked: _unref(widgetProps).transparent,
      showHeader: false,
      "data-cy-mkw-aichan": "",
      class: "mkw-aichan"
    }, {
      default: _withCtx(() => [
        _createElementVNode("iframe", {
          ref_key: "live2d", ref: live2d,
          class: _normalizeClass(_ctx.$style.root),
          src: "https://misskey-dev.github.io/mascot-web/?scale=1.5&y=1.1&eyeY=100",
          onClick: touched
        }, null, 512 /* NEED_PATCH */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["naked", "showHeader"]))
}
}

})
