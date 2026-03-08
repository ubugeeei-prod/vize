import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderList as _renderList, toDisplayString as _toDisplayString, unref as _unref } from "vue"

import { deepClone } from '@/utility/clone.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetActivity.calendar',
  props: {
    activity: { type: Array, required: true }
  },
  setup(__props: any) {

const props = __props
const activity = deepClone(props.activity).map(d => ({
	...d,
	total: d.notes + d.replies + d.renotes,
	x: 0,
	date: {
		year: 0,
		month: 0,
		day: 0,
		weekday: 0,
	},
	v: 0,
	color: '',
}));
const peak = Math.max(...activity.map(d => d.total));
const now = new Date();
const year = now.getFullYear();
const month = now.getMonth();
const day = now.getDate();
let x = 20;
activity.slice().forEach((d, i) => {
	d.x = x;
	const date = new Date(year, month, day - i);
	d.date = {
		year: date.getFullYear(),
		month: date.getMonth(),
		day: date.getDate(),
		weekday: date.getDay(),
	};
	d.v = peak === 0 ? 0 : d.total / (peak / 2);
	if (d.v > 1) d.v = 1;
	const ch = d.date.weekday === 0 || d.date.weekday === 6 ? 275 : 170;
	const cs = d.v * 100;
	const cl = 15 + ((1 - d.v) * 80);
	d.color = `hsl(${ch}, ${cs}%, ${cl}%)`;
	if (d.date.weekday === 0) x--;
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("svg", { viewBox: "0 0 21 7" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(activity), (record) => {
        return (_openBlock(), _createElementBlock("rect", { class: "day", width: "1", height: "1", x: record.x, y: record.date.weekday, rx: "1", ry: "1", fill: "transparent" }, [
          _createElementVNode("title", null, _toDisplayString(record.date.year) + "/" + _toDisplayString(record.date.month + 1) + "/" + _toDisplayString(record.date.day), 1 /* TEXT */)
        ], 8 /* PROPS */, ["x", "y"]))
      }), 256 /* UNKEYED_FRAGMENT */)), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(activity), (record) => {
        return (_openBlock(), _createElementBlock("rect", { class: "day", width: record.v, height: record.v, x: record.x + ((1 - record.v) / 2), y: record.date.weekday + ((1 - record.v) / 2), rx: "1", ry: "1", fill: record.color, style: "pointer-events: none;" }, 8 /* PROPS */, ["width", "height", "x", "y", "fill"]))
      }), 256 /* UNKEYED_FRAGMENT */)), _createElementVNode("rect", {
        class: "today",
        width: "1",
        height: "1",
        x: _unref(activity)[0].x,
        y: _unref(activity)[0].date.weekday,
        rx: "1",
        ry: "1",
        fill: "none",
        "stroke-width": "0.1",
        stroke: "#f73520"
      }, null, 8 /* PROPS */, ["x", "y"]) ]))
}
}

})
