import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue"

import { computed, onMounted, onBeforeUnmount, ref } from 'vue'
import tinycolor from 'tinycolor2'
import { globalEvents } from '@/events.js'
import { defaultIdlingRenderScheduler } from '@/utility/idle-render.js'
const graduationsPadding = 0.5;
const textsPadding = 0.6;
const handsPadding = 1;
const handsTailLength = 0.7;
const hHandLengthRatio = 0.75;
const mHandLengthRatio = 1;
const sHandLengthRatio = 1;
const numbersOpacityFactor = 0.35;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAnalogClock',
  props: {
    thickness: { type: Number, required: false, default: 0.1 },
    offset: { type: Number, required: false, default: 0 - new Date().getTimezoneOffset() },
    twentyfour: { type: Boolean, required: false, default: false },
    graduations: { type: String, required: false, default: 'dots' },
    fadeGraduations: { type: Boolean, required: false, default: true },
    sAnimation: { type: String, required: false, default: 'elastic' },
    now: { type: Function, required: false, default: () => new Date() }
  },
  setup(__props: any) {

const props = __props
// https://stackoverflow.com/questions/1878907/how-can-i-find-the-difference-between-two-angles
const angleDiff = (a: number, b: number) => {
	const x = Math.abs(a - b);
	return Math.abs((x + Math.PI) % (Math.PI * 2) - Math.PI);
};
const graduationsMajor = computed(() => {
	const angles: number[] = [];
	const times = props.twentyfour ? 24 : 12;
	for (let i = 0; i < times; i++) {
		const angle = Math.PI * i / (times / 2);
		angles.push(angle);
	}
	return angles;
});
const texts = computed(() => {
	const angles: number[] = [];
	const times = props.twentyfour ? 24 : 12;
	for (let i = 0; i < times; i++) {
		const angle = Math.PI * i / (times / 2);
		angles.push(angle);
	}
	return angles;
});
let enabled = true;
const majorGraduationColor = ref<string>();
//let minorGraduationColor = $ref<string>();
const sHandColor = ref<string>();
const mHandColor = ref<string>();
const hHandColor = ref<string>();
const nowColor = ref<string>();
const h = ref<number>(0);
const m = ref<number>(0);
const s = ref<number>(0);
const hAngle = ref<number>(0);
const mAngle = ref<number>(0);
const sAngle = ref<number>(0);
const disableSAnimate = ref(false);
let sOneRound = false;
const sLine = ref<SVGPathElement>();
function tick() {
	const now = props.now();
	now.setMinutes(now.getMinutes() + now.getTimezoneOffset() + props.offset);
	const previousS = s.value;
	const previousM = m.value;
	const previousH = h.value;
	s.value = now.getSeconds();
	m.value = now.getMinutes();
	h.value = now.getHours();
	if (previousS === s.value && previousM === m.value && previousH === h.value) {
		return;
	}
	hAngle.value = Math.PI * (h.value % (props.twentyfour ? 24 : 12) + (m.value + s.value / 60) / 60) / (props.twentyfour ? 12 : 6);
	mAngle.value = Math.PI * (m.value + s.value / 60) / 30;
	if (sOneRound && sLine.value) { // 秒針が一周した際のアニメーションをよしなに処理する(これが無いと秒が59->0になったときに期待したアニメーションにならない)
		sAngle.value = Math.PI * 60 / 30;
		defaultIdlingRenderScheduler.delete(tick);
		sLine.value.addEventListener('transitionend', () => {
			disableSAnimate.value = true;
			requestAnimationFrame(() => {
				sAngle.value = 0;
				requestAnimationFrame(() => {
					disableSAnimate.value = false;
					if (enabled) {
						defaultIdlingRenderScheduler.add(tick);
					}
				});
			});
		}, { once: true });
	} else {
		sAngle.value = Math.PI * s.value / 30;
	}
	sOneRound = s.value === 59;
}
tick();
function calcColors() {
	const computedStyle = getComputedStyle(window.document.documentElement);
	const dark = tinycolor(computedStyle.getPropertyValue('--MI_THEME-bg')).isDark();
	const accent = tinycolor(computedStyle.getPropertyValue('--MI_THEME-accent')).toHexString();
	majorGraduationColor.value = dark ? 'rgba(255, 255, 255, 0.3)' : 'rgba(0, 0, 0, 0.3)';
	//minorGraduationColor = dark ? 'rgba(255, 255, 255, 0.2)' : 'rgba(0, 0, 0, 0.2)';
	sHandColor.value = dark ? 'rgba(255, 255, 255, 0.5)' : 'rgba(0, 0, 0, 0.3)';
	mHandColor.value = tinycolor(computedStyle.getPropertyValue('--MI_THEME-fg')).toHexString();
	hHandColor.value = accent;
	nowColor.value = accent;
}
calcColors();
onMounted(() => {
	defaultIdlingRenderScheduler.add(tick);
	globalEvents.on('themeChanged', calcColors);
});
onBeforeUnmount(() => {
	enabled = false;
	defaultIdlingRenderScheduler.delete(tick);
	globalEvents.off('themeChanged', calcColors);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("svg", {
      class: _normalizeClass(_ctx.$style.root),
      viewBox: "0 0 10 10",
      preserveAspectRatio: "none"
    }, [ (props.graduations === 'dots') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(graduationsMajor.value, (angle, i) => {
            return (_openBlock(), _createElementBlock("circle", { cx: 5 + (Math.sin(angle) * (5 - graduationsPadding)), cy: 5 - (Math.cos(angle) * (5 - graduationsPadding)), r: 0.125, fill: (props.twentyfour ? h.value : h.value % 12) === i ? nowColor.value : majorGraduationColor.value, opacity: !props.fadeGraduations || (props.twentyfour ? h.value : h.value % 12) === i ? 1 : Math.max(0, 1 - (angleDiff(hAngle.value, angle) / Math.PI) - numbersOpacityFactor) }, 8 /* PROPS */, ["cx", "cy", "r", "fill", "opacity"]))
          }), 256 /* UNKEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : (props.graduations === 'numbers') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(texts.value, (angle, i) => {
              return (_openBlock(), _createElementBlock("text", { x: 5 + (Math.sin(angle) * (5 - textsPadding)), y: 5 - (Math.cos(angle) * (5 - textsPadding)), "text-anchor": "middle", "dominant-baseline": "middle", "font-size": (props.twentyfour ? h.value : h.value % 12) === i ? 1 : 0.7, "font-weight": (props.twentyfour ? h.value : h.value % 12) === i ? 'bold' : 'normal', fill: (props.twentyfour ? h.value : h.value % 12) === i ? nowColor.value : 'currentColor', opacity: !props.fadeGraduations || (props.twentyfour ? h.value : h.value % 12) === i ? 1 : Math.max(0, 1 - (angleDiff(hAngle.value, angle) / Math.PI) - numbersOpacityFactor) }, _toDisplayString(i === 0 ? (props.twentyfour ? '24' : '12') : i), 9 /* TEXT, PROPS */, ["x", "y", "font-size", "font-weight", "fill", "opacity"]))
            }), 256 /* UNKEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n\t" + "\n\n\t"), _createElementVNode("line", {
        ref_key: "sLine", ref: sLine,
        class: _normalizeClass([_ctx.$style.s, { [_ctx.$style.animate]: !disableSAnimate.value && __props.sAnimation !== 'none', [_ctx.$style.elastic]: __props.sAnimation === 'elastic', [_ctx.$style.easeOut]: __props.sAnimation === 'easeOut' }]),
        x1: 5 - (0 * (sHandLengthRatio * handsTailLength)),
        y1: 5 + (1 * (sHandLengthRatio * handsTailLength)),
        x2: 5 + (0 * ((sHandLengthRatio * 5) - handsPadding)),
        y2: 5 - (1 * ((sHandLengthRatio * 5) - handsPadding)),
        stroke: sHandColor.value,
        "stroke-width": __props.thickness / 2,
        style: _normalizeStyle(`transform: rotateZ(${sAngle.value}rad)`),
        "stroke-linecap": "round"
      }, null, 14 /* CLASS, STYLE, PROPS */, ["x1", "y1", "x2", "y2", "stroke", "stroke-width"]), _createElementVNode("line", {
        x1: 5 - (Math.sin(mAngle.value) * (mHandLengthRatio * handsTailLength)),
        y1: 5 + (Math.cos(mAngle.value) * (mHandLengthRatio * handsTailLength)),
        x2: 5 + (Math.sin(mAngle.value) * ((mHandLengthRatio * 5) - handsPadding)),
        y2: 5 - (Math.cos(mAngle.value) * ((mHandLengthRatio * 5) - handsPadding)),
        stroke: mHandColor.value,
        "stroke-width": __props.thickness,
        "stroke-linecap": "round"
      }, null, 8 /* PROPS */, ["x1", "y1", "x2", "y2", "stroke", "stroke-width"]), _createElementVNode("line", {
        x1: 5 - (Math.sin(hAngle.value) * (hHandLengthRatio * handsTailLength)),
        y1: 5 + (Math.cos(hAngle.value) * (hHandLengthRatio * handsTailLength)),
        x2: 5 + (Math.sin(hAngle.value) * ((hHandLengthRatio * 5) - handsPadding)),
        y2: 5 - (Math.cos(hAngle.value) * ((hHandLengthRatio * 5) - handsPadding)),
        stroke: hHandColor.value,
        "stroke-width": __props.thickness,
        "stroke-linecap": "round"
      }, null, 8 /* PROPS */, ["x1", "y1", "x2", "y2", "stroke", "stroke-width"]) ]))
}
}

})
