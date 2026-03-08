import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderList as _renderList, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("animate", { attributeName: "r", begin: "0s", dur: "0.5s", values: "4; 32", calcMode: "spline", keyTimes: "0; 1", keySplines: "0.165, 0.84, 0.44, 1", repeatCount: "1" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("animate", { attributeName: "stroke-width", begin: "0s", dur: "0.5s", values: "16; 0", calcMode: "spline", keyTimes: "0; 1", keySplines: "0.3, 0.61, 0.355, 1", repeatCount: "1" })
import { onMounted } from 'vue'
import * as os from '@/os.js'
const origin = 64;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkRippleEffect',
  props: {
    x: { type: Number, required: true },
    y: { type: Number, required: true },
    particle: { type: Boolean, required: false, default: true }
  },
  emits: ["end"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const particles: {
	size: number;
	xA: number;
	yA: number;
	xB: number;
	yB: number;
	color: string;
}[] = [];
const colors = ['#FF1493', '#00FFFF', '#FFE202'];
const zIndex = os.claimZIndex('high');
if (props.particle) {
	for (let i = 0; i < 12; i++) {
		const angle = Math.random() * (Math.PI * 2);
		const pos = Math.random() * 16;
		const velocity = 16 + (Math.random() * 48);
		particles.push({
			size: 4 + (Math.random() * 8),
			xA: origin + (Math.sin(angle) * pos),
			yA: origin + (Math.cos(angle) * pos),
			xB: origin + (Math.sin(angle) * (pos + velocity)),
			yB: origin + (Math.cos(angle) * (pos + velocity)),
			color: colors[Math.floor(Math.random() * colors.length)],
		});
	}
}
onMounted(() => {
	window.setTimeout(() => {
		emit('end');
	}, 1100);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root),
      style: _normalizeStyle({ zIndex: _unref(zIndex), top: `${__props.y - 64}px`, left: `${__props.x - 64}px` })
    }, [ _createElementVNode("svg", {
        width: "128",
        height: "128",
        viewBox: "0 0 128 128",
        xmlns: "http://www.w3.org/2000/svg"
      }, [ _createElementVNode("circle", {
          fill: "none",
          cx: "64",
          cy: "64",
          style: "stroke: var(--MI_THEME-accent);"
        }, [ _hoisted_1, _hoisted_2 ]), _createElementVNode("g", {
          fill: "none",
          "fill-rule": "evenodd"
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(particles, (particle, i) => {
            return (_openBlock(), _createElementBlock("circle", {
              key: i,
              fill: particle.color,
              style: "stroke: var(--MI_THEME-accent);"
            }, [
              _createElementVNode("animate", {
                attributeName: "r",
                begin: "0s",
                dur: "0.8s",
                values: `${particle.size}; 0`,
                calcMode: "spline",
                keyTimes: "0; 1",
                keySplines: "0.165, 0.84, 0.44, 1",
                repeatCount: "1"
              }, null, 8 /* PROPS */, ["values"]),
              _createElementVNode("animate", {
                attributeName: "cx",
                begin: "0s",
                dur: "0.8s",
                values: `${particle.xA}; ${particle.xB}`,
                calcMode: "spline",
                keyTimes: "0; 1",
                keySplines: "0.3, 0.61, 0.355, 1",
                repeatCount: "1"
              }, null, 8 /* PROPS */, ["values"]),
              _createElementVNode("animate", {
                attributeName: "cy",
                begin: "0s",
                dur: "0.8s",
                values: `${particle.yA}; ${particle.yB}`,
                calcMode: "spline",
                keyTimes: "0; 1",
                keySplines: "0.3, 0.61, 0.355, 1",
                repeatCount: "1"
              }, null, 8 /* PROPS */, ["values"])
            ], 8 /* PROPS */, ["fill"]))
          }), 128 /* KEYED_FRAGMENT */)) ]) ]) ], 4 /* STYLE */))
}
}

})
