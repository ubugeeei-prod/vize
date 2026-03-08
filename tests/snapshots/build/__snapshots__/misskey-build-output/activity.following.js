import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, normalizeClass as _normalizeClass, vShow as _vShow } from "vue"

import { onMounted, useTemplateRef, ref } from 'vue'
import { Chart } from 'chart.js'
import * as Misskey from 'misskey-js'
import gradient from 'chartjs-plugin-gradient'
import type { ChartDataset } from 'chart.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { store } from '@/store.js'
import { useChartTooltip } from '@/composables/use-chart-tooltip.js'
import { chartVLine } from '@/utility/chart-vline.js'
import { initChart } from '@/utility/init-chart.js'
import { chartLegend } from '@/utility/chart-legend.js'
import MkChartLegend from '@/components/MkChartLegend.vue'
const chartLimit = 30;

export default /*@__PURE__*/_defineComponent({
  __name: 'activity.following',
  props: {
    user: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
initChart();
const chartEl = useTemplateRef('chartEl');
const legendEl = useTemplateRef('legendEl');
const now = new Date();
let chartInstance: Chart | null = null;
const fetching = ref(true);
const { handler: externalTooltipHandler } = useChartTooltip();
async function renderChart() {
	if (chartEl.value == null) return;
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
	const raw = await misskeyApi('charts/user/following', { userId: props.user.id, limit: chartLimit, span: 'day' });
	const vLineColor = store.s.darkMode ? 'rgba(255, 255, 255, 0.2)' : 'rgba(0, 0, 0, 0.2)';
	const colorFollowLocal = '#008FFB';
	const colorFollowRemote = '#008FFB88';
	const colorFollowedLocal = '#2ecc71';
	const colorFollowedRemote = '#2ecc7188';
	function makeDataset(label: string, data: ChartDataset['data'], extra: Partial<ChartDataset> = {}): ChartDataset {
		return Object.assign({
			label: label,
			data: data,
			parsing: false,
			pointRadius: 0,
			borderWidth: 0,
			borderJoinStyle: 'round',
			borderRadius: 4,
			barPercentage: 0.7,
			categoryPercentage: 0.7,
			fill: true,
		/* @see <https://github.com/misskey-dev/misskey/pull/10365#discussion_r1155511107>
		} satisfies ChartData, extra);
		 */
		}, extra);
	}
	chartInstance = new Chart(chartEl.value, {
		type: 'bar',
		data: {
			datasets: [
				makeDataset('Follow (local)', format(raw.local.followings.inc).slice().reverse(), { backgroundColor: colorFollowLocal, stack: 'follow' }),
				makeDataset('Follow (remote)', format(raw.remote.followings.inc).slice().reverse(), { backgroundColor: colorFollowRemote, stack: 'follow' }),
				makeDataset('Followed (local)', format(raw.local.followers.inc).slice().reverse(), { backgroundColor: colorFollowedLocal, stack: 'followed' }),
				makeDataset('Followed (remote)', format(raw.remote.followers.inc).slice().reverse(), { backgroundColor: colorFollowedRemote, stack: 'followed' }),
			],
		},
		options: {
			aspectRatio: 3,
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
					stacked: true,
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
						display: true,
						maxRotation: 0,
						autoSkipPadding: 8,
					},
				},
				y: {
					position: 'left',
					stacked: true,
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
				...({ // TSを黙らすため
					gradient,
				}),
			},
		},
		plugins: [chartVLine(vLineColor), chartLegend(legendEl.value)],
	});
	fetching.value = false;
}
onMounted(async () => {
	renderChart();
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", null, [ (fetching.value) ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("div", {
        class: _normalizeClass(["_panel", _ctx.$style.root])
      }, [ _createElementVNode("canvas", { ref_key: "chartEl", ref: chartEl }, null, 512 /* NEED_PATCH */), _createVNode(MkChartLegend, {
          ref_key: "legendEl", ref: legendEl,
          style: "margin-top: 8px;"
        }, null, 512 /* NEED_PATCH */) ], 512 /* NEED_PATCH */), [ [_vShow, !fetching.value] ]) ]))
}
}

})
