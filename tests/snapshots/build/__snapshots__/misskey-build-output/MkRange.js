import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, renderList as _renderList, renderSlot as _renderSlot, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "shine right" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { class: "shine left" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { class: "thumbInner" })
import { computed, defineAsyncComponent, onMounted, onUnmounted, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue'
import { isTouchUsing } from '@/utility/touch.js'
import * as os from '@/os.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkRange',
  props: {
    modelValue: { type: Number, required: true },
    disabled: { type: Boolean, required: false },
    min: { type: Number, required: true },
    max: { type: Number, required: true },
    step: { type: Number, required: false, default: 1 },
    textConverter: { type: Function, required: false, default: (v: number) => (Math.round(v * 1000) / 1000).toString() },
    showTicks: { type: Boolean, required: false },
    easing: { type: Boolean, required: false, default: false },
    continuousUpdate: { type: Boolean, required: false }
  },
  emits: ["update:modelValue", "dragEnded", "thumbDoubleClicked"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const containerEl = useTemplateRef('containerEl');
const thumbEl = useTemplateRef('thumbEl');
const maxRatio = computed(() => Math.abs(props.max) / (props.max + Math.abs(Math.min(0, props.min))));
const minRatio = computed(() => Math.abs(Math.min(0, props.min)) / (props.max + Math.abs(Math.min(0, props.min))));
const rightTrackWidth = computed(() => {
	return Math.max(0, (steppedRawValue.value - minRatio.value) * 100) + '%';
});
const leftTrackWidth = computed(() => {
	return Math.max(0, (minRatio.value - steppedRawValue.value) * 100) + '%';
});
const rightTrackPosition = computed(() => {
	return (Math.abs(Math.min(0, props.min)) / (props.max + Math.abs(Math.min(0, props.min)))) * 100 + '%';
});
const leftTrackPosition = computed(() => {
	return (Math.min(minRatio.value, steppedRawValue.value) * 100) + '%';
});
const calcRawValue = (value: number) => {
	return (value - props.min) / (props.max - props.min);
};
const rawValue = ref(calcRawValue(props.modelValue));
const steppedRawValue = computed(() => {
	if (props.step) {
		const step = props.step / (props.max - props.min);
		return (step * Math.round(rawValue.value / step));
	} else {
		return rawValue.value;
	}
});
const finalValue = computed(() => {
	if (Number.isInteger(props.step)) {
		return Math.round((steppedRawValue.value * (props.max - props.min)) + props.min);
	} else {
		return (steppedRawValue.value * (props.max - props.min)) + props.min;
	}
});
const getThumbWidth = () => {
	if (thumbEl.value == null) return 0;
	return thumbEl.value!.offsetWidth;
};
const thumbPosition = ref(0);
const calcThumbPosition = () => {
	if (containerEl.value == null) {
		thumbPosition.value = 0;
	} else {
		thumbPosition.value = (containerEl.value.offsetWidth - getThumbWidth()) * steppedRawValue.value;
	}
};
watch([steppedRawValue, containerEl], calcThumbPosition);
watch(() => props.modelValue, (newVal) => {
	const newRawValue = calcRawValue(newVal);
	if (rawValue.value === newRawValue) return;
	rawValue.value = newRawValue;
});
let ro: ResizeObserver | undefined;
onMounted(() => {
	ro = new ResizeObserver((entries, observer) => {
		calcThumbPosition();
	});
	if (containerEl.value) ro.observe(containerEl.value);
});
onUnmounted(() => {
	if (ro) ro.disconnect();
});
const steps = computed(() => {
	if (props.step) {
		return (props.max - props.min) / props.step;
	} else {
		return 0;
	}
});
const tooltipForDragShowing = ref(false);
const tooltipForHoverShowing = ref(false);
onBeforeUnmount(() => {
	// 何らかの問題で表示されっぱなしでもコンポーネントを離れたら消えるように
	tooltipForDragShowing.value = false;
	tooltipForHoverShowing.value = false;
});
function onMouseenter() {
	if (isTouchUsing) return;
	tooltipForHoverShowing.value = true;
	const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkTooltip.vue')), {
		showing: computed(() => tooltipForHoverShowing.value && !tooltipForDragShowing.value),
		text: computed(() => {
			return props.textConverter(finalValue.value);
		}),
		anchorElement: thumbEl.value ?? undefined,
	}, {
		closed: () => dispose(),
	});
	thumbEl.value!.addEventListener('mouseleave', () => {
		tooltipForHoverShowing.value = false;
	}, { once: true, passive: true });
}
let lastClickTime: number | null = null;
function onMousedown(ev: MouseEvent | TouchEvent) {
	if (props.disabled) return; // Prevent interaction if disabled
	ev.preventDefault();
	tooltipForDragShowing.value = true;
	const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkTooltip.vue')), {
		showing: tooltipForDragShowing,
		text: computed(() => {
			return props.textConverter(finalValue.value);
		}),
		anchorElement: thumbEl.value ?? undefined,
	}, {
		closed: () => dispose(),
	});
	const style = window.document.createElement('style');
	style.appendChild(window.document.createTextNode('* { cursor: grabbing !important; } body * { pointer-events: none !important; }'));
	window.document.head.appendChild(style);
	const thumbWidth = getThumbWidth();
	const onDrag = (ev: MouseEvent | TouchEvent) => {
		ev.preventDefault();
		let beforeValue = finalValue.value;
		const containerRect = containerEl.value!.getBoundingClientRect();
		const pointerX = 'touches' in ev && ev.touches.length > 0 ? ev.touches[0].clientX : 'clientX' in ev ? ev.clientX : 0;
		const pointerPositionOnContainer = pointerX - (containerRect.left + (thumbWidth / 2));
		rawValue.value = Math.min(1, Math.max(0, pointerPositionOnContainer / (containerEl.value!.offsetWidth - thumbWidth)));

		if (props.continuousUpdate && beforeValue !== finalValue.value) {
			emit('update:modelValue', finalValue.value);
		}
	};
	let beforeValue = finalValue.value;
	const onMouseup = () => {
		window.document.head.removeChild(style);
		tooltipForDragShowing.value = false;
		window.removeEventListener('mousemove', onDrag);
		window.removeEventListener('touchmove', onDrag);
		window.removeEventListener('mouseup', onMouseup);
		window.removeEventListener('touchend', onMouseup);

		// 値が変わってたら通知
		if (beforeValue !== finalValue.value) {
			emit('update:modelValue', finalValue.value);
			emit('dragEnded', finalValue.value);
		}
	};
	window.addEventListener('mousemove', onDrag);
	window.addEventListener('touchmove', onDrag);
	window.addEventListener('mouseup', onMouseup, { once: true });
	window.addEventListener('touchend', onMouseup, { once: true });
	if (lastClickTime == null) {
		lastClickTime = Date.now();
		return;
	} else {
		const now = Date.now();
		if (now - lastClickTime < 300) { // 300ms以内のクリックはダブルクリックとみなす
			lastClickTime = null;
			emit('thumbDoubleClicked');
			return;
		} else {
			lastClickTime = now;
		}
	}
}

return (_ctx: any,_cache: any) => {
  const _directive_adaptive_border = _resolveDirective("adaptive-border")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["timctyfi", { disabled: __props.disabled, easing: __props.easing }])
    }, [ _createElementVNode("div", { class: "label" }, [ _renderSlot(_ctx.$slots, "label") ]), _createElementVNode("div", {
        class: _normalizeClass(["body", { 'disabled': __props.disabled }])
      }, [ _renderSlot(_ctx.$slots, "prefix"), _createElementVNode("div", {
          ref_key: "containerEl", ref: containerEl,
          class: "container"
        }, [ _createElementVNode("div", { class: "track" }, [ _createElementVNode("div", {
              class: "highlight right",
              style: _normalizeStyle({ width: rightTrackWidth.value, left: rightTrackPosition.value })
            }, [ _hoisted_1 ], 4 /* STYLE */), _createElementVNode("div", {
              class: "highlight left",
              style: _normalizeStyle({ width: leftTrackWidth.value, left: leftTrackPosition.value })
            }, [ _hoisted_2 ], 4 /* STYLE */) ]), (steps.value && __props.showTicks) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "ticks"
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList((steps.value + 1), (i) => {
                return (_openBlock(), _createElementBlock("div", { class: "tick", style: _normalizeStyle({ left: (((i - 1) / steps.value) * 100) + '%' }) }, 4 /* STYLE */))
              }), 256 /* UNKEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
            ref_key: "thumbEl", ref: thumbEl,
            class: "thumb",
            style: _normalizeStyle({ left: thumbPosition.value + 'px' }),
            onMouseenterPassive: onMouseenter,
            onMousedown: onMousedown,
            onTouchstart: onMousedown
          }, [ _hoisted_3 ], 36 /* STYLE, NEED_HYDRATION */) ], 512 /* NEED_PATCH */), _renderSlot(_ctx.$slots, "suffix") ], 2 /* CLASS */), _createElementVNode("div", { class: "caption" }, [ _renderSlot(_ctx.$slots, "caption") ]) ], 2 /* CLASS */))
}
}

})
