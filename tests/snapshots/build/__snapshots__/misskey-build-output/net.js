import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString } from "vue"

import { onMounted, onBeforeUnmount, ref } from 'vue'
import * as Misskey from 'misskey-js'
import bytes from '@/filters/bytes.js'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'net',
  props: {
    connection: { type: null, required: true },
    meta: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const viewBoxX = ref<number>(50);
const viewBoxY = ref<number>(30);
const stats = ref<Misskey.entities.ServerStats[]>([]);
const inPolylinePoints = ref<string>('');
const outPolylinePoints = ref<string>('');
const inPolygonPoints = ref<string>('');
const outPolygonPoints = ref<string>('');
const inHeadX = ref<number>();
const inHeadY = ref<number>();
const outHeadX = ref<number>();
const outHeadY = ref<number>();
const inRecent = ref<number>(0);
const outRecent = ref<number>(0);
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
	const inPeak = Math.max(1024 * 64, Math.max(...stats.value.map(s => s.net.rx)));
	const outPeak = Math.max(1024 * 64, Math.max(...stats.value.map(s => s.net.tx)));
	let inPolylinePointsStats = stats.value.map((s, i) => [viewBoxX.value - ((stats.value.length - 1) - i), (1 - (s.net.rx / inPeak)) * viewBoxY.value]);
	let outPolylinePointsStats = stats.value.map((s, i) => [viewBoxX.value - ((stats.value.length - 1) - i), (1 - (s.net.tx / outPeak)) * viewBoxY.value]);
	inPolylinePoints.value = inPolylinePointsStats.map(xy => `${xy[0]},${xy[1]}`).join(' ');
	outPolylinePoints.value = outPolylinePointsStats.map(xy => `${xy[0]},${xy[1]}`).join(' ');
	inPolygonPoints.value = `${viewBoxX.value - (stats.value.length - 1)},${viewBoxY.value} ${inPolylinePoints.value} ${viewBoxX.value},${viewBoxY.value}`;
	outPolygonPoints.value = `${viewBoxX.value - (stats.value.length - 1)},${viewBoxY.value} ${outPolylinePoints.value} ${viewBoxX.value},${viewBoxY.value}`;
	inHeadX.value = inPolylinePointsStats.at(-1)![0];
	inHeadY.value = inPolylinePointsStats.at(-1)![1];
	outHeadX.value = outPolylinePointsStats.at(-1)![0];
	outHeadY.value = outPolylinePointsStats.at(-1)![1];
	inRecent.value = connStats.net.rx;
	outRecent.value = connStats.net.tx;
}
function onStatsLog(statsLog: Misskey.entities.ServerStatsLog) {
	for (const revStats of statsLog.toReversed()) {
		onStats(revStats);
	}
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "oxxrhrto" }, [ _createElementVNode("svg", { viewBox: `0 0 ${ viewBoxX.value } ${ viewBoxY.value }` }, [ _createElementVNode("polygon", {
          points: inPolygonPoints.value,
          fill: "#94a029",
          "fill-opacity": "0.5"
        }, null, 8 /* PROPS */, ["points"]), _createElementVNode("polyline", {
          points: inPolylinePoints.value,
          fill: "none",
          stroke: "#94a029",
          "stroke-width": "1"
        }, null, 8 /* PROPS */, ["points"]), _createElementVNode("circle", {
          cx: inHeadX.value,
          cy: inHeadY.value,
          r: "1.5",
          fill: "#94a029"
        }, null, 8 /* PROPS */, ["cx", "cy"]), _createElementVNode("text", {
          x: "1",
          y: "5"
        }, [ _createTextVNode("NET rx "), _createElementVNode("tspan", null, _toDisplayString(bytes(inRecent.value)), 1 /* TEXT */) ]) ], 8 /* PROPS */, ["viewBox"]), _createElementVNode("svg", { viewBox: `0 0 ${ viewBoxX.value } ${ viewBoxY.value }` }, [ _createElementVNode("polygon", {
          points: outPolygonPoints.value,
          fill: "#ff9156",
          "fill-opacity": "0.5"
        }, null, 8 /* PROPS */, ["points"]), _createElementVNode("polyline", {
          points: outPolylinePoints.value,
          fill: "none",
          stroke: "#ff9156",
          "stroke-width": "1"
        }, null, 8 /* PROPS */, ["points"]), _createElementVNode("circle", {
          cx: outHeadX.value,
          cy: outHeadY.value,
          r: "1.5",
          fill: "#ff9156"
        }, null, 8 /* PROPS */, ["cx", "cy"]), _createElementVNode("text", {
          x: "1",
          y: "5"
        }, [ _createTextVNode("NET tx "), _createElementVNode("tspan", null, _toDisplayString(bytes(outRecent.value)), 1 /* TEXT */) ]) ], 8 /* PROPS */, ["viewBox"]) ]))
}
}

})
