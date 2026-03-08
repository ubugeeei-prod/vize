import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, renderSlot as _renderSlot, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue"

import { onMounted, onUnmounted, ref, useTemplateRef } from 'vue'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSparkle',
  setup(__props) {

const particles = ref<{
	id: string,
	x: number,
	y: number,
	size: number,
	dur: number,
	color: string
}[]>([]);
const el = useTemplateRef('el');
const width = ref(0);
const height = ref(0);
const colors = ['#FF1493', '#00FFFF', '#FFE202', '#FFE202', '#FFE202'];
let stop = false;
let ro: ResizeObserver | undefined;
onMounted(() => {
	ro = new ResizeObserver((entries, observer) => {
		if (el.value == null) return;
		width.value = el.value.offsetWidth + 64;
		height.value = el.value.offsetHeight + 64;
	});
	if (el.value) ro.observe(el.value);
	const add = () => {
		if (stop) return;
		const x = (Math.random() * (width.value - 64));
		const y = (Math.random() * (height.value - 64));
		const sizeFactor = Math.random();
		const particle = {
			id: genId(),
			x,
			y,
			size: 0.2 + ((sizeFactor / 10) * 3),
			dur: 1000 + (sizeFactor * 1000),
			color: colors[Math.floor(Math.random() * colors.length)],
		};
		particles.value.push(particle);
		window.setTimeout(() => {
			particles.value = particles.value.filter(x => x.id !== particle.id);
		}, particle.dur - 100);

		window.setTimeout(() => {
			add();
		}, 500 + (Math.random() * 500));
	};
	add();
});
onUnmounted(() => {
	if (ro) ro.disconnect();
	stop = true;
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("span", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("span", {
        ref_key: "el", ref: el,
        style: "display: inline-block;"
      }, [ _renderSlot(_ctx.$slots, "default") ], 512 /* NEED_PATCH */), _createTextVNode("\n\t" + "\n\t" + "\n\t"), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(particles.value, (particle) => {
        return (_openBlock(), _createElementBlock("svg", {
          key: particle.id,
          width: width.value,
          height: height.value,
          viewBox: `0 0 ${width.value} ${height.value}`,
          xmlns: "http://www.w3.org/2000/svg",
          style: "position: absolute; top: -32px; left: -32px; pointer-events: none;"
        }, [
          _createElementVNode("path", {
            style: _normalizeStyle({
  				'--translateX': particle.x + 'px',
  				'--translateY': particle.y + 'px',
  				'--duration': particle.dur + 'ms',
  				'--size': particle.size,
  			}),
            class: _normalizeClass(_ctx.$style.particle),
            fill: particle.color,
            d: "M29.427,2.011C29.721,0.83 30.782,0 32,0C33.218,0 34.279,0.83 34.573,2.011L39.455,21.646C39.629,22.347 39.991,22.987 40.502,23.498C41.013,24.009 41.653,24.371 42.354,24.545L61.989,29.427C63.17,29.721 64,30.782 64,32C64,33.218 63.17,34.279 61.989,34.573L42.354,39.455C41.653,39.629 41.013,39.991 40.502,40.502C39.991,41.013 39.629,41.653 39.455,42.354L34.573,61.989C34.279,63.17 33.218,64 32,64C30.782,64 29.721,63.17 29.427,61.989L24.545,42.354C24.371,41.653 24.009,41.013 23.498,40.502C22.987,39.991 22.347,39.629 21.646,39.455L2.011,34.573C0.83,34.279 0,33.218 0,32C0,30.782 0.83,29.721 2.011,29.427L21.646,24.545C22.347,24.371 22.987,24.009 23.498,23.498C24.009,22.987 24.371,22.347 24.545,21.646L29.427,2.011Z"
          }, null, 12 /* STYLE, PROPS */, ["fill"])
        ], 8 /* PROPS */, ["width", "height", "viewBox"]))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
