import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, normalizeClass as _normalizeClass, withModifiers as _withModifiers } from "vue"

import { onMounted, ref } from 'vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetActivity.chart',
  props: {
    activity: { type: Array, required: true }
  },
  setup(__props: any) {

const props = __props
const viewBoxX = ref(147);
const viewBoxY = ref(60);
const zoom = ref(1);
const pos = ref(0);
const pointsNote = ref<string>();
const pointsReply = ref<string>();
const pointsRenote = ref<string>();
const pointsTotal = ref<string>();
function dragListen(fn: (ev: MouseEvent | TouchEvent) => void) {
	window.addEventListener('mousemove', fn);
	window.addEventListener('mouseleave', dragClear.bind(null, fn));
	window.addEventListener('mouseup', dragClear.bind(null, fn));
}
function dragClear(fn: (ev: MouseEvent | TouchEvent) => void) {
	window.removeEventListener('mousemove', fn);
	window.removeEventListener('mouseleave', dragClear as any);
	window.removeEventListener('mouseup', dragClear as any);
}
function getPositionX(event: MouseEvent | TouchEvent) {
	return 'touches' in event && event.touches.length > 0 ? event.touches[0].clientX : 'clientX' in event ? event.clientX : 0;
}
function getPositionY(event: MouseEvent | TouchEvent) {
	return 'touches' in event && event.touches.length > 0 ? event.touches[0].clientY : 'clientY' in event ? event.clientY : 0;
}
function onMousedown(ev: MouseEvent) {
	const clickX = ev.clientX;
	const clickY = ev.clientY;
	const baseZoom = zoom.value;
	const basePos = pos.value;
	// 動かした時
	dragListen(me => {
		const x = getPositionX(me);
		const y = getPositionY(me);
		let moveLeft = x - clickX;
		let moveTop = y - clickY;
		zoom.value = Math.max(1, baseZoom + (-moveTop / 20));
		pos.value = Math.min(0, basePos + moveLeft);
		if (pos.value < -(((props.activity.length - 1) * zoom.value) - viewBoxX.value)) pos.value = -(((props.activity.length - 1) * zoom.value) - viewBoxX.value);
		render();
	});
}
function render() {
	const peak = Math.max(...props.activity.map(d => d.total));
	if (peak !== 0) {
		const activity = props.activity.slice().reverse();
		pointsNote.value = activity.map((d, i) => `${(i * zoom.value) + pos.value},${(1 - (d.notes / peak)) * viewBoxY.value}`).join(' ');
		pointsReply.value = activity.map((d, i) => `${(i * zoom.value) + pos.value},${(1 - (d.replies / peak)) * viewBoxY.value}`).join(' ');
		pointsRenote.value = activity.map((d, i) => `${(i * zoom.value) + pos.value},${(1 - (d.renotes / peak)) * viewBoxY.value}`).join(' ');
		pointsTotal.value = activity.map((d, i) => `${(i * zoom.value) + pos.value},${(1 - (d.total / peak)) * viewBoxY.value}`).join(' ');
	}
}
onMounted(() => {
	render();
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("svg", {
      viewBox: `0 0 ${ viewBoxX.value } ${ viewBoxY.value }`,
      class: _normalizeClass(_ctx.$style.root),
      onMousedown: _withModifiers(onMousedown, ["prevent"])
    }, [ _createElementVNode("polyline", {
        points: pointsNote.value,
        fill: "none",
        "stroke-width": "1",
        stroke: "#41ddde"
      }, null, 8 /* PROPS */, ["points"]), _createElementVNode("polyline", {
        points: pointsReply.value,
        fill: "none",
        "stroke-width": "1",
        stroke: "#f7796c"
      }, null, 8 /* PROPS */, ["points"]), _createElementVNode("polyline", {
        points: pointsRenote.value,
        fill: "none",
        "stroke-width": "1",
        stroke: "#a1de41"
      }, null, 8 /* PROPS */, ["points"]), _createElementVNode("polyline", {
        points: pointsTotal.value,
        fill: "none",
        "stroke-width": "1",
        stroke: "#555",
        "stroke-dasharray": "2 2"
      }, null, 8 /* PROPS */, ["points"]) ], 40 /* PROPS, NEED_HYDRATION */, ["viewBox"]))
}
}

})
