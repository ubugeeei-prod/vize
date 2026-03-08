import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"

import { onMounted, useTemplateRef } from 'vue'
import { Chart } from 'chart.js'
import { useChartTooltip } from '@/composables/use-chart-tooltip.js'
import { initChart } from '@/utility/init-chart.js'

export type InstanceForPie = {
	name: string,
	color: string | null,
	value: number,
	onClick?: () => void
};

export default /*@__PURE__*/_defineComponent({
  __name: 'overview.pie',
  props: {
    data: { type: Array, required: true }
  },
  setup(__props: any) {

const props = __props
initChart();
const chartEl = useTemplateRef('chartEl');
const { handler: externalTooltipHandler } = useChartTooltip({
	position: 'middle',
});
let chartInstance: Chart | null = null;
onMounted(() => {
	if (chartEl.value == null) return;
	chartInstance = new Chart(chartEl.value, {
		type: 'doughnut',
		data: {
			labels: props.data.map(x => x.name),
			datasets: [{
				backgroundColor: props.data.map(x => x.color ?? '#000'),
				borderColor: getComputedStyle(window.document.documentElement).getPropertyValue('--MI_THEME-panel'),
				borderWidth: 2,
				hoverOffset: 0,
				data: props.data.map(x => x.value),
			}],
		},
		options: {
			layout: {
				padding: {
					left: 16,
					right: 16,
					top: 16,
					bottom: 16,
				},
			},
			onClick: (ev) => {
				if (ev.native == null) return;
				const hit = chartInstance!.getElementsAtEventForMode(ev.native, 'nearest', { intersect: true }, false)[0];
				if (hit && props.data[hit.index].onClick != null) {
					props.data[hit.index].onClick!();
				}
			},
			plugins: {
				legend: {
					display: false,
				},
				tooltip: {
					enabled: false,
					mode: 'index',
					animation: {
						duration: 0,
					},
					external: externalTooltipHandler,
				},
			},
		},
	});
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("canvas", { ref_key: "chartEl", ref: chartEl }, null, 512 /* NEED_PATCH */))
}
}

})
