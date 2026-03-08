import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("stop", { offset: "0%", "stop-color": "hsl(180, 80%, 70%)" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("stop", { offset: "100%", "stop-color": "hsl(0, 80%, 70%)" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("stop", { offset: "0%", "stop-color": "hsl(180, 80%, 70%)" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("stop", { offset: "100%", "stop-color": "hsl(0, 80%, 70%)" })
import { onMounted, onBeforeUnmount, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'cpu-mem',
  props: {
    connection: { type: null, required: true },
    meta: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const viewBoxX = ref<number>(50);
const viewBoxY = ref<number>(30);
const stats = ref<Misskey.entities.ServerStats[]>([]);
const cpuGradientId = genId();
const cpuMaskId = genId();
const memGradientId = genId();
const memMaskId = genId();
const cpuPolylinePoints = ref<string>('');
const memPolylinePoints = ref<string>('');
const cpuPolygonPoints = ref<string>('');
const memPolygonPoints = ref<string>('');
const cpuHeadX = ref<number>();
const cpuHeadY = ref<number>();
const memHeadX = ref<number>();
const memHeadY = ref<number>();
const cpuP = ref<string>('');
const memP = ref<string>('');
onMounted(() => {
	props.connection.on('stats', onStats);
	props.connection.on('statsLog', onStatsLog);
	props.connection.send('requestLog', {
		id: genId(),
		length: 50,
	});
});
onBeforeUnmount(() => {
	props.connection.off('stats', onStats);
	props.connection.off('statsLog', onStatsLog);
});
function onStats(connStats: Misskey.entities.ServerStats) {
	stats.value.push(connStats);
	if (stats.value.length > 50) stats.value.shift();
	let cpuPolylinePointsStats = stats.value.map((s, i) => [viewBoxX.value - ((stats.value.length - 1) - i), (1 - s.cpu) * viewBoxY.value]);
	let memPolylinePointsStats = stats.value.map((s, i) => [viewBoxX.value - ((stats.value.length - 1) - i), (1 - (s.mem.active / props.meta.mem.total)) * viewBoxY.value]);
	cpuPolylinePoints.value = cpuPolylinePointsStats.map(xy => `${xy[0]},${xy[1]}`).join(' ');
	memPolylinePoints.value = memPolylinePointsStats.map(xy => `${xy[0]},${xy[1]}`).join(' ');
	cpuPolygonPoints.value = `${viewBoxX.value - (stats.value.length - 1)},${viewBoxY.value} ${cpuPolylinePoints.value} ${viewBoxX.value},${viewBoxY.value}`;
	memPolygonPoints.value = `${viewBoxX.value - (stats.value.length - 1)},${viewBoxY.value} ${memPolylinePoints.value} ${viewBoxX.value},${viewBoxY.value}`;
	cpuHeadX.value = cpuPolylinePointsStats.at(-1)![0];
	cpuHeadY.value = cpuPolylinePointsStats.at(-1)![1];
	memHeadX.value = memPolylinePointsStats.at(-1)![0];
	memHeadY.value = memPolylinePointsStats.at(-1)![1];
	cpuP.value = (connStats.cpu * 100).toFixed(0);
	memP.value = (connStats.mem.active / props.meta.mem.total * 100).toFixed(0);
}
function onStatsLog(statsLog: Misskey.entities.ServerStatsLog) {
	for (const revStats of statsLog.toReversed()) {
		onStats(revStats);
	}
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "lcfyofjk" }, [ _createElementVNode("svg", { viewBox: `0 0 ${ viewBoxX.value } ${ viewBoxY.value }` }, [ _createElementVNode("defs", null, [ _createElementVNode("linearGradient", {
            id: _unref(cpuGradientId),
            x1: "0",
            x2: "0",
            y1: "1",
            y2: "0"
          }, [ _hoisted_1, _hoisted_2 ], 8 /* PROPS */, ["id"]), _createElementVNode("mask", {
            id: _unref(cpuMaskId),
            x: "0",
            y: "0",
            width: viewBoxX.value,
            height: viewBoxY.value
          }, [ _createElementVNode("polygon", {
              points: cpuPolygonPoints.value,
              fill: "#fff",
              "fill-opacity": "0.5"
            }, null, 8 /* PROPS */, ["points"]), _createElementVNode("polyline", {
              points: cpuPolylinePoints.value,
              fill: "none",
              stroke: "#fff",
              "stroke-width": "1"
            }, null, 8 /* PROPS */, ["points"]), _createElementVNode("circle", {
              cx: cpuHeadX.value,
              cy: cpuHeadY.value,
              r: "1.5",
              fill: "#fff"
            }, null, 8 /* PROPS */, ["cx", "cy"]) ], 8 /* PROPS */, ["id", "width", "height"]) ]), _createElementVNode("rect", {
          x: "-2",
          y: "-2",
          width: viewBoxX.value + 4,
          height: viewBoxY.value + 4,
          style: _normalizeStyle({ stroke: 'none', fill: `url(#${ _unref(cpuGradientId) })`, mask: `url(#${ _unref(cpuMaskId) })` })
        }, null, 12 /* STYLE, PROPS */, ["width", "height"]), _createElementVNode("text", {
          x: "1",
          y: "5"
        }, [ _createTextVNode("CPU "), _createElementVNode("tspan", null, _toDisplayString(cpuP.value) + "%", 1 /* TEXT */) ]) ], 8 /* PROPS */, ["viewBox"]), _createElementVNode("svg", { viewBox: `0 0 ${ viewBoxX.value } ${ viewBoxY.value }` }, [ _createElementVNode("defs", null, [ _createElementVNode("linearGradient", {
            id: _unref(memGradientId),
            x1: "0",
            x2: "0",
            y1: "1",
            y2: "0"
          }, [ _hoisted_3, _hoisted_4 ], 8 /* PROPS */, ["id"]), _createElementVNode("mask", {
            id: _unref(memMaskId),
            x: "0",
            y: "0",
            width: viewBoxX.value,
            height: viewBoxY.value
          }, [ _createElementVNode("polygon", {
              points: memPolygonPoints.value,
              fill: "#fff",
              "fill-opacity": "0.5"
            }, null, 8 /* PROPS */, ["points"]), _createElementVNode("polyline", {
              points: memPolylinePoints.value,
              fill: "none",
              stroke: "#fff",
              "stroke-width": "1"
            }, null, 8 /* PROPS */, ["points"]), _createElementVNode("circle", {
              cx: memHeadX.value,
              cy: memHeadY.value,
              r: "1.5",
              fill: "#fff"
            }, null, 8 /* PROPS */, ["cx", "cy"]) ], 8 /* PROPS */, ["id", "width", "height"]) ]), _createElementVNode("rect", {
          x: "-2",
          y: "-2",
          width: viewBoxX.value + 4,
          height: viewBoxY.value + 4,
          style: _normalizeStyle({ stroke: 'none', fill: `url(#${ _unref(memGradientId) })`, mask: `url(#${ _unref(memMaskId) })` })
        }, null, 12 /* STYLE, PROPS */, ["width", "height"]), _createElementVNode("text", {
          x: "1",
          y: "5"
        }, [ _createTextVNode("MEM "), _createElementVNode("tspan", null, _toDisplayString(memP.value) + "%", 1 /* TEXT */) ]) ], 8 /* PROPS */, ["viewBox"]) ]))
}
}

})
