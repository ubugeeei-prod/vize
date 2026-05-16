import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "text-3xs font-mono text-fg-subtle tracking-wide uppercase" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { tabindex: "0", class: "i-lucide:info w-3.5 h-3.5 text-fg-subtle cursor-help shrink-0 rounded-sm", role: "img", "aria-label": "versions info" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { tabindex: "0", class: "i-lucide:info w-3.5 h-3.5 text-fg-subtle cursor-help shrink-0 rounded-sm", role: "img", "aria-label": "versions info" })
const _hoisted_4 = { id: "version-distribution-title", class: "sr-only" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:undo-2 w-5 h-5", "aria-hidden": "true" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle font-mono pointer-events-none" }, "CSV")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle font-mono pointer-events-none" }, "PNG")
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle font-mono pointer-events-none" }, "SVG")
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-6 h-6 text-fg-subtle", style: "pointer-events: none", "aria-hidden": "true" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:undo-2 w-6 h-6 text-fg-subtle", style: "pointer-events: none", "aria-hidden": "true" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:redo-2 w-6 h-6 text-fg-subtle", style: "pointer-events: none", "aria-hidden": "true" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:trash w-6 h-6 text-fg-subtle", style: "pointer-events: none", "aria-hidden": "true" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("div")
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:database w-8 h-8" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:octagon-alert w-8 h-8 text-red-400" })
const _hoisted_16 = { class: "text-xs" }
const _hoisted_17 = { class: "text-xs text-fg-subtle font-mono bg-bg/70 backdrop-blur px-3 py-2 rounded-md border border-border" }
import { VueUiXy } from 'vue-data-ui/vue-ui-xy'
import { type VueUiXyDatasetItem, type VueUiXyConfig } from 'vue-data-ui'
import { useElementSize } from '@vueuse/core'
import { useCssVariables } from '~/composables/useColors'
import { OKLCH_NEUTRAL_FALLBACK, transparentizeOklch, lightenHex } from '~/utils/colors'
import { drawSvgPrintLegend, drawNpmxLogoAndTaglineWatermark } from '~/composables/useChartWatermark'
import TooltipApp from '~/components/Tooltip/App.vue'
import { copyAltTextForVersionsBarChart } from '~/utils/charts'
const mobileBreakpointWidth = 640

export default /*@__PURE__*/_defineComponent({
  __name: 'VersionDistribution',
  props: {
    packageName: { type: String, required: true },
    inModal: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const { accentColors, selectedAccentColor } = useAccentColor()
const { copy, copied } = useClipboard()
const colorMode = useColorMode()
const resolvedMode = shallowRef<'light' | 'dark'>('light')
const rootEl = shallowRef<HTMLElement | null>(null)
onMounted(async () => {
  rootEl.value = document.documentElement
  resolvedMode.value = colorMode.value === 'dark' ? 'dark' : 'light'
})
const { colors } = useCssVariables(
  ['--bg', '--fg', '--bg-subtle', '--bg-elevated', '--fg-subtle', '--border', '--border-subtle'],
  {
    element: rootEl,
    watchHtmlAttributes: true,
    watchResize: false,
  },
)
watch(
  () => colorMode.value,
  value => {
    resolvedMode.value = value === 'dark' ? 'dark' : 'light'
  },
  { flush: 'sync' },
)
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
const watermarkColors = computed(() => ({
  fg: colors.value.fg ?? OKLCH_NEUTRAL_FALLBACK,
  bg: colors.value.bg ?? OKLCH_NEUTRAL_FALLBACK,
  fgSubtle: colors.value.fgSubtle ?? OKLCH_NEUTRAL_FALLBACK,
}))
const { width } = useElementSize(rootEl)
const isMobile = computed(() => width.value > 0 && width.value < mobileBreakpointWidth)
const {
  groupingMode,
  showRecentOnly,
  showLowUsageVersions,
  pending,
  error,
  chartDataset,
  hasData,
} = useVersionDistribution(() => props.packageName)
const compactNumberFormatter = useCompactNumberFormatter()
// Show loading indicator immediately to maintain stable layout
const showLoadingIndicator = computed(() => pending.value)
const loadFile = (link: string, filename: string) => {
  const a = document.createElement('a')
  a.href = link
  a.download = filename
  a.click()
  a.remove()
}
const sanitise = (value: string) =>
  value
    .replace(/^@/, '')
    .replace(/[\\/:"*?<>|]/g, '-')
    .replace(/\//g, '-')
const { locale } = useI18n()
function formatDate(date: Date) {
  return date.toLocaleString(locale.value, {
    month: 'short',
    day: 'numeric',
    timeZone: 'UTC',
  })
}
const endDate = computed(() => {
  const t = new Date()
  return new Date(Date.UTC(t.getUTCFullYear(), t.getUTCMonth(), t.getUTCDate() - 1))
})
const startDate = computed(() => {
  const start = new Date(endDate.value)
  start.setUTCDate(start.getUTCDate() - 6)
  return start
})
const { t } = useI18n()
const dateRangeLabel = computed(() => {
  const from = formatDate(startDate.value)
  const to = formatDate(endDate.value)
  const endYear = endDate.value.getUTCFullYear()
  const startYear = startDate.value.getUTCFullYear()

  if (startYear !== endYear) {
    return t('package.versions.distribution_range_date_multiple_years', {
      from,
      to,
      startYear,
      endYear,
    })
  }

  return t('package.versions.distribution_range_date_same_year', { from, to, endYear })
})
function buildExportFilename(extension: string): string {
  const range = dateRangeLabel.value.replaceAll(' ', '_').replaceAll(',', '')
  const label = props.packageName
  return `${sanitise(label ?? '')}_${range}.${extension}`
}
// VueUiXy expects one series with multiple values for bar charts
const xyDataset = computed<VueUiXyDatasetItem[]>(() => {
  if (!chartDataset.value.length) return []

  return [
    {
      name: props.packageName,
      series: chartDataset.value.map(item => item.downloads),
      type: 'bar' as const,
      color: accent.value,
    },
  ]
})
const xAxisLabels = computed(() => {
  return chartDataset.value.map(item => item.name)
})
const hasMinimap = computed<boolean>(() => {
  const series = xyDataset.value[0]?.series ?? []
  return series.length > 6
})
const chartConfig = computed<VueUiXyConfig>(() => {
  return {
    theme: isDarkMode.value ? 'dark' : '',
    chart: {
      title: {
        text: dateRangeLabel.value,
        fontSize: isMobile.value ? 24 : 16,
        bold: false,
      },
      height: isMobile.value ? 750 : hasMinimap.value ? 500 : 611,
      backgroundColor: colors.value.bg,
      padding: {
        top: 24,
        right: 24,
        bottom: 60,
      },
      userOptions: {
        buttons: {
          pdf: false,
          labels: false,
          fullscreen: false,
          table: false,
          tooltip: false,
          altCopy: true,
        },
        buttonTitles: {
          csv: $t('package.trends.download_file', { fileType: 'CSV' }),
          img: $t('package.trends.download_file', { fileType: 'PNG' }),
          svg: $t('package.trends.download_file', { fileType: 'SVG' }),
          annotator: $t('package.trends.toggle_annotator'),
          altCopy: $t('package.trends.copy_alt.button_label'), // Do not make this text dependant on the `copied` variable, since this would re-render the component, which is undesirable if the minimap was used to select a time frame.
        },
        callbacks: {
          img: args => {
            const imageUri = args?.imageUri
            if (!imageUri) return
            loadFile(imageUri, buildExportFilename('png'))
          },
          csv: csvStr => {
            if (!csvStr) return
            const PLACEHOLDER_CHAR = '\0'
            const multilineDateTemplate = $t('package.trends.date_range_multiline', {
              start: PLACEHOLDER_CHAR,
              end: PLACEHOLDER_CHAR,
            })
              .replaceAll(PLACEHOLDER_CHAR, '')
              .trim()
            const blob = new Blob([
              csvStr
                .replace('data:text/csv;charset=utf-8,', '')
                .replaceAll(`\n${multilineDateTemplate}`, ` ${multilineDateTemplate}`),
            ])
            const url = URL.createObjectURL(blob)
            loadFile(url, buildExportFilename('csv'))
            URL.revokeObjectURL(url)
          },
          svg: args => {
            const blob = args?.blob
            if (!blob) return
            const url = URL.createObjectURL(blob)
            loadFile(url, buildExportFilename('svg'))
            URL.revokeObjectURL(url)
          },
          altCopy: ({ dataset: dst, config: cfg }) =>
            copyAltTextForVersionsBarChart({
              dataset: dst,
              config: {
                ...cfg,
                datapointLabels: xAxisLabels.value,
                dateRangeLabel: dateRangeLabel.value,
                semverGroupingMode: groupingMode.value,
                copy,
                $t,
                numberFormatter: compactNumberFormatter.value.format,
              },
            }),
        },
      },
      grid: {
        stroke: colors.value.border,
        showHorizontalLines: true,
        labels: {
          fontSize: isMobile.value ? 24 : 16,
          color: pending.value ? colors.value.border : colors.value.fgSubtle,
          axis: {
            yLabel: $t('package.versions.y_axis_label'),
            yLabelOffsetX: 12,
            fontSize: isMobile.value ? 32 : 24,
          },
          yAxis: {
            formatter: ({ value }: { value: number }) => {
              return compactNumberFormatter.value.format(Number.isFinite(value) ? value : 0)
            },
            useNiceScale: true,
          },
          xAxisLabels: {
            show: xAxisLabels.value.length <= 25,
            values: xAxisLabels.value,
            fontSize: 16,
            color: colors.value.fgSubtle,
          },
        },
      },
      highlighter: { useLine: false },
      legend: { show: false, position: 'top' },
      bar: {
        periodGap: 16,
        innerGap: 8,
        borderRadius: 4,
      },
      tooltip: {
        teleportTo: props.inModal ? '#chart-modal' : undefined,
        borderColor: 'transparent',
        backdropFilter: false,
        backgroundColor: 'transparent',
        customFormat: ({ datapoint, absoluteIndex, bars }) => {
          if (!datapoint || pending.value) return ''

          // Use absoluteIndex to get the correct version from chartDataset
          const index = Number(absoluteIndex)
          if (!Number.isInteger(index) || index < 0 || index >= chartDataset.value.length) return ''
          const chartItem = chartDataset.value[index]

          if (!chartItem) return ''

          const barSeries = Array.isArray(bars?.[0]?.series) ? bars[0].series : []
          const barValue = index < barSeries.length ? barSeries[index] : undefined
          const raw = Number(barValue ?? chartItem.downloads ?? 0)
          const v = compactNumberFormatter.value.format(Number.isFinite(raw) ? raw : 0)

          return `<div class="font-mono text-xs p-3 border border-border rounded-md bg-[var(--bg)]/10 backdrop-blur-md">
            <div class="flex flex-col gap-2">
              <div class="flex items-center justify-between gap-4">
                <span class="text-3xs tracking-wide text-[var(--fg)]/70">
                  ${chartItem.name}
                </span>
                <span class="text-base text-[var(--fg)] font-mono tabular-nums">
                  ${v}
                </span>
              </div>
            </div>
          </div>`
        },
      },
      zoom: {
        maxWidth: isMobile.value ? 350 : 500,
        highlightColor: colors.value.bgElevated,
        useResetSlot: true,
        minimap: {
          show: true,
          lineColor: '#FAFAFA',
          selectedColor: accent.value,
          selectedColorOpacity: 0.06,
          frameColor: colors.value.border,
          handleWidth: isMobile.value ? 40 : 20, // does not affect the size of the touch area
          handleBorderColor: colors.value.fgSubtle,
          handleType: 'grab', // 'empty' | 'chevron' | 'arrow' | 'grab'
        },
        preview: {
          fill: transparentizeOklch(accent.value, isDarkMode.value ? 0.95 : 0.92),
          stroke: transparentizeOklch(accent.value, 0.5),
          strokeWidth: 1,
          strokeDasharray: 3,
        },
      },
    },
  }
})

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_ButtonGroup = _resolveComponent("ButtonGroup")
  const _component_ClientOnly = _resolveComponent("ClientOnly")

  return (_openBlock(), _createElementBlock("div", {
      class: "w-full flex flex-col",
      id: "version-distribution",
      "aria-busy": _unref(pending) ? 'true' : 'false'
    }, [ _createElementVNode("div", { class: "flex flex-col sm:flex-row gap-3 sm:gap-4 sm:items-end mb-6" }, [ _createElementVNode("div", { class: "flex flex-col gap-1" }, [ _createElementVNode("label", _hoisted_1, _toDisplayString(_ctx.$t('package.versions.distribution_title')), 1 /* TEXT */), _createVNode(_component_ButtonGroup, null, {
            default: _withCtx(() => [
              _createVNode(_component_ButtonBase, {
                onClick: _cache[0] || (_cache[0] = ($event: any) => (groupingMode.value = 'major')),
                variant: _unref(groupingMode) === 'major' ? 'primary' : 'secondary'
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('package.versions.grouping_major')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["variant"]),
              _createVNode(_component_ButtonBase, {
                onClick: _cache[1] || (_cache[1] = ($event: any) => (groupingMode.value = 'minor')),
                variant: _unref(groupingMode) === 'minor' ? 'primary' : 'secondary'
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('package.versions.grouping_minor')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["variant"])
            ]),
            _: 1 /* STABLE */
          }) ]), _createElementVNode("div", { class: "flex flex-col gap-1" }, [ _createElementVNode("label", { class: "text-3xs font-mono text-fg-subtle tracking-wide uppercase flex items-center gap-2" }, [ _createElementVNode("span", null, _toDisplayString(_ctx.$t('package.versions.grouping_versions_title')), 1 /* TEXT */), _createVNode(TooltipApp, {
              text: _ctx.$t('package.versions.recent_versions_only_tooltip'),
              interactive: "",
              position: "top",
              to: __props.inModal ? '#chart-modal' : undefined,
              offset: 8
            }, {
              default: _withCtx(() => [
                _hoisted_2
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["text", "to", "offset"]) ]), _createVNode(_component_ButtonGroup, null, {
            default: _withCtx(() => [
              _createVNode(_component_ButtonBase, {
                onClick: _cache[2] || (_cache[2] = ($event: any) => (showRecentOnly.value = false)),
                variant: _unref(showRecentOnly) ? 'secondary' : 'primary'
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('package.versions.grouping_versions_all')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["variant"]),
              _createVNode(_component_ButtonBase, {
                onClick: _cache[3] || (_cache[3] = ($event: any) => (showRecentOnly.value = true)),
                variant: _unref(showRecentOnly) ? 'primary' : 'secondary'
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('package.versions.grouping_versions_only_recent')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["variant"])
            ]),
            _: 1 /* STABLE */
          }) ]), _createElementVNode("div", { class: "flex flex-col gap-1" }, [ _createElementVNode("label", { class: "text-3xs font-mono text-fg-subtle tracking-wide uppercase flex items-center gap-2" }, [ _createElementVNode("span", null, _toDisplayString(_ctx.$t('package.versions.grouping_usage_title')), 1 /* TEXT */), _createVNode(TooltipApp, {
              text: _ctx.$t('package.versions.show_low_usage_tooltip'),
              interactive: "",
              position: "top",
              to: __props.inModal ? '#chart-modal' : undefined,
              offset: 8
            }, {
              default: _withCtx(() => [
                _hoisted_3
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["text", "to", "offset"]) ]), _createVNode(_component_ButtonGroup, null, {
            default: _withCtx(() => [
              _createVNode(_component_ButtonBase, {
                onClick: _cache[4] || (_cache[4] = ($event: any) => (showLowUsageVersions.value = false)),
                variant: _unref(showLowUsageVersions) ? 'secondary' : 'primary'
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('package.versions.grouping_usage_all')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["variant"]),
              _createVNode(_component_ButtonBase, {
                onClick: _cache[5] || (_cache[5] = ($event: any) => (showLowUsageVersions.value = true)),
                variant: _unref(showLowUsageVersions) ? 'primary' : 'secondary'
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('package.versions.grouping_usage_low')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["variant"])
            ]),
            _: 1 /* STABLE */
          }) ]) ]), _createElementVNode("h2", _hoisted_4, _toDisplayString(_ctx.$t('package.versions.distribution_title')), 1 /* TEXT */), _createElementVNode("div", {
        role: "region",
        "aria-labelledby": "version-distribution-title",
        class: _normalizeClass(["relative", isMobile.value ? 'min-h-[260px]' : 'min-h-[520px]'])
      }, [ (xyDataset.value.length > 0 && !_unref(error)) ? (_openBlock(), _createBlock(_component_ClientOnly, { key: 0 }, {
            fallback: _withCtx(() => [
              _hoisted_13
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", {
                class: "chart-container w-full",
                key: _unref(groupingMode)
              }, [
                _createVNode(VueUiXy, {
                  dataset: xyDataset.value,
                  config: chartConfig.value,
                  class: "[direction:ltr]"
                }, {
                  svg: _withCtx(({ svg }) => [
                    (svg.isPrintingSvg)
                      ? (_openBlock(), _createElementBlock("g", {
                        key: 0,
                        innerHTML: _unref(drawSvgPrintLegend)(svg, watermarkColors.value)
                      }))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n\n              "),
                    _createTextVNode("\n              "),
                    (svg.isPrintingSvg || svg.isPrintingImg)
                      ? (_openBlock(), _createElementBlock("g", {
                        key: 0,
                        innerHTML: _unref(drawNpmxLogoAndTaglineWatermark)(svg, watermarkColors.value, _ctx.$t, 'bottom')
                      }))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n\n              "),
                    _createTextVNode("\n              "),
                    (_unref(pending))
                      ? (_openBlock(), _createElementBlock("rect", {
                        key: 0,
                        x: svg.drawingArea.left,
                        y: svg.drawingArea.top - 12,
                        width: svg.drawingArea.width + 12,
                        height: svg.drawingArea.height + 48,
                        fill: _unref(colors).bg
                      }))
                      : _createCommentVNode("v-if", true)
                  ]),
                  "bar-gradient": _withCtx(({ series, positiveId }) => [
                    _createElementVNode("linearGradient", {
                      id: positiveId,
                      x1: "0",
                      x2: "0",
                      y1: "0",
                      y2: "1"
                    }, [
                      _createElementVNode("stop", {
                        offset: "0%",
                        "stop-color": _unref(lightenHex)(series.color, 0.618)
                      }, null, 8 /* PROPS */, ["stop-color"]),
                      _createElementVNode("stop", {
                        offset: "100%",
                        "stop-color": series.color,
                        "stop-opacity": "0.618"
                      }, null, 8 /* PROPS */, ["stop-color"])
                    ], 8 /* PROPS */, ["id"])
                  ]),
                  legend: _withCtx(({ legend }) => [
                    _createElementVNode("div", { class: "flex gap-4 flex-wrap justify-center pt-8" }, [
                      (legend.length > 0)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: "flex gap-1 place-items-center"
                        }, [
                          _createElementVNode("div", { class: "h-3 w-3" }, [
                            _createElementVNode("svg", {
                              viewBox: "0 0 2 2",
                              class: "w-full"
                            }, [
                              _createElementVNode("rect", {
                                x: "0",
                                y: "0",
                                width: "2",
                                height: "2",
                                rx: "0.3",
                                fill: legend[0]?.color
                              }, null, 8 /* PROPS */, ["fill"])
                            ])
                          ]),
                          _createElementVNode("span", null, _toDisplayString(legend[0]?.name), 1 /* TEXT */)
                        ]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  "reset-action": _withCtx(({ reset: resetMinimap }) => [
                    _createElementVNode("button", {
                      type: "button",
                      "aria-label": "reset minimap",
                      class: "absolute inset-is-1/2 -translate-x-1/2 -bottom-18 sm:inset-is-unset sm:translate-x-0 sm:bottom-auto sm:-inset-ie-20 sm:-top-3 flex items-center justify-center px-2.5 py-1.75 border border-transparent rounded-md text-fg-subtle hover:text-fg transition-colors hover:border-border focus-visible:outline-accent/70 sm:mb-0",
                      style: "pointer-events: all !important",
                      onClick: _cache[6] || (_cache[6] = (...args) => (_ctx.resetMinimap && _ctx.resetMinimap(...args)))
                    }, [
                      _hoisted_5
                    ])
                  ]),
                  menuIcon: _withCtx(({ isOpen }) => [
                    (isOpen)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "i-lucide:x w-6 h-6",
                        "aria-hidden": "true"
                      }))
                      : (_openBlock(), _createElementBlock("span", {
                        key: 1,
                        class: "i-lucide:ellipsis-vertical w-6 h-6",
                        "aria-hidden": "true"
                      }))
                  ]),
                  optionCsv: _withCtx(() => [
                    _hoisted_6
                  ]),
                  optionImg: _withCtx(() => [
                    _hoisted_7
                  ]),
                  optionSvg: _withCtx(() => [
                    _hoisted_8
                  ]),
                  "annotator-action-close": _withCtx(() => [
                    _hoisted_9
                  ]),
                  "annotator-action-color": _withCtx(({ color }) => [
                    _createElementVNode("span", {
                      class: "i-lucide:palette w-6 h-6",
                      style: { color: color },
                      "aria-hidden": "true"
                    })
                  ]),
                  "annotator-action-undo": _withCtx(() => [
                    _hoisted_10
                  ]),
                  "annotator-action-redo": _withCtx(() => [
                    _hoisted_11
                  ]),
                  "annotator-action-delete": _withCtx(() => [
                    _hoisted_12
                  ]),
                  optionAnnotator: _withCtx(({ isAnnotator }) => [
                    (isAnnotator)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "i-lucide:pen-off w-6 h-6 text-fg-subtle",
                        style: "pointer-events: none",
                        "aria-hidden": "true"
                      }))
                      : (_openBlock(), _createElementBlock("span", {
                        key: 1,
                        class: "i-lucide:pen w-6 h-6 text-fg-subtle",
                        style: "pointer-events: none",
                        "aria-hidden": "true"
                      }))
                  ]),
                  optionAltCopy: _withCtx(() => [
                    _createElementVNode("span", {
                      class: _normalizeClass(["w-6 h-6", 
                    _unref(copied) ? 'i-lucide:check text-accent' : 'i-lucide:person-standing text-fg-subtle'
                  ]),
                      style: "pointer-events: none",
                      "aria-hidden": "true"
                    }, null, 2 /* CLASS */)
                  ]),
                  default: _withCtx(() => [
                    _createTextVNode("\n\n            "),
                    _createTextVNode("\n            "),
                    _createTextVNode("\n\n            "),
                    _createTextVNode("\n            "),
                    _createTextVNode("\n\n            "),
                    _createTextVNode("\n            "),
                    _createTextVNode("\n\n            "),
                    _createTextVNode("\n            "),
                    _createTextVNode("\n\n            "),
                    _createTextVNode("\n            "),
                    _createTextVNode("\n\n            "),
                    _createTextVNode("\n            ")
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["dataset", "config"])
              ])
            ]),
            _: 1 /* STABLE */
          })) : _createCommentVNode("v-if", true), _createTextVNode("\n\n      " + "\n      "), (!_unref(hasData) && !_unref(pending) && !_unref(error)) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "flex items-center justify-center h-full"
          }, [ _createElementVNode("div", { class: "text-sm text-fg-subtle font-mono text-center flex flex-col items-center gap-2" }, [ _hoisted_14, _createElementVNode("p", null, _toDisplayString(_ctx.$t('package.trends.no_data')), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n      " + "\n      "), (_unref(error)) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "flex items-center justify-center h-full",
            role: "alert"
          }, [ _createElementVNode("div", { class: "text-sm text-fg-subtle font-mono text-center flex flex-col items-center gap-2" }, [ _hoisted_15, _createElementVNode("p", null, _toDisplayString(_unref(error).message), 1 /* TEXT */), _createElementVNode("p", _hoisted_16, "Package: " + _toDisplayString(__props.packageName), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n      " + "\n      "), (showLoadingIndicator.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            role: "status",
            "aria-live": "polite",
            class: "absolute top-1/2 inset-is-1/2 -translate-x-1/2 -translate-y-1/2"
          }, [ _createElementVNode("div", _hoisted_17, _toDisplayString(_ctx.$t('common.loading')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */) ], 8 /* PROPS */, ["aria-busy"]))
}
}

})
