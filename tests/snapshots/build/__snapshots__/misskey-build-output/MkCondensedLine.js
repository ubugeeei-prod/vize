import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass } from "vue"

import { ref, watch } from 'vue'

interface Props {
	readonly minScale?: number;
}

const contentSymbol = Symbol();
const observer = new ResizeObserver((entries) => {
	const results: {
		container: HTMLSpanElement;
		transform: string;
	}[] = [];
	for (const entry of entries) {
		const content = ((entry.target as any)[contentSymbol] ? entry.target : entry.target.firstElementChild) as HTMLSpanElement;
		const props: Required<Props> = (content as any)[contentSymbol];
		const container = content.parentElement as HTMLSpanElement;
		const contentWidth = content.getBoundingClientRect().width;
		const containerWidth = container.getBoundingClientRect().width;
		results.push({ container, transform: `scaleX(${Math.max(props.minScale, Math.min(1, containerWidth / contentWidth))})` });
	}
	for (const result of results) {
		result.container.style.transform = result.transform;
	}
});

export default /*@__PURE__*/_defineComponent({
  __name: 'MkCondensedLine',
  setup(__props: any) {

const props = __props
const content = ref<HTMLSpanElement>();
watch(content, (value, oldValue) => {
	if (oldValue != null) {
		delete (oldValue as any)[contentSymbol];
		observer.unobserve(oldValue);
		if (oldValue.parentElement) {
			observer.unobserve(oldValue.parentElement);
		}
	}
	if (value != null) {
		(value as any)[contentSymbol] = props;
		observer.observe(value);
		if (value.parentElement) {
			observer.observe(value.parentElement);
		}
	}
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("span", {
      class: _normalizeClass(_ctx.$style.container)
    }, [ _createElementVNode("span", {
        ref_key: "content", ref: content,
        class: _normalizeClass(_ctx.$style.content),
        style: { maxWidth: `${100 / _ctx.minScale}%` }
      }, [ _renderSlot(_ctx.$slots, "default") ], 512 /* NEED_PATCH */) ]))
}
}

})
