import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


const _hoisted_1 = { class: "sr-only" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div")
import { VueUiSparkline } from 'vue-data-ui/vue-ui-sparkline'
import { useCssVariables } from '~/composables/useColors'
import type { WeeklyDataPoint } from '~/types/chart'
import { applyDataCorrection } from '~/utils/chart-data-correction'
import { OKLCH_NEUTRAL_FALLBACK, lightenOklch } from '~/utils/colors'
import { applyBlocklistCorrection } from '~/utils/download-anomalies'
import type { RepoRef } from '#shared/utils/git-providers'
import type { VueUiSparklineConfig, VueUiSparklineDatasetItem } from 'vue-data-ui'

export default /*@__PURE__*/_defineComponent({
  __name: 'WeeklyDownloadStats',
  props: {
    packageName: { type: String, required: true },
    createdIso: { type: String, required: true },
    repoRef: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const router = useRouter()
const route = useRoute()
const { settings } = useSettings()
const chartModal = useModal('chart-modal')
const hasChartModalTransitioned = shallowRef(false)
const modalTitle = computed(() => {
  const facet = route.query.facet as string | undefined
  if (facet === 'likes') return $t('package.trends.items.likes')
  if (facet === 'contributors') return $t('package.trends.items.contributors')
  return $t('package.trends.items.downloads')
})
const modalSubtitle = computed(() => {
  const facet = route.query.facet as string | undefined
  if (facet === 'likes' || facet === 'contributors') return undefined
  return $t('package.downloads.subtitle')
})
const isChartModalOpen = shallowRef<boolean>(false)
function handleModalClose() {
  isChartModalOpen.value = false
  hasChartModalTransitioned.value = false
  router.replace({
    query: {
      ...route.query,
      modal: undefined,
      granularity: undefined,
      end: undefined,
      start: undefined,
      facet: undefined,
    },
  })
}
function handleModalTransitioned() {
  hasChartModalTransitioned.value = true
}
const { fetchPackageDownloadEvolution } = useCharts()
const { accentColors, selectedAccentColor } = useAccentColor()
const colorMode = useColorMode()
const resolvedMode = shallowRef<'light' | 'dark'>('light')
const rootEl = shallowRef<HTMLElement | null>(null)
onMounted(() => {
  rootEl.value = document.documentElement
  resolvedMode.value = colorMode.value === 'dark' ? 'dark' : 'light'
})
watch(
  () => colorMode.value,
  value => {
    resolvedMode.value = value === 'dark' ? 'dark' : 'light'
  },
  { flush: 'sync' },
)
const { colors } = useCssVariables(
  [
    '--bg',
    '--fg',
    '--bg-subtle',
    '--bg-elevated',
    '--border-hover',
    '--fg-subtle',
    '--border',
    '--border-subtle',
  ],
  {
    element: rootEl,
    watchHtmlAttributes: true,
    watchResize: false, // set to true only if a var changes color on resize
  },
)
function toggleSparklineAnimation() {
  settings.value.sidebar.animateSparkline = !settings.value.sidebar.animateSparkline
}
const hasSparklineAnimation = computed(() => settings.value.sidebar.animateSparkline)
const isDarkMode = computed(() => resolvedMode.value === 'dark')
const accentColorValueById = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  for (const item of accentColors.value) {
    map[item.id] = item.value
  }
  return map
})
const accent = computed(() => {
  const id = selectedAccentColor.value
  return id
    ? (accentColorValueById.value[id] ?? colors.value.fgSubtle ?? OKLCH_NEUTRAL_FALLBACK)
    : (colors.value.fgSubtle ?? OKLCH_NEUTRAL_FALLBACK)
})
const pulseColor = computed(() => {
  if (!selectedAccentColor.value) {
    return colors.value.fgSubtle
  }
  return isDarkMode.value ? accent.value : lightenOklch(accent.value, 0.5)
})
const weeklyDownloads = shallowRef<WeeklyDataPoint[]>([])
const isLoadingWeeklyDownloads = shallowRef(true)
const hasWeeklyDownloads = computed(() => weeklyDownloads.value.length > 0)
async function openChartModal() {
  if (!hasWeeklyDownloads.value) return
  isChartModalOpen.value = true
  hasChartModalTransitioned.value = false
  await router.replace({
    query: {
      ...route.query,
      modal: 'chart',
    },
  })
  // ensure the component renders before opening the dialog
  await nextTick()
  await nextTick()
  chartModal.open()
}
async function loadWeeklyDownloads() {
  if (!import.meta.client) return
  isLoadingWeeklyDownloads.value = true
  try {
    const result = await fetchPackageDownloadEvolution(
      () => props.packageName,
      () => props.createdIso,
      () => ({ granularity: 'week' as const, weeks: 52 }),
    )
    weeklyDownloads.value = (result as WeeklyDataPoint[]) ?? []
  } catch {
    weeklyDownloads.value = []
  } finally {
    isLoadingWeeklyDownloads.value = false
  }
}
onMounted(async () => {
  await loadWeeklyDownloads()
  if (route.query.modal === 'chart') {
    isChartModalOpen.value = true
  }
  if (isChartModalOpen.value && hasWeeklyDownloads.value) {
    openChartModal()
  }
})
watch(
  () => props.packageName,
  () => loadWeeklyDownloads(),
)
const correctedDownloads = computed<WeeklyDataPoint[]>(() => {
  let data = weeklyDownloads.value as WeeklyDataPoint[]
  if (!data.length) return data
  if (settings.value.chartFilter.anomaliesFixed) {
    data = applyBlocklistCorrection({
      data,
      packageName: props.packageName,
      granularity: 'weekly',
    }) as WeeklyDataPoint[]
  }
  data = applyDataCorrection(data, settings.value.chartFilter) as WeeklyDataPoint[]
  return data
})
const dataset = computed<VueUiSparklineDatasetItem[]>(() =>
  correctedDownloads.value.map(d => ({
    value: d?.value ?? 0,
    period: $t('package.trends.date_range', {
      start: d.weekStart ?? '-',
      end: d.weekEnd ?? '-',
    }),
  })),
)
const lastDatapoint = computed(() => dataset.value.at(-1)?.period ?? '')
const config = computed<VueUiSparklineConfig>(() => {
  return {
    theme: 'dark',
    /**
     * The built-in skeleton loader kicks in when the component is mounted but the data is not yet ready.
     * The configuration of the skeleton is customized for a seemless transition with the final state
     */
    skeletonConfig: {
      style: {
        backgroundColor: 'transparent',
        dataLabel: {
          show: true,
          color: 'transparent',
        },
        area: {
          color: colors.value.borderHover,
          useGradient: false,
          opacity: 10,
        },
        line: {
          color: colors.value.borderHover,
        },
      },
    },
    // Same idea: initialize the line at zero, so it nicely transitions to the final dataset
    skeletonDataset: Array.from({ length: 52 }, () => 0),
    style: {
      backgroundColor: 'transparent',
      animation: { show: false },
      area: {
        color: colors.value.borderHover,
        useGradient: false,
        opacity: 10,
      },
      dataLabel: {
        offsetX: -10,
        fontSize: 28,
        bold: false,
        color: colors.value.fg,
      },
      line: {
        color: colors.value.borderHover,
        pulse: {
          show: hasSparklineAnimation.value, // the pulse will not show if prefers-reduced-motion (enforced by vue-data-ui)
          loop: true, // runs only once if false
          radius: 1.5,
          color: pulseColor.value!,
          easing: 'ease-in-out',
          trail: {
            show: true,
            length: 30,
            opacity: 0.75,
          },
        },
      },
      plot: {
        radius: 6,
        stroke: isDarkMode.value ? 'oklch(0.985 0 0)' : 'oklch(0.145 0 0)',
      },
      title: {
        text: String(lastDatapoint.value),
        fontSize: 12,
        color: colors.value.fgSubtle,
        bold: false,
      },
      verticalIndicator: {
        strokeDasharray: 0,
        color: isDarkMode.value ? 'oklch(0.985 0 0)' : colors.value.fgSubtle,
      },
    },
  }
})

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_SkeletonInline = _resolveComponent("SkeletonInline")
  const _component_ClientOnly = _resolveComponent("ClientOnly")
  const _component_CollapsibleSection = _resolveComponent("CollapsibleSection")
  const _component_PackageTrendsChart = _resolveComponent("PackageTrendsChart")
  const _component_PackageChartModal = _resolveComponent("PackageChartModal")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("div", { class: "space-y-8" }, [ _createVNode(_component_CollapsibleSection, {
          id: "downloads",
          title: _ctx.$t('package.downloads.title'),
          subtitle: _ctx.$t('package.downloads.subtitle')
        }, {
          actions: _withCtx(() => [
            (hasWeeklyDownloads.value)
              ? (_openBlock(), _createBlock(_component_ButtonBase, {
                key: 0,
                type: "button",
                onClick: openChartModal,
                class: "text-fg-subtle hover:text-fg transition-colors duration-200 inline-flex items-center justify-center min-w-6 min-h-6 -m-1 p-1 focus-visible:outline-accent/70 rounded",
                title: _ctx.$t('package.trends.title'),
                classicon: "i-lucide:chart-line"
              }, {
                default: _withCtx(() => [
                  _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('package.trends.title')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["title"]))
              : (isLoadingWeeklyDownloads.value)
                ? (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  class: "min-w-6 min-h-6 -m-1 p-1"
                }))
              : _createCommentVNode("v-if", true)
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", { class: "w-full overflow-hidden h-[76px] motion-safe:h-[calc(92px+0.75rem)]" }, [
              (isLoadingWeeklyDownloads.value || hasWeeklyDownloads.value)
                ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                  _createVNode(_component_ClientOnly, null, {
                    fallback: _withCtx(() => [
                      _createElementVNode("div", { class: "max-w-xs" }, [
                        _createElementVNode("div", { class: "h-6 flex items-center ps-3" }, [
                          _createVNode(_component_SkeletonInline, { class: "h-3 w-36" })
                        ]),
                        _createTextVNode("\n                " + "\n                "),
                        _createElementVNode("div", { class: "aspect-[500/80] flex items-center" }, [
                          _createElementVNode("div", { class: "w-[42%] flex items-center ps-0.5" }, [
                            _createVNode(_component_SkeletonInline, { class: "h-7 w-24" })
                          ]),
                          _createTextVNode("\n                  " + "\n                  "),
                          _createElementVNode("div", { class: "flex-1 flex items-end pe-3" }, [
                            _createVNode(_component_SkeletonInline, { class: "h-px w-full" })
                          ])
                        ]),
                        _createTextVNode("\n                " + "\n                "),
                        _createElementVNode("div", { class: "w-full hidden motion-safe:flex flex-1 items-end justify-end" }, [
                          _createVNode(_component_SkeletonInline, { class: "h-[20px] w-30" })
                        ])
                      ])
                    ]),
                    default: _withCtx(() => [
                      _createVNode(VueUiSparkline, {
                        class: "w-full max-w-xs",
                        dataset: dataset.value,
                        config: config.value
                      }, {
                        skeleton: _withCtx(() => [
                          _hoisted_2
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["dataset", "config"])
                    ]),
                    _: 1 /* STABLE */
                  }),
                  (hasWeeklyDownloads.value)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: "hidden motion-safe:flex justify-end p-1"
                    }, [
                      _createVNode(_component_ButtonBase, {
                        size: "small",
                        onClick: toggleSparklineAnimation
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(hasSparklineAnimation.value
                    ? _ctx.$t('package.trends.pause_animation')
                    : _ctx.$t('package.trends.play_animation')), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]))
                    : _createCommentVNode("v-if", true)
                ], 64 /* STABLE_FRAGMENT */))
                : (_openBlock(), _createElementBlock("p", {
                  key: 1,
                  class: "py-2 text-sm font-mono text-fg-subtle"
                }, _toDisplayString(_ctx.$t('package.trends.no_data')), 1 /* TEXT */))
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["title", "subtitle"]) ]), (isChartModalOpen.value && hasWeeklyDownloads.value) ? (_openBlock(), _createBlock(_component_PackageChartModal, {
          key: 0,
          "modal-title": modalTitle.value,
          "modal-subtitle": modalSubtitle.value,
          onClose: handleModalClose,
          onTransitioned: handleModalTransitioned
        }, {
          default: _withCtx(() => [
            _createVNode(_Transition, {
              name: "opacity",
              mode: "out-in"
            }, {
              default: _withCtx(() => [
                (hasChartModalTransitioned.value)
                  ? (_openBlock(), _createBlock(_component_PackageTrendsChart, {
                    key: 0,
                    weeklyDownloads: weeklyDownloads.value,
                    inModal: true,
                    packageName: props.packageName,
                    repoRef: props.repoRef,
                    createdIso: __props.createdIso,
                    permalink: "",
                    "show-facet-selector": ""
                  }, null, 8 /* PROPS */, ["weeklyDownloads", "inModal", "packageName", "repoRef", "createdIso"]))
                  : _createCommentVNode("v-if", true)
              ]),
              _: 1 /* STABLE */
            }),
            _createTextVNode("\n\n    "),
            _createTextVNode("\n    "),
            _createTextVNode("\n    "),
            (!hasChartModalTransitioned.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "w-full aspect-[390/634.5] sm:aspect-[718/647]"
              }))
              : _createCommentVNode("v-if", true)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modal-title", "modal-subtitle"])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
