import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import { watch, ref } from 'vue'
import { genId } from '@/utility/id.js'
import tinycolor from 'tinycolor2'
import { useInterval } from '@@/js/use-interval.js'
const viewBoxX = 50;
const viewBoxY = 50;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMiniChart',
  props: {
    src: { type: Array, required: true }
  },
  setup(__props: any) {

const props = __props
const gradientId = genId();
const polylinePoints = ref('');
const polygonPoints = ref('');
const headX = ref<number | null>(null);
const headY = ref<number | null>(null);
const clock = ref<number | null>(null);
const accent = tinycolor(getComputedStyle(window.document.documentElement).getPropertyValue('--MI_THEME-accent'));
const color = accent.toRgbString();
function draw(): void {
	const stats = props.src.slice().reverse();
	const peak = Math.max.apply(null, stats) || 1;
	const _polylinePoints = stats.map((n, i) => [
		i * (viewBoxX / (stats.length - 1)),
		(1 - (n / peak)) * viewBoxY,
	]);
	polylinePoints.value = _polylinePoints.map(xy => `${xy[0]},${xy[1]}`).join(' ');
	polygonPoints.value = `0,${ viewBoxY } ${ polylinePoints.value } ${ viewBoxX },${ viewBoxY }`;
	headX.value = _polylinePoints.at(-1)![0];
	headY.value = _polylinePoints.at(-1)![1];
}
watch(() => props.src, draw, { immediate: true });
// Vueが何故かWatchを発動させない場合があるので
useInterval(draw, 1000, {
	immediate: false,
	afterMounted: true,
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("svg", {
      viewBox: `0 0 ${ viewBoxX } ${ viewBoxY }`,
      style: "overflow:visible"
    }, [ _createElementVNode("defs", null, [ _createElementVNode("linearGradient", {
          id: _unref(gradientId),
          x1: "0",
          x2: "0",
          y1: "1",
          y2: "0"
        }, [ _createElementVNode("stop", {
            offset: "0%",
            "stop-color": _unref(color),
            "stop-opacity": "0"
          }, null, 8 /* PROPS */, ["stop-color"]), _createElementVNode("stop", {
            offset: "100%",
            "stop-color": _unref(color),
            "stop-opacity": "0.65"
          }, null, 8 /* PROPS */, ["stop-color"]) ], 8 /* PROPS */, ["id"]) ]), _createElementVNode("polygon", {
        points: polygonPoints.value,
        style: _normalizeStyle(`stroke: none; fill: url(#${ _unref(gradientId) });`)
      }, null, 12 /* STYLE, PROPS */, ["points"]), _createElementVNode("polyline", {
        points: polylinePoints.value,
        fill: "none",
        stroke: _unref(color),
        "stroke-width": "2"
      }, null, 8 /* PROPS */, ["points", "stroke"]), _createElementVNode("circle", {
        cx: headX.value ?? undefined,
        cy: headY.value ?? undefined,
        r: "3",
        fill: _unref(color)
      }, null, 8 /* PROPS */, ["cx", "cy", "fill"]) ], 8 /* PROPS */, ["viewBox"]))
}
}

})
