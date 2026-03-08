import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, normalizeClass as _normalizeClass, vShow as _vShow } from "vue"

import { onMounted, useTemplateRef, ref, nextTick } from 'vue'
import { Chart } from 'chart.js'
import gradient from 'chartjs-plugin-gradient'
import tinycolor from 'tinycolor2'
import { misskeyApi } from '@/utility/misskey-api.js'
import { store } from '@/store.js'
import { useChartTooltip } from '@/composables/use-chart-tooltip.js'
import { chartVLine } from '@/utility/chart-vline.js'
import { initChart } from '@/utility/init-chart.js'
const chartLimit = 30;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkVisitorDashboard.ActiveUsersChart',
  setup(__props) {

initChart();
const chartEl = useTemplateRef('chartEl');
const now = new Date();
let chartInstance: Chart | null = null;
const fetching = ref(true);
const { handler: externalTooltipHandler } = useChartTooltip();
async function renderChart() {
	if (chartInstance) {
		chartInstance.destroy();
	}
	const getDate = (ago: number) => {
		const y = now.getFullYear();
		const m = now.getMonth();
		const d = now.getDate();

		return new Date(y, m, d - ago);
	};
	const format = (arr: number[]) => {
		return arr.map((v, i) => ({
			x: getDate(i).getTime(),
			y: v,
		}));
	};
	const raw = await misskeyApi('charts/active-users', { limit: chartLimit, span: 'day' });
	fetching.value = false;
	await nextTick();
	const vLineColor = store.s.darkMode ? 'rgba(255, 255, 255, 0.2)' : 'rgba(0, 0, 0, 0.2)';
	const computedStyle = getComputedStyle(window.document.documentElement);
	const accent = tinycolor(computedStyle.getPropertyValue('--MI_THEME-accent')).toHexString();
	const colorRead = accent;
	const colorWrite = '#2ecc71';
	const max = Math.max(...raw.read);
	if (chartEl.value == null) return;
	chartInstance = new Chart(chartEl.value, {
		type: 'bar',
		data: {
			datasets: [{
				parsing: false,
				label: 'Read',
				data: format(raw.read).slice().reverse(),
				pointRadius: 0,
				borderWidth: 0,
				borderJoinStyle: 'round',
				borderRadius: 4,
				backgroundColor: colorRead,
				barPercentage: 0.5,
				categoryPercentage: 1,
				fill: true,
			}],
		},
		options: {
			aspectRatio: 2.5,
			layout: {
				padding: {
					left: 0,
					right: 8,
					top: 0,
					bottom: 0,
				},
			},
			scales: {
				x: {
					type: 'time',
					offset: true,
					time: {
						unit: 'day',
						displayFormats: {
							day: 'M/d',
							month: 'Y/M',
						},
					},
					grid: {
						display: false,
					},
					ticks: {
						stepSize: 1,
						display: true,
						maxRotation: 0,
						autoSkipPadding: 8,
					},
				},
				y: {
					position: 'left',
					suggestedMax: 10,
					grid: {
						display: true,
					},
					ticks: {
						display: true,
						//mirror: true,
					},
				},
			},
			interaction: {
				intersect: false,
				mode: 'index',
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
		plugins: [chartVLine(vLineColor)],
	});
}
onMounted(async () => {
	renderChart();
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", null, [ (fetching.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.root)
      }, [ _createElementVNode("canvas", { ref_key: "chartEl", ref: chartEl }, null, 512 /* NEED_PATCH */) ], 512 /* NEED_PATCH */), [ [_vShow, !fetching.value] ]) ]))
}
}

})
