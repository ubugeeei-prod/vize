import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vModelText as _vModelText, vModelCheckbox as _vModelCheckbox } from "vue"


const _hoisted_1 = { for: "startDate", class: "text-2xs font-mono text-fg-subtle tracking-wide uppercase" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "absolute inset-is-2 i-lucide:calendar w-4 h-4 text-fg-subtle shrink-0 pointer-events-none", "aria-hidden": "true" })
const _hoisted_3 = { for: "endDate", class: "text-2xs font-mono text-fg-subtle tracking-wide uppercase" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "absolute inset-is-2 i-lucide:calendar w-4 h-4 text-fg-subtle shrink-0 pointer-events-none", "aria-hidden": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "block i-lucide:undo-2 w-5 h-5", "aria-hidden": "true" })
const _hoisted_6 = { class: "text-fg-muted" }
const _hoisted_7 = { class: "text-fg-muted" }
const _hoisted_8 = { class: "text-xs text-fg-muted" }
const _hoisted_9 = { class: "text-xs text-fg-subtle font-medium" }
const _hoisted_10 = { id: "trends-chart-title", class: "sr-only" }
const _hoisted_11 = { class: "text-fg-subtle" }
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:undo-2 w-5 h-5", "aria-hidden": "true" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle font-mono pointer-events-none" }, "CSV")
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle font-mono pointer-events-none" }, "PNG")
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("span", { class: "text-fg-subtle font-mono pointer-events-none" }, "SVG")
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-6 h-6 text-fg-subtle", style: "pointer-events: none", "aria-hidden": "true" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:undo-2 w-6 h-6 text-fg-subtle", style: "pointer-events: none", "aria-hidden": "true" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:redo-2 w-6 h-6 text-fg-subtle", style: "pointer-events: none", "aria-hidden": "true" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:trash w-6 h-6 text-fg-subtle", style: "pointer-events: none", "aria-hidden": "true" })
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("div", { class: "min-h-[260px]" })
import type { Theme as VueDataUiTheme, VueUiXyConfig, VueUiXyDatasetItem } from 'vue-data-ui'
import { VueUiXy } from 'vue-data-ui/vue-ui-xy'
import { useDebounceFn, useElementSize } from '@vueuse/core'
import { useCssVariables } from '~/composables/useColors'
import { OKLCH_NEUTRAL_FALLBACK, transparentizeOklch, lightenOklch } from '~/utils/colors'
import { getFrameworkColor, isListedFramework } from '~/utils/frameworks'
import { drawNpmxLogoAndTaglineWatermark } from '~/composables/useChartWatermark'
import type { RepoRef } from '#shared/utils/git-providers'
import type { ChartTimeGranularity, DailyDataPoint, DateRangeFields, EvolutionData, EvolutionOptions, MonthlyDataPoint, WeeklyDataPoint, YearlyDataPoint } from '~/types/chart'
import { DATE_INPUT_MAX } from '~/utils/input'
import { applyDataCorrection } from '~/utils/chart-data-correction'
import { applyBlocklistCorrection, getAnomaliesForPackages } from '~/utils/download-anomalies'
import { copyAltTextForTrendLineChart } from '~/utils/charts'

type MetricId = 'downloads' | 'likes' | 'contributors'
type MetricContext = {
  packageName: string
  repoRef?: RepoRef | null
}
type MetricDef = {
  id: MetricId
  label: string
  fetch: (context: MetricContext, options: EvolutionOptions) => Promise<EvolutionData>
  supportsMulti?: boolean
}
const mobileBreakpointWidth = 640
const DEFAULT_GRANULARITY: ChartTimeGranularity = 'weekly'
const DEFAULT_METRIC_ID: MetricId = 'downloads'

export default /*@__PURE__*/_defineComponent({
  __name: 'TrendsChart',
  props: {
    weeklyDownloads: { type: Array, required: false },
    inModal: { type: Boolean, required: false },
    packageName: { type: String, required: false },
    packageNames: { type: Array, required: false },
    repoRef: { type: null, required: false },
    createdIso: { type: String, required: false },
    showFacetSelector: { type: Boolean, required: false },
    permalink: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props
const { locale } = useI18n()
const { accentColors, selectedAccentColor } = useAccentColor()
const { settings } = useSettings()
const { copy, copied } = useClipboard()
const colorMode = useColorMode()
const resolvedMode = shallowRef<'light' | 'dark'>('light')
const rootEl = shallowRef<HTMLElement | null>(null)
const isZoomed = shallowRef(false)
function setIsZoom({ isZoom }: { isZoom: boolean }) {
  isZoomed.value = isZoom
}
const { width } = useElementSize(rootEl)
const compactNumberFormatter = useCompactNumberFormatter()
onMounted(async () => {
  rootEl.value = document.documentElement
  resolvedMode.value = colorMode.value === 'dark' ? 'dark' : 'light'
  initDateRangeFromWeekly()
  initDateRangeForMultiPackageWeekly52()
  initDateRangeFallbackClient()
  await nextTick()
  isMounted.value = true
  loadMetric(selectedMetric.value)
})
const { colors } = useCssVariables(
  [
    '--bg',
    '--fg',
    '--bg-subtle',
    '--bg-elevated',
    '--fg-subtle',
    '--fg-muted',
    '--border',
    '--border-subtle',
  ],
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
const isMobile = computed(() => width.value > 0 && width.value < mobileBreakpointWidth)
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}
function isWeeklyDataset(data: unknown): data is WeeklyDataPoint[] {
  return (
    Array.isArray(data) &&
    data.length > 0 &&
    isRecord(data[0]) &&
    'weekStart' in data[0] &&
    'weekEnd' in data[0] &&
    'value' in data[0]
  )
}
function isDailyDataset(data: unknown): data is DailyDataPoint[] {
  return (
    Array.isArray(data) &&
    data.length > 0 &&
    isRecord(data[0]) &&
    'day' in data[0] &&
    'value' in data[0]
  )
}
function isMonthlyDataset(data: unknown): data is MonthlyDataPoint[] {
  return (
    Array.isArray(data) &&
    data.length > 0 &&
    isRecord(data[0]) &&
    'month' in data[0] &&
    'value' in data[0]
  )
}
function isYearlyDataset(data: unknown): data is YearlyDataPoint[] {
  return (
    Array.isArray(data) &&
    data.length > 0 &&
    isRecord(data[0]) &&
    'year' in data[0] &&
    'value' in data[0]
  )
}
/**
 * Formats a single evolution dataset into the structure expected by `VueUiXy`
 * for single-series charts.
 *
 * The dataset is interpreted based on the selected time granularity:
 * - **daily**   → uses `timestamp`
 * - **weekly**  → uses `timestampEnd`
 * - **monthly** → uses `timestamp`
 * - **yearly**  → uses `timestamp`
 *
 * Only datasets matching the expected shape for the given granularity are
 * accepted. If the dataset does not match, an empty result is returned.
 *
 * The returned structure includes:
 * - a single line-series dataset with a consistent color
 * - a list of timestamps used as the x-axis values
 *
 * @param selectedGranularity - Active chart time granularity
 * @param dataset - Raw evolution dataset to format
 * @param seriesName - Display name for the resulting series
 * @returns An object containing a formatted dataset and its associated dates,
 *          or `{ dataset: null, dates: [] }` when the input is incompatible
 */
function formatXyDataset(
  selectedGranularity: ChartTimeGranularity,
  dataset: EvolutionData,
  seriesName: string,
): { dataset: VueUiXyDatasetItem[] | null; dates: number[] } {
  const lightColor = isDarkMode.value ? lightenOklch(accent.value, 0.618) : undefined
  // Subtle path gradient applied in dark mode only
  const temperatureColors = lightColor ? [lightColor, accent.value] : undefined
  const datasetItem: VueUiXyDatasetItem = {
    name: seriesName,
    type: 'line',
    series: dataset.map(d => d.value),
    color: accent.value,
    temperatureColors,
    useArea: true,
    dashIndices: dataset
      .map((item, index) => (item.hasAnomaly ? index : -1))
      .filter(index => index !== -1),
  }
  if (selectedGranularity === 'weekly' && isWeeklyDataset(dataset)) {
    return {
      dataset: [datasetItem],
      dates: dataset.map(d => d.timestampEnd),
    }
  }
  if (selectedGranularity === 'daily' && isDailyDataset(dataset)) {
    return {
      dataset: [datasetItem],
      dates: dataset.map(d => d.timestamp),
    }
  }
  if (selectedGranularity === 'monthly' && isMonthlyDataset(dataset)) {
    return {
      dataset: [datasetItem],
      dates: dataset.map(d => d.timestamp),
    }
  }
  if (selectedGranularity === 'yearly' && isYearlyDataset(dataset)) {
    return {
      dataset: [datasetItem],
      dates: dataset.map(d => d.timestamp),
    }
  }
  return { dataset: null, dates: [] }
}
/**
 * Extracts normalized time-series points from an evolution dataset based on
 * the selected time granularity.
 *
 * Each returned point contains:
 * - `timestamp`: the numeric time value used for x-axis alignment
 * - `value`: the corresponding value at that time
 *
 * The timestamp field is selected according to granularity:
 * - **daily**   → `timestamp`
 * - **weekly**  → `timestampEnd`
 * - **monthly** → `timestamp`
 * - **yearly**  → `timestamp`
 *
 * If the dataset does not match the expected shape for the given granularity,
 * an empty array is returned.
 *
 * This helper is primarily used in multi-package mode to align multiple
 * datasets on a shared time axis.
 *
 * @param selectedGranularity - Active chart time granularity
 * @param dataset - Raw evolution dataset to extract points from
 * @returns An array of normalized `{ timestamp, value }` points
 */
function extractSeriesPoints(
  selectedGranularity: ChartTimeGranularity,
  dataset: EvolutionData,
): Array<{ timestamp: number; value: number; hasAnomaly: boolean }> {
  if (selectedGranularity === 'weekly' && isWeeklyDataset(dataset)) {
    return dataset.map(d => ({
      timestamp: d.timestampEnd,
      value: d.value,
      hasAnomaly: !!d.hasAnomaly,
    }))
  }
  if (
    (selectedGranularity === 'daily' && isDailyDataset(dataset)) ||
    (selectedGranularity === 'monthly' && isMonthlyDataset(dataset)) ||
    (selectedGranularity === 'yearly' && isYearlyDataset(dataset))
  ) {
    return (dataset as Array<{ timestamp: number; value: number; hasAnomaly?: boolean }>).map(
      d => ({
        timestamp: d.timestamp,
        value: d.value,
        hasAnomaly: !!d.hasAnomaly,
      }),
    )
  }
  return []
}
function toIsoDateOnly(value: string): string {
  return value.slice(0, 10)
}
function isValidIsoDateOnly(value: string): boolean {
  return /^\d{4}-\d{2}-\d{2}$/.test(value)
}
function safeMin(a: string, b: string): string {
  return a.localeCompare(b) <= 0 ? a : b
}
function safeMax(a: string, b: string): string {
  return a.localeCompare(b) >= 0 ? a : b
}
/**
 * Multi-package mode detection:
 * packageNames has entries, and packageName is not set.
 */
const isMultiPackageMode = computed(() => {
  const names = (props.packageNames ?? []).map(n => String(n).trim()).filter(Boolean)
  const single = String(props.packageName ?? '').trim()
  return names.length > 0 && !single
})
const effectivePackageNames = computed<string[]>(() => {
  if (isMultiPackageMode.value)
    return (props.packageNames ?? []).map(n => String(n).trim()).filter(Boolean)
  const single = String(props.packageName ?? '').trim()
  return single ? [single] : []
})
const {
  fetchPackageDownloadEvolution,
  fetchPackageLikesEvolution,
  fetchRepoContributorsEvolution,
  fetchRepoRefsForPackages,
} = useCharts()
const repoRefsByPackage = shallowRef<Record<string, RepoRef | null>>({})
const repoRefsRequestToken = shallowRef(0)
watch(
  () => effectivePackageNames.value,
  async names => {
    if (!import.meta.client) return
    if (!isMultiPackageMode.value) {
      repoRefsByPackage.value = {}
      return
    }
    const currentToken = ++repoRefsRequestToken.value
    const refs = await fetchRepoRefsForPackages(names)
    if (currentToken !== repoRefsRequestToken.value) return
    repoRefsByPackage.value = refs
  },
  { immediate: true },
)
const selectedGranularity = usePermalink<ChartTimeGranularity>('granularity', DEFAULT_GRANULARITY, {
  permanent: props.permalink,
})
const displayedGranularity = shallowRef<ChartTimeGranularity>(DEFAULT_GRANULARITY)
const isEndDateOnPeriodEnd = computed(() => {
  const g = selectedGranularity.value
  if (g !== 'monthly' && g !== 'yearly') return false

  const iso = String(endDate.value ?? '').slice(0, 10)
  if (!/^\d{4}-\d{2}-\d{2}$/.test(iso)) return false

  const [year, month, day] = iso.split('-').map(Number)
  if (!year || !month || !day) return false

  // Monthly: endDate is the last day of its month (UTC)
  if (g === 'monthly') {
    const lastDayOfMonth = new Date(Date.UTC(year, month, 0)).getUTCDate()
    return day === lastDayOfMonth
  }

  // Yearly: endDate is the last day of the year (UTC)
  return month === 12 && day === 31
})
const isEstimationGranularity = computed(
  () => displayedGranularity.value === 'monthly' || displayedGranularity.value === 'yearly',
)
const supportsEstimation = computed(
  () => isEstimationGranularity.value && selectedMetric.value !== 'contributors',
)
const hasDownloadAnomalies = computed(() =>
  normalisedDataset.value?.some(datapoint => !!datapoint.dashIndices.length),
)
const shouldRenderEstimationOverlay = computed(() => !pending.value && supportsEstimation.value)
const startDate = usePermalink<string>('start', '', {
  permanent: props.permalink,
})
const endDate = usePermalink<string>('end', '', {
  permanent: props.permalink,
})
const hasUserEditedDates = shallowRef(false)
/**
 * Initializes the date range from the provided weeklyDownloads dataset.
 *
 * The range is inferred directly from the dataset boundaries:
 * - `startDate` is set from the `weekStart` of the first entry
 * - `endDate` is set from the `weekEnd` of the last entry
 *
 * Dates are normalized to `YYYY-MM-DD` and validated before assignment.
 *
 * This function is a no-op when:
 * - the user has already edited the date range
 * - no weekly download data is available
 *
 * The inferred range takes precedence over client-side fallbacks but does not
 * override user-defined dates.
 */
function initDateRangeFromWeekly() {
  if (hasUserEditedDates.value) return
  if (!props.weeklyDownloads?.length) return
  const first = props.weeklyDownloads[0]
  const last = props.weeklyDownloads[props.weeklyDownloads.length - 1]
  const start = first?.weekStart ? toIsoDateOnly(first.weekStart) : ''
  const end = last?.weekEnd ? toIsoDateOnly(last.weekEnd) : ''
  if (isValidIsoDateOnly(start)) startDate.value = start
  if (isValidIsoDateOnly(end)) endDate.value = end
}
/**
 * Initializes a default date range on the client when no explicit dates
 * have been provided and the user has not manually edited the range, typically
 * when weeklyDownloads is not provided.
 *
 * The range is computed in UTC to avoid timezone-related off-by-one errors:
 * - `endDate` is set to yesterday (UTC)
 * - `startDate` is set to 29 days before yesterday (UTC), yielding a 30-day range
 *
 * This function is a no-op when:
 * - the user has already edited the date range
 * - the code is running on the server
 * - both `startDate` and `endDate` are already defined
 */
function initDateRangeFallbackClient() {
  if (hasUserEditedDates.value) return
  if (!import.meta.client) return
  if (startDate.value && endDate.value) return
  const today = new Date()
  const yesterday = new Date(
    Date.UTC(today.getUTCFullYear(), today.getUTCMonth(), today.getUTCDate() - 1),
  )
  const end = yesterday.toISOString().slice(0, 10)
  const startObj = new Date(yesterday)
  startObj.setUTCDate(startObj.getUTCDate() - 29)
  const start = startObj.toISOString().slice(0, 10)
  if (!startDate.value) startDate.value = start
  if (!endDate.value) endDate.value = end
}
function toUtcDateOnly(date: Date): string {
  return date.toISOString().slice(0, 10)
}
function addUtcDays(date: Date, days: number): Date {
  const next = new Date(date)
  next.setUTCDate(next.getUTCDate() + days)
  return next
}
/**
 * Initializes a default date range for multi-package mode using a fixed
 * 52-week rolling window.
 *
 * The range is computed in UTC to ensure consistent boundaries across
 * timezones:
 * - `endDate` is set to yesterday (UTC)
 * - `startDate` is set to the first day of the 52-week window ending yesterday
 *
 * This function is intended for multi-package comparisons where no explicit
 * date range or dataset-derived range is available.
 *
 * This function is a no-op when:
 * - the user has already edited the date range
 * - the code is running on the server
 * - the component is not in multi-package mode
 * - both `startDate` and `endDate` are already defined
 */
function initDateRangeForMultiPackageWeekly52() {
  if (hasUserEditedDates.value) return
  if (!import.meta.client) return
  if (!isMultiPackageMode.value) return
  if (startDate.value && endDate.value) return
  const today = new Date()
  const yesterday = new Date(
    Date.UTC(today.getUTCFullYear(), today.getUTCMonth(), today.getUTCDate() - 1),
  )
  endDate.value = toUtcDateOnly(yesterday)
  startDate.value = toUtcDateOnly(addUtcDays(yesterday, -(52 * 7) + 1))
}
watch(
  () => (props.packageNames ?? []).length,
  () => {
    initDateRangeForMultiPackageWeekly52()
  },
  { immediate: true },
)
const initialStartDate = shallowRef<string>('') // YYYY-MM-DD
const initialEndDate = shallowRef<string>('') // YYYY-MM-DD
function setInitialRangeIfEmpty() {
  if (initialStartDate.value || initialEndDate.value) return
  if (startDate.value) initialStartDate.value = startDate.value
  if (endDate.value) initialEndDate.value = endDate.value
}
watch(
  [startDate, endDate],
  () => {
    if (startDate.value || endDate.value) hasUserEditedDates.value = true
    setInitialRangeIfEmpty()
  },
  { immediate: true, flush: 'post' },
)
const showResetButton = computed(() => {
  if (!initialStartDate.value && !initialEndDate.value) return false
  return startDate.value !== initialStartDate.value || endDate.value !== initialEndDate.value
})
function resetDateRange() {
  hasUserEditedDates.value = false
  startDate.value = ''
  endDate.value = ''
  initDateRangeFromWeekly()
  initDateRangeForMultiPackageWeekly52()
  initDateRangeFallbackClient()
}
const options = shallowRef<
  | { granularity: 'day'; startDate?: string; endDate?: string }
  | { granularity: 'week'; weeks: number; startDate?: string; endDate?: string }
  | {
      granularity: 'month'
      months: number
      startDate?: string
      endDate?: string
    }
  | { granularity: 'year'; startDate?: string; endDate?: string }
>({ granularity: 'week', weeks: 52 })
/**
 * Applies the current date range (`startDate` / `endDate`) to a base options
 * object, returning a new object augmented with validated date fields.
 *
 * Dates are normalized to `YYYY-MM-DD`, validated, and ordered to ensure
 * logical consistency:
 * - When both dates are valid, the earliest is assigned to `startDate` and
 *   the latest to `endDate`
 * - When only one valid date is present, only that boundary is applied
 * - Invalid or empty dates are omitted from the result
 *
 * The input object is not mutated.
 *
 * @typeParam T - Base options type to extend with date range fields
 * @param base - Base options object to which the date range should be applied
 * @returns A new options object including the applicable `startDate` and/or
 *          `endDate` fields
 */
function applyDateRange<T extends Record<string, unknown>>(base: T): T & DateRangeFields {
  const next: T & DateRangeFields = { ...base }
  const start = startDate.value ? toIsoDateOnly(startDate.value) : ''
  const end = endDate.value ? toIsoDateOnly(endDate.value) : ''
  const validStart = start && isValidIsoDateOnly(start) ? start : ''
  const validEnd = end && isValidIsoDateOnly(end) ? end : ''
  if (validStart && validEnd) {
    next.startDate = safeMin(validStart, validEnd)
    next.endDate = safeMax(validStart, validEnd)
  } else {
    if (validStart) next.startDate = validStart
    else delete next.startDate
    if (validEnd) next.endDate = validEnd
    else delete next.endDate
  }
  return next
}
const hasContributorsFacet = computed(() => {
  if (isMultiPackageMode.value) {
    return Object.values(repoRefsByPackage.value).some(ref => ref?.provider === 'github')
  }
  const ref = props.repoRef
  return ref?.provider === 'github' && ref.owner && ref.repo
})
const METRICS = computed<MetricDef[]>(() => {
  const metrics: MetricDef[] = [
    {
      id: 'downloads',
      label: $t('package.trends.items.downloads'),
      fetch: ({ packageName }, opts) =>
        fetchPackageDownloadEvolution(
          packageName,
          props.createdIso ?? null,
          opts,
        ) as Promise<EvolutionData>,
      supportsMulti: true,
    },
    {
      id: 'likes',
      label: $t('package.trends.items.likes'),
      fetch: ({ packageName }, opts) => fetchPackageLikesEvolution(packageName, opts),
      supportsMulti: true,
    },
  ]

  if (hasContributorsFacet.value) {
    metrics.push({
      id: 'contributors',
      label: $t('package.trends.items.contributors'),
      fetch: ({ repoRef }, opts) => fetchRepoContributorsEvolution(repoRef, opts),
      supportsMulti: true,
    })
  }

  return metrics
})
const selectedMetric = usePermalink<MetricId>('facet', DEFAULT_METRIC_ID, {
  permanent: props.permalink,
})
const effectivePackageNamesForMetric = computed<string[]>(() => {
  if (!isMultiPackageMode.value) return effectivePackageNames.value
  if (selectedMetric.value !== 'contributors') return effectivePackageNames.value
  return effectivePackageNames.value.filter(
    name => repoRefsByPackage.value[name]?.provider === 'github',
  )
})
const skippedPackagesWithoutGitHub = computed(() => {
  if (!isMultiPackageMode.value) return []
  if (selectedMetric.value !== 'contributors') return []
  if (!effectivePackageNames.value.length) return []

  return effectivePackageNames.value.filter(
    name => repoRefsByPackage.value[name]?.provider !== 'github',
  )
})
const availableGranularities = computed<ChartTimeGranularity[]>(() => {
  if (selectedMetric.value === 'contributors') {
    return ['weekly', 'monthly', 'yearly']
  }

  return ['daily', 'weekly', 'monthly', 'yearly']
})
watch(
  () => [selectedMetric.value, availableGranularities.value] as const,
  () => {
    if (!availableGranularities.value.includes(selectedGranularity.value)) {
      selectedGranularity.value = 'weekly'
    }
  },
  { immediate: true },
)
watch(
  () => METRICS.value,
  metrics => {
    if (!metrics.some(m => m.id === selectedMetric.value)) {
      selectedMetric.value = DEFAULT_METRIC_ID
    }
  },
  { immediate: true },
)
// Per-metric state keyed by metric id
const metricStates = reactive<
  Record<
    MetricId,
    {
      pending: boolean
      evolution: EvolutionData
      evolutionsByPackage: Record<string, EvolutionData>
      requestToken: number
    }
  >
>({
  downloads: {
    pending: false,
    evolution: props.weeklyDownloads ?? [],
    evolutionsByPackage: {},
    requestToken: 0,
  },
  likes: {
    pending: false,
    evolution: [],
    evolutionsByPackage: {},
    requestToken: 0,
  },
  contributors: {
    pending: false,
    evolution: [],
    evolutionsByPackage: {},
    requestToken: 0,
  },
})
const activeMetricState = computed(() => metricStates[selectedMetric.value])
const activeMetricDef = computed(
  () => METRICS.value.find(m => m.id === selectedMetric.value) ?? METRICS.value[0],
)
const pending = computed(() => activeMetricState.value.pending)
const isMounted = shallowRef(false)
// Watches granularity and date inputs to keep request options in sync and
// manage the loading state.
//
// This watcher does NOT perform the fetch itself. Its responsibilities are:
// - derive the correct API options from the selected granularity
// - apply the current validated date range to those options
// - determine whether a loading indicator should be shown
//
// Fetching is debounced separately to avoid excessive
// network requests while the user is interacting with controls.
watch(
  [selectedGranularity, startDate, endDate],
  ([granularityValue]) => {
    if (granularityValue === 'daily') options.value = applyDateRange({ granularity: 'day' })
    else if (granularityValue === 'weekly')
      options.value = applyDateRange({ granularity: 'week', weeks: 52 })
    else if (granularityValue === 'monthly')
      options.value = applyDateRange({ granularity: 'month', months: 24 })
    else options.value = applyDateRange({ granularity: 'year' })
    // Do not set pending during initial setup
    if (!isMounted.value) return
    const packageNames = effectivePackageNames.value
    if (!import.meta.client || !packageNames.length) {
      activeMetricState.value.pending = false
      return
    }
    const o = options.value
    const hasExplicitRange = ('startDate' in o && o.startDate) || ('endDate' in o && o.endDate)
    // Do not show loading when weeklyDownloads is already provided
    if (
      selectedMetric.value === DEFAULT_METRIC_ID &&
      !isMultiPackageMode.value &&
      granularityValue === DEFAULT_GRANULARITY &&
      props.weeklyDownloads?.length &&
      !hasExplicitRange
    ) {
      activeMetricState.value.pending = false
      return
    }
    activeMetricState.value.pending = true
  },
  { immediate: true },
)
/**
 * Fetches evolution data for a given metric based on the current granularity,
 * date range, and package selection.
 *
 * This function:
 * - runs only on the client
 * - supports both single-package and multi-package modes
 * - applies request de-duplication via a request token to avoid race conditions
 * - updates the appropriate reactive stores with fetched data
 * - manages the metric's `pending` loading state
 */
async function loadMetric(metricId: MetricId) {
  if (!import.meta.client) return
  const state = metricStates[metricId]
  const metric = METRICS.value.find(m => m.id === metricId)!
  const currentToken = ++state.requestToken
  state.pending = true
  const fetchFn = (context: MetricContext) => metric.fetch(context, options.value)
  try {
    const packageNames = effectivePackageNamesForMetric.value
    if (!packageNames.length) {
      if (isMultiPackageMode.value) state.evolutionsByPackage = {}
      else state.evolution = []
      displayedGranularity.value = selectedGranularity.value
      return
    }
    if (isMultiPackageMode.value) {
      if (metric.supportsMulti === false) {
        state.evolutionsByPackage = {}
        displayedGranularity.value = selectedGranularity.value
        return
      }
      const settled = await Promise.allSettled(
        packageNames.map(async pkg => {
          const repoRef = metricId === 'contributors' ? repoRefsByPackage.value[pkg] : null
          const result = await fetchFn({ packageName: pkg, repoRef })
          return { pkg, result: (result ?? []) as EvolutionData }
        }),
      )
      if (currentToken !== state.requestToken) return
      const next: Record<string, EvolutionData> = {}
      for (const entry of settled) {
        if (entry.status === 'fulfilled') next[entry.value.pkg] = entry.value.result
      }
      state.evolutionsByPackage = next
      displayedGranularity.value = selectedGranularity.value
      return
    }
    const pkg = packageNames[0] ?? ''
    if (!pkg) {
      state.evolution = []
      displayedGranularity.value = selectedGranularity.value
      return
    }
    // In single-package mode the parent already fetches weekly downloads for the
    // sparkline (WeeklyDownloadStats). When the user hasn't customised the date
    // range we can reuse that prop directly and skip a redundant API call.
    if (metricId === DEFAULT_METRIC_ID) {
      const o = options.value
      const hasExplicitRange = ('startDate' in o && o.startDate) || ('endDate' in o && o.endDate)
      if (
        selectedGranularity.value === DEFAULT_GRANULARITY &&
        props.weeklyDownloads?.length &&
        !hasExplicitRange
      ) {
        state.evolution = props.weeklyDownloads
        displayedGranularity.value = DEFAULT_GRANULARITY
        return
      }
    }
    const result = await fetchFn({ packageName: pkg, repoRef: props.repoRef })
    if (currentToken !== state.requestToken) return
    state.evolution = (result ?? []) as EvolutionData
    displayedGranularity.value = selectedGranularity.value
  } catch {
    if (currentToken !== state.requestToken) return
    if (isMultiPackageMode.value) state.evolutionsByPackage = {}
    else state.evolution = []
  } finally {
    if (currentToken === state.requestToken) state.pending = false
  }
}
// Debounced wrapper around `loadNow` to avoid triggering a network request
// on every intermediate state change while the user is interacting with inputs
//
// This 'arbitrary' 1000 ms delay:
// - gives enough time for the user to finish changing granularity or dates
// - prevents unnecessary API load and visual flicker of the loading state
//
const debouncedLoadNow = useDebounceFn(() => {
  loadMetric(selectedMetric.value)
}, 1000)
const fetchTriggerKey = computed(() => {
  const names = effectivePackageNames.value.join(',')
  const o = options.value
  const repoKey = props.repoRef
    ? `${props.repoRef.provider}:${props.repoRef.owner}/${props.repoRef.repo}`
    : ''
  return [
    isMultiPackageMode.value ? 'M' : 'S',
    names,
    repoKey,
    String(props.createdIso ?? ''),
    String(o.granularity ?? ''),
    String('weeks' in o ? (o.weeks ?? '') : ''),
    String('months' in o ? (o.months ?? '') : ''),
    String('startDate' in o ? (o.startDate ?? '') : ''),
    String('endDate' in o ? (o.endDate ?? '') : ''),
  ].join('|')
})
watch(
  () => fetchTriggerKey.value,
  () => {
    if (!import.meta.client) return
    if (!isMounted.value) return
    debouncedLoadNow()
  },
  { flush: 'post' },
)
watch(
  () => repoRefsByPackage.value,
  () => {
    if (!import.meta.client) return
    if (!isMounted.value) return
    if (!isMultiPackageMode.value) return
    if (selectedMetric.value !== 'contributors') return
    debouncedLoadNow()
  },
  { deep: true },
)
const effectiveDataSingle = computed<EvolutionData>(() => {
  const state = activeMetricState.value
  let data: EvolutionData
  if (
    selectedMetric.value === DEFAULT_METRIC_ID &&
    displayedGranularity.value === DEFAULT_GRANULARITY &&
    props.weeklyDownloads?.length
  ) {
    data =
      isWeeklyDataset(state.evolution) && state.evolution.length
        ? state.evolution
        : props.weeklyDownloads
  } else {
    data = state.evolution
  }

  if (isDownloadsMetric.value && data.length) {
    const pkg = effectivePackageNames.value[0] ?? props.packageName ?? ''
    if (settings.value.chartFilter.anomaliesFixed) {
      data = applyBlocklistCorrection({
        data,
        packageName: pkg,
        granularity: displayedGranularity.value,
      })
    }

    return applyDataCorrection(
      data as Array<{ value: number }>,
      settings.value.chartFilter,
    ) as EvolutionData
  }

  return data
})
/**
 * Normalized chart data derived from the active metric's evolution datasets.
 *
 * Adapts its behavior based on the current mode:
 * - **Single-package mode**: formats via `formatXyDataset`
 * - **Multi-package mode**: merges datasets into a shared time axis
 * The returned structure matches the expectations of `VueUiXy`:
 * - `dataset`: array of series definitions, or `null` when no data is available
 * - `dates`: sorted list of timestamps used as the x-axis reference
 *
 * Returning `dataset: null` explicitly signals the absence of data and allows
 * the template to handle empty states without ambiguity.
 */
const chartData = computed<{
  dataset: VueUiXyDatasetItem[] | null
  dates: number[]
}>(() => {
  if (!isMultiPackageMode.value) {
    const pkg = effectivePackageNames.value[0] ?? props.packageName ?? ''
    return formatXyDataset(displayedGranularity.value, effectiveDataSingle.value, pkg)
  }

  const state = activeMetricState.value
  const names = effectivePackageNamesForMetric.value
  const granularity = displayedGranularity.value

  const timestampSet = new Set<number>()
  const pointsByPackage = new Map<
    string,
    Array<{ timestamp: number; value: number; hasAnomaly?: boolean }>
  >()

  for (const pkg of names) {
    let data = state.evolutionsByPackage[pkg] ?? []
    if (isDownloadsMetric.value && data.length) {
      if (settings.value.chartFilter.anomaliesFixed) {
        data = applyBlocklistCorrection({ data, packageName: pkg, granularity })
      }
      data = applyDataCorrection(
        data as Array<{ value: number }>,
        settings.value.chartFilter,
      ) as EvolutionData
    }
    const points = extractSeriesPoints(granularity, data)

    pointsByPackage.set(pkg, points)
    for (const p of points) timestampSet.add(p.timestamp)
  }

  const dates = Array.from(timestampSet).sort((a, b) => a - b)
  if (!dates.length) return { dataset: null, dates: [] }

  const dataset: VueUiXyDatasetItem[] = names.map(pkg => {
    const points = pointsByPackage.get(pkg) ?? []
    const valueByTimestamp = new Map<number, number>()
    const anomalyTimestamps = new Set<number>()
    for (const p of points) {
      valueByTimestamp.set(p.timestamp, p.value)
      if (p.hasAnomaly) anomalyTimestamps.add(p.timestamp)
    }

    const series = dates.map(t => valueByTimestamp.get(t) ?? 0)
    const dashIndices = dates
      .map((t, index) => (anomalyTimestamps.has(t) ? index : -1))
      .filter(index => index !== -1)

    const item: VueUiXyDatasetItem = {
      name: pkg,
      type: 'line',
      series,
      dashIndices,
    } as VueUiXyDatasetItem

    if (isListedFramework(pkg)) {
      item.color = getFrameworkColor(pkg)
    }
    return item
  })

  return { dataset, dates }
})
const normalisedDataset = computed(() => {
  return chartData.value.dataset?.map(d => {
    const lastValue = d.series.at(-1) ?? 0

    // Contributors is an absolute metric: keep the partial period value as-is.
    const projectedLastValue =
      selectedMetric.value === 'contributors' ? lastValue : extrapolateLastValue(lastValue)

    return {
      ...d,
      series: [...d.series.slice(0, -1), projectedLastValue],
      dashIndices: d.dashIndices ?? [],
    }
  })
})
const maxDatapoints = computed(() =>
  Math.max(0, ...(chartData.value.dataset ?? []).map(d => d.series.length)),
)
const loadFile = (link: string, filename: string) => {
  const a = document.createElement('a')
  a.href = link
  a.download = filename
  a.click()
  a.remove()
}
const datetimeFormatterOptions = computed(() => {
  return {
    daily: { year: 'yyyy-MM-dd', month: 'yyyy-MM-dd', day: 'yyyy-MM-dd' },
    weekly: { year: 'yyyy-MM-dd', month: 'yyyy-MM-dd', day: 'yyyy-MM-dd' },
    monthly: { year: 'MMM yyyy', month: 'MMM yyyy', day: 'MMM yyyy' },
    yearly: { year: 'yyyy', month: 'yyyy', day: 'yyyy' },
  }[selectedGranularity.value]
})
const sanitise = (value: string) =>
  value
    .replace(/^@/, '')
    .replace(/[\\/:"*?<>|]/g, '-')
    .replace(/\//g, '-')
function buildExportFilename(extension: string): string {
  const g = selectedGranularity.value
  const range = `${startDate.value}_${endDate.value}`
  if (!isMultiPackageMode.value) {
    const name = effectivePackageNames.value[0] ?? props.packageName ?? 'package'
    return `${sanitise(name)}-${g}_${range}.${extension}`
  }
  const names = effectivePackageNames.value
  const label = names.length === 1 ? names[0] : names.join('_')
  return `${sanitise(label ?? '')}-${g}_${range}.${extension}`
}
const granularityLabels = computed(() => ({
  daily: $t('package.trends.granularity_daily'),
  weekly: $t('package.trends.granularity_weekly'),
  monthly: $t('package.trends.granularity_monthly'),
  yearly: $t('package.trends.granularity_yearly'),
}))
function getGranularityLabel(granularity: ChartTimeGranularity) {
  return granularityLabels.value[granularity]
}
const granularityItems = computed(() =>
  availableGranularities.value.map(granularity => ({
    label: granularityLabels.value[granularity],
    value: granularity,
  })),
)
function clampRatio(value: number): number {
  if (value < 0) return 0
  if (value > 1) return 1
  return value
}
/**
 * Convert a `YYYY-MM-DD` date to UTC timestamp representing the end of that day.
 * The returned timestamp corresponds to `23:59:59.999` in UTC
 *
 * @param endDateOnly - ISO-like date string (`YYYY-MM-DD`)
 * @returns The UTC timestamp in milliseconds for the end of the given day,
 * or `null` if the input is invalid.
 */
function endDateOnlyToUtcMs(endDateOnly: string): number | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(endDateOnly)) return null
  const [y, m, d] = endDateOnly.split('-').map(Number)
  if (!y || !m || !d) return null
  return Date.UTC(y, m - 1, d, 23, 59, 59, 999)
}
/**
 * Computes the UTC timestamp corresponding to the start of the time bucket
 * that contains the given timestamp.
 *
 * This function is used to derive period boundaries when computing completion
 * ratios or extrapolating values for partially completed periods.
 *
 * Bucket boundaries are defined in UTC:
 * - **monthly** : first day of the month at `00:00:00.000` UTC
 * - **yearly** : January 1st of the year at `00:00:00.000` UTC
 *
 * @param timestampMs - Reference timestamp in milliseconds
 * @param granularity - Bucket granularity (`monthly` or `yearly`)
 * @returns The UTC timestamp representing the start of the corresponding
 * time bucket.
 */
function getBucketStartUtc(timestampMs: number, granularity: 'monthly' | 'yearly'): number {
  const date = new Date(timestampMs)
  if (granularity === 'yearly') return Date.UTC(date.getUTCFullYear(), 0, 1, 0, 0, 0, 0)
  return Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1, 0, 0, 0, 0)
}
/**
 * Computes the UTC timestamp corresponding to the end of the time
 * bucket that contains the given timestamp. This end timestamp is paired with `getBucketStartUtc` to define
 * a half-open interval `[start, end)` when computing elapsed time or completion
 * ratios within a period.
 *
 * Bucket boundaries are defined in UTC and are **exclusive**:
 * - **monthly** : first day of the following month at `00:00:00.000` UTC
 * - **yearly** : January 1st of the following year at `00:00:00.000` UTC
 *
 * @param timestampMs - Reference timestamp in milliseconds
 * @param granularity - Bucket granularity (`monthly` or `yearly`)
 * @returns The UTC timestamp (in milliseconds) representing the exclusive end
 * of the corresponding time bucket.
 */
function getBucketEndUtc(timestampMs: number, granularity: 'monthly' | 'yearly'): number {
  const date = new Date(timestampMs)
  if (granularity === 'yearly') return Date.UTC(date.getUTCFullYear() + 1, 0, 1, 0, 0, 0, 0)
  return Date.UTC(date.getUTCFullYear(), date.getUTCMonth() + 1, 1, 0, 0, 0, 0)
}
/**
 * Computes the completion ratio of a time bucket relative to a reference time.
 *
 * The ratio represents how much of the bucket’s duration has elapsed at
 * `referenceMs`, expressed as a normalized value in the range `[0, 1]`.
 *
 * The bucket is defined by the calendar period (monthly or yearly) that
 * contains `bucketTimestampMs`, using UTC boundaries:
 * - start: `getBucketStartUtc(...)`
 * - end: `getBucketEndUtc(...)`
 *
 * The returned value is clamped to `[0, 1]`:
 * - `0`: reference time is at or before the start of the bucket
 * - `1`: reference time is at or after the end of the bucket
 *
 * This function is used to detect partially completed periods and to
 * extrapolate full period values from partial data.
 *
 * @param params.bucketTimestampMs - Timestamp belonging to the bucket
 * @param params.granularity - Bucket granularity (`monthly` or `yearly`)
 * @param params.referenceMs - Reference timestamp used to measure progress
 * @returns A normalized completion ratio in the range `[0, 1]`.
 */
function getCompletionRatioForBucket(params: {
  bucketTimestampMs: number
  granularity: 'monthly' | 'yearly'
  referenceMs: number
}): number {
  const start = getBucketStartUtc(params.bucketTimestampMs, params.granularity)
  const end = getBucketEndUtc(params.bucketTimestampMs, params.granularity)
  const total = end - start
  if (total <= 0) return 1
  return clampRatio((params.referenceMs - start) / total)
}
/**
 * Extrapolate the last observed value of a time series when the last bucket
 * (month or year) is only partially complete.
 *
 * This is used to replace the final value in each `VueUiXy` series
 * before rendering, so the chart can display an estimated full-period value
 * for the current month or year.
 *
 * Notes:
 * - This function assumes `lastValue` is the value corresponding to the last
 *   date in `chartData.value.dates`
 *
 * @param lastValue - The last observed numeric value for a series.
 * @returns The extrapolated value for partially completed monthly or yearly granularities,
 * or the original `lastValue` when no extrapolation should be applied.
 */
function extrapolateLastValue(lastValue: number) {
  if (selectedMetric.value === 'contributors') return lastValue
  if (displayedGranularity.value !== 'monthly' && displayedGranularity.value !== 'yearly')
    return lastValue
  const endDateMs = endDate.value ? endDateOnlyToUtcMs(endDate.value) : null
  const referenceMs = endDateMs ?? Date.now()
  const completionRatio = getCompletionRatioForBucket({
    bucketTimestampMs: chartData.value.dates.at(-1) ?? 0,
    granularity: displayedGranularity.value,
    referenceMs,
  })
  if (!(completionRatio > 0 && completionRatio < 1)) return lastValue
  const extrapolatedValue = lastValue / completionRatio
  if (!Number.isFinite(extrapolatedValue)) return lastValue
  return extrapolatedValue
}
/**
 * Build and return svg markup for estimation overlays on the chart.
 *
 * This function is used in the `#svg` slot of `VueUiXy` to draw a dashed line
 * between the last datapoint and its ancestor, for partial month or year.
 *
 * The function returns an empty string when:
 * - estimation overlays are disabled
 * - no valid series or datapoints are available
 *
 * @param svg - svg context object provided by `VueUiXy` via the `#svg` slot
 * @returns A string containing SVG elements to be injected, or an empty string
 * when no estimation overlay should be rendered.
 */
function drawEstimationLine(svg: Record<string, any>) {
  if (!shouldRenderEstimationOverlay.value) return ''
  const data = Array.isArray(svg?.data) ? svg.data : []
  if (!data.length) return ''
  // Collect per-series estimates and a global max candidate for the y-axis
  const lines: string[] = []
  for (const serie of data) {
    const plots = serie?.plots
    if (!Array.isArray(plots) || plots.length < 2) continue
    const previousPoint = plots.at(-2)
    const lastPoint = plots.at(-1)
    if (!previousPoint || !lastPoint) continue
    const stroke = String(serie?.color ?? colors.value.fg)
    /**
     * The following svg elements are injected in the #svg slot of VueUiXy:
     * - a line overlay covering the plain path bewteen the last datapoint and its ancestor
     * - a dashed line connecting the last datapoint to its ancestor
     * - a circle for the last datapoint
     */
    lines.push(`
      <line
        x1="${previousPoint.x}"
        y1="${previousPoint.y}"
        x2="${lastPoint.x}"
        y2="${lastPoint.y}"
        stroke="${colors.value.bg}"
        stroke-width="3"
        opacity="1"
      />
      <line
        x1="${previousPoint.x}"
        y1="${previousPoint.y}"
        x2="${lastPoint.x}"
        y2="${lastPoint.y}"
        stroke="${stroke}"
        stroke-width="3"
        stroke-dasharray="4 8"
        stroke-linecap="round"
      />
      <circle
        cx="${lastPoint.x}"
        cy="${lastPoint.y}"
        r="4"
        fill="${stroke}"
        stroke="${colors.value.bg}"
        stroke-width="2"
      />
    `)
  }
  if (!lines.length) return ''
  return lines.join('\n')
}
/**
 * Build and return svg text label for the last datapoint of each series.
 *
 * This function is used in the `#svg` slot of `VueUiXy` to render a value label
 * next to the final datapoint of each series when the data represents fully
 * completed periods (for example, daily or weekly granularities).
 *
 * For each series:
 * - retrieves the last plotted point
 * - renders a text label slightly offset to the right of the point
 * - formats the value using the compact number formatter
 *
 * Return an empty string when no series data is available.
 *
 * @param svg - SVG context object provided by `VueUiXy` via the `#svg` slot
 * @returns A string containing SVG `<text>` elements, or an empty string when
 * no labels should be rendered.
 */
function drawLastDatapointLabel(svg: Record<string, any>) {
  const data = Array.isArray(svg?.data) ? svg.data : []
  if (!data.length) return ''
  const dataLabels: string[] = []
  for (const serie of data) {
    const lastPlot = serie.plots.at(-1)
    dataLabels.push(`
      <text
        text-anchor="start"
        dominant-baseline="middle"
        x="${lastPlot.x + 12}"
        y="${lastPlot.y}"
        font-size="24"
        fill="${colors.value.fg}"
        stroke="${colors.value.bg}"
        stroke-width="1"
        paint-order="stroke fill"
      >
        ${compactNumberFormatter.value.format(Number.isFinite(lastPlot.value) ? lastPlot.value : 0)}
      </text>
    `)
  }
  return dataLabels.join('\n')
}
/**
 * Build and return a legend to be injected during the SVG export only, since the custom legend is
 * displayed as an independant div, content has to be injected within the chart's viewBox.
 *
 * Legend items are displayed in a column, on the top left of the chart.
 */
function drawSvgPrintLegend(svg: Record<string, any>) {
  const data = Array.isArray(svg?.data) ? svg.data : []
  if (!data.length) return ''
  const seriesNames: string[] = []
  data.forEach((serie, index) => {
    seriesNames.push(`
      <rect
        x="${svg.drawingArea.left + 12}"
        y="${svg.drawingArea.top + 24 * index - 7}"
        width="12"
        height="12"
        fill="${serie.color}"
        rx="3"
      />
      <text
        text-anchor="start"
        dominant-baseline="middle"
        x="${svg.drawingArea.left + 32}"
        y="${svg.drawingArea.top + 24 * index}"
        font-size="16"
        fill="${colors.value.fg}"
        stroke="${colors.value.bg}"
        stroke-width="1"
        paint-order="stroke fill"
      >
        ${serie.name}
      </text>
  `)
  })
  // Inject the estimation legend item when necessary
  if (
    (supportsEstimation.value && !isEndDateOnPeriodEnd.value && !isZoomed.value) ||
    hasDownloadAnomalies.value
  ) {
    seriesNames.push(`
        <line
          x1="${svg.drawingArea.left + 12}"
          y1="${svg.drawingArea.top + 24 * data.length}"
          x2="${svg.drawingArea.left + 24}"
          y2="${svg.drawingArea.top + 24 * data.length}"
          stroke="${colors.value.fg}"
          stroke-dasharray="4"
          stroke-linecap="round"
        />
        <text
          text-anchor="start"
          dominant-baseline="middle"
          x="${svg.drawingArea.left + 32}"
          y="${svg.drawingArea.top + 24 * data.length}"
          font-size="16"
          fill="${colors.value.fg}"
          stroke="${colors.value.bg}"
          stroke-width="1"
          paint-order="stroke fill"
        >
          ${$t('package.trends.legend_estimation')}
        </text>
      `)
  }
  return seriesNames.join('\n')
}
// VueUiXy chart component configuration
const chartConfig = computed<VueUiXyConfig>(() => {
  return {
    theme: isDarkMode.value ? 'dark' : ('' as VueDataUiTheme),
    chart: {
      height: isMobile.value ? 950 : 600,
      backgroundColor: colors.value.bg,
      padding: { bottom: displayedGranularity.value === 'yearly' ? 84 : 64, right: 128 }, // padding right is set to leave space of last datapoint label(s)
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
          stack: $t('package.trends.toggle_stack_mode'),
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
            copyAltTextForTrendLineChart({
              dataset: dst,
              config: {
                ...cfg,
                formattedDatasetValues: (dst?.lines || []).map(d =>
                  d.series.map(n => compactNumberFormatter.value.format(n ?? 0)),
                ),
                hasEstimation:
                  supportsEstimation.value && !isEndDateOnPeriodEnd.value && !isZoomed.value,
                granularity: displayedGranularity.value,
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
            yLabel: $t('package.trends.y_axis_label', {
              granularity: getGranularityLabel(selectedGranularity.value),
              facet: activeMetricDef.value?.label,
            }),
            yLabelOffsetX: 12,
            fontSize: isMobile.value ? 32 : 24,
          },
          xAxisLabels: {
            show: true,
            showOnlyAtModulo: true,
            modulo: 12,
            values: chartData.value?.dates,
            datetimeFormatter: {
              enable: true,
              locale: locale.value,
              useUTC: true,
              options: datetimeFormatterOptions.value,
            },
          },
          yAxis: {
            formatter: ({ value }: { value: number }) => {
              return compactNumberFormatter.value.format(Number.isFinite(value) ? value : 0)
            },
            useNiceScale: true, // daily/weekly -> true, monthly/yearly -> false
            gap: 24, // vertical gap between individual series in stacked mode
          },
        },
      },
      timeTag: {
        show: true,
        backgroundColor: colors.value.bgElevated,
        color: colors.value.fg,
        fontSize: 16,
        circleMarker: { radius: 3, color: colors.value.border },
        useDefaultFormat: true,
        timeFormat: 'yyyy-MM-dd HH:mm:ss',
      },
      highlighter: { useLine: true },
      legend: { show: false, position: 'top' },
      tooltip: {
        teleportTo: props.inModal ? '#chart-modal' : undefined,
        borderColor: 'transparent',
        backdropFilter: false,
        backgroundColor: 'transparent',
        customFormat: ({ datapoint: items }) => {
          if (!items || pending.value) return ''

          const hasMultipleItems = items.length > 1

          const rows = items
            .map((d: Record<string, any>) => {
              const label = String(d?.name ?? '').trim()
              const raw = Number(d?.value ?? 0)
              const v = compactNumberFormatter.value.format(Number.isFinite(raw) ? raw : 0)

              if (!hasMultipleItems) {
                // We don't need the name of the package in this case, since it is shown in the xAxis label
                return `<div>
                  <span class="text-base text-[var(--fg)] font-mono tabular-nums">${v}</span>
                </div>`
              }

              return `<div class="grid grid-cols-[12px_minmax(0,1fr)_max-content] items-center gap-x-3">
                <div class="w-3 h-3">
                  <svg viewBox="0 0 2 2" class="w-full h-full">
                    <rect x="0" y="0" width="2" height="2" rx="0.3" fill="${d.color}" />
                  </svg>
                </div>

                <span class="text-3xs uppercase tracking-wide text-[var(--fg)]/70 truncate">
                  ${label}
                </span>

                <span class="text-base text-[var(--fg)] font-mono tabular-nums text-end">
                  ${v}
                </span>
              </div>`
            })
            .join('')

          return `<div class="font-mono text-xs p-3 border border-border rounded-md bg-[var(--bg)]/10 backdrop-blur-md">
            <div class="${hasMultipleItems ? 'flex flex-col gap-2' : ''}">
              ${rows}
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
const isDownloadsMetric = computed(() => selectedMetric.value === 'downloads')
const showCorrectionControls = shallowRef(false)
const packageAnomalies = computed(() => getAnomaliesForPackages(effectivePackageNames.value))
const hasAnomalies = computed(() => packageAnomalies.value.length > 0)
function formatAnomalyDate(dateStr: string) {
  const [y, m, d] = dateStr.split('-').map(Number)
  if (!y || !m || !d) return dateStr
  return new Intl.DateTimeFormat(locale.value, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    timeZone: 'UTC',
  }).format(new Date(Date.UTC(y, m - 1, d)))
}
// Trigger data loading when the metric is switched
watch(selectedMetric, value => {
  if (!isMounted.value) return
  loadMetric(value)
})

return (_ctx: any,_cache: any) => {
  const _component_SelectField = _resolveComponent("SelectField")
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_ClientOnly = _resolveComponent("ClientOnly")

  return (_openBlock(), _createElementBlock("div", {
      class: "w-full relative",
      id: "trends-chart",
      "aria-busy": activeMetricState.value.pending ? 'true' : 'false'
    }, [ _createElementVNode("div", { class: "w-full mb-4 flex flex-col gap-3" }, [ _createElementVNode("div", { class: "flex flex-col sm:flex-row gap-3 sm:gap-2 sm:items-end" }, [ (__props.showFacetSelector) ? (_openBlock(), _createBlock(_component_SelectField, {
              key: 0,
              id: "trends-metric-select",
              disabled: activeMetricState.value.pending,
              items: METRICS.value.map((m) => ({
  	label: m.label,
  	value: m.id
  })),
              label: _ctx.$t('package.trends.facet'),
              modelValue: _unref(selectedMetric),
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((selectedMetric).value = $event))
            }, null, 8 /* PROPS */, ["disabled", "items", "label", "modelValue"])) : _createCommentVNode("v-if", true), _createVNode(_component_SelectField, {
            label: _ctx.$t('package.trends.granularity'),
            id: "granularity",
            disabled: activeMetricState.value.pending,
            items: granularityItems.value,
            modelValue: _unref(selectedGranularity),
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((selectedGranularity).value = $event))
          }, null, 8 /* PROPS */, ["label", "disabled", "items", "modelValue"]), _createElementVNode("div", { class: "grid grid-cols-2 gap-2 flex-1" }, [ _createElementVNode("div", { class: "flex flex-col gap-1" }, [ _createElementVNode("label", _hoisted_1, _toDisplayString(_ctx.$t('package.trends.start_date')), 1 /* TEXT */), _createElementVNode("div", { class: "relative flex items-center" }, [ _hoisted_2, _createVNode(_component_InputBase, {
                  id: "startDate",
                  type: "date",
                  max: _unref(DATE_INPUT_MAX),
                  class: "w-full min-w-0 bg-transparent ps-7",
                  size: "medium",
                  modelValue: _unref(startDate),
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((startDate).value = $event))
                }, null, 8 /* PROPS */, ["max", "modelValue"]) ]) ]), _createElementVNode("div", { class: "flex flex-col gap-1" }, [ _createElementVNode("label", _hoisted_3, _toDisplayString(_ctx.$t('package.trends.end_date')), 1 /* TEXT */), _createElementVNode("div", { class: "relative flex items-center" }, [ _hoisted_4, _createVNode(_component_InputBase, {
                  id: "endDate",
                  type: "date",
                  max: _unref(DATE_INPUT_MAX),
                  class: "w-full min-w-0 bg-transparent ps-7",
                  size: "medium",
                  modelValue: _unref(endDate),
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((endDate).value = $event))
                }, null, 8 /* PROPS */, ["max", "modelValue"]) ]) ]) ]), (showResetButton.value) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              type: "button",
              "aria-label": "Reset date range",
              class: "self-end flex items-center justify-center px-2.5 py-2.25 border border-transparent rounded-md text-fg-subtle hover:text-fg transition-colors hover:border-border focus-visible:outline-accent/70 sm:mb-0",
              onClick: resetDateRange
            }, [ _hoisted_5 ])) : _createCommentVNode("v-if", true) ]), _createTextVNode("\n\n      " + "\n      "), (isDownloadsMetric.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "flex flex-col gap-2"
          }, [ _createElementVNode("button", {
              type: "button",
              class: "self-start flex items-center gap-1 text-2xs font-mono text-fg-subtle hover:text-fg transition-colors",
              onClick: _cache[4] || (_cache[4] = ($event: any) => (showCorrectionControls.value = !showCorrectionControls.value))
            }, [ _createElementVNode("span", {
                class: _normalizeClass(["w-3.5 h-3.5 transition-transform", showCorrectionControls.value ? 'i-lucide:chevron-down' : 'i-lucide:chevron-right']),
                "aria-hidden": "true"
              }, null, 2 /* CLASS */), _createTextVNode("\n          " + _toDisplayString(_ctx.$t('package.trends.data_correction')), 1 /* TEXT */) ]), (showCorrectionControls.value) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: "flex items-end gap-3"
              }, [ _createElementVNode("label", { class: "flex flex-col gap-1 flex-1" }, [ _createElementVNode("span", { class: "text-2xs font-mono text-fg-subtle tracking-wide uppercase" }, [ _createTextVNode(_toDisplayString(_ctx.$t('package.trends.average_window')) + "\n              ", 1 /* TEXT */), _createElementVNode("span", _hoisted_6, "(" + _toDisplayString(_unref(settings).chartFilter.averageWindow) + ")", 1 /* TEXT */) ]), _withDirectives(_createElementVNode("input", {
                    "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((_unref(settings).chartFilter.averageWindow) = $event)),
                    type: "range",
                    min: "0",
                    max: "20",
                    step: "1",
                    class: "accent-[var(--accent-color,var(--fg-subtle))]"
                  }, null, 512 /* NEED_PATCH */), [ [ _vModelText, _unref(settings).chartFilter.averageWindow, void 0, { number: true } ] ]) ]), _createElementVNode("label", { class: "flex flex-col gap-1 flex-1" }, [ _createElementVNode("span", { class: "text-2xs font-mono text-fg-subtle tracking-wide uppercase" }, [ _createTextVNode(_toDisplayString(_ctx.$t('package.trends.smoothing')) + "\n              ", 1 /* TEXT */), _createElementVNode("span", _hoisted_7, "(" + _toDisplayString(_unref(settings).chartFilter.smoothingTau) + ")", 1 /* TEXT */) ]), _withDirectives(_createElementVNode("input", {
                    "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((_unref(settings).chartFilter.smoothingTau) = $event)),
                    type: "range",
                    min: "0",
                    max: "20",
                    step: "1",
                    class: "accent-[var(--accent-color,var(--fg-subtle))]"
                  }, null, 512 /* NEED_PATCH */), [ [ _vModelText, _unref(settings).chartFilter.smoothingTau, void 0, { number: true } ] ]) ]), _createElementVNode("div", { class: "flex flex-col gap-1 shrink-0" }, [ _createElementVNode("span", { class: "text-2xs font-mono text-fg-subtle tracking-wide uppercase flex items-center justify-between" }, [ _createTextVNode(_toDisplayString(_ctx.$t('package.trends.known_anomalies')) + "\n              ", 1 /* TEXT */), _createVNode(_component_TooltipApp, {
                      interactive: "",
                      to: __props.inModal ? '#chart-modal' : undefined
                    }, {
                      content: _withCtx(() => [
                        _createElementVNode("div", { class: "flex flex-col gap-3" }, [
                          _createElementVNode("p", _hoisted_8, _toDisplayString(_ctx.$t('package.trends.known_anomalies_description')), 1 /* TEXT */),
                          (hasAnomalies.value)
                            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                              _createElementVNode("p", _hoisted_9, _toDisplayString(_ctx.$t('package.trends.known_anomalies_ranges')), 1 /* TEXT */),
                              _createElementVNode("ul", { class: "text-xs text-fg-subtle list-disc list-inside" }, [
                                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(packageAnomalies.value, (a) => {
                                  return (_openBlock(), _createElementBlock("li", { key: `${a.packageName}-${a.start}` }, _toDisplayString(isMultiPackageMode.value
                                ? _ctx.$t('package.trends.known_anomalies_range_named', {
                                    packageName: a.packageName,
                                    start: formatAnomalyDate(a.start),
                                    end: formatAnomalyDate(a.end),
                                  })
                                : _ctx.$t('package.trends.known_anomalies_range', {
                                    start: formatAnomalyDate(a.start),
                                    end: formatAnomalyDate(a.end),
                                  })), 1 /* TEXT */))
                                }), 128 /* KEYED_FRAGMENT */))
                              ])
                            ]))
                            : (_openBlock(), _createElementBlock("p", {
                              key: 1,
                              class: "text-xs text-fg-muted"
                            }, _toDisplayString(_ctx.$t('package.trends.known_anomalies_none', effectivePackageNames.value.length)), 1 /* TEXT */)),
                          _createElementVNode("div", { class: "flex justify-end" }, [
                            _createVNode(_component_LinkBase, {
                              to: "https://github.com/npmx-dev/npmx.dev/edit/main/app/utils/download-anomalies.data.ts",
                              class: "text-xs text-accent"
                            }, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_ctx.$t('package.trends.known_anomalies_contribute')), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ])
                        ])
                      ]),
                      default: _withCtx(() => [
                        _createElementVNode("button", {
                          type: "button",
                          class: "i-lucide:info w-3.5 h-3.5 text-fg-muted cursor-help",
                          "aria-label": _ctx.$t('package.trends.known_anomalies')
                        }, null, 8 /* PROPS */, ["aria-label"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]) ]), _createElementVNode("label", {
                    class: _normalizeClass(["flex items-center gap-1.5 text-2xs font-mono text-fg-subtle cursor-pointer", { 'opacity-50 pointer-events-none': !hasAnomalies.value }])
                  }, [ _withDirectives(_createElementVNode("input", {
                      "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((_unref(settings).chartFilter.anomaliesFixed) = $event)),
                      type: "checkbox",
                      disabled: !hasAnomalies.value,
                      class: "accent-[var(--accent-color,var(--fg-subtle))]"
                    }, null, 8 /* PROPS */, ["disabled"]), [ [_vModelCheckbox, _unref(settings).chartFilter.anomaliesFixed] ]), _createTextVNode("\n              " + _toDisplayString(_ctx.$t('package.trends.apply_correction')), 1 /* TEXT */) ], 2 /* CLASS */) ]) ])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), (skippedPackagesWithoutGitHub.value.length > 0) ? (_openBlock(), _createElementBlock("p", {
            key: 0,
            class: "text-2xs font-mono text-fg-subtle"
          }, _toDisplayString(_ctx.$t('package.trends.contributors_skip', { count: skippedPackagesWithoutGitHub.value.length })) + "\n        " + _toDisplayString(skippedPackagesWithoutGitHub.value.join(', ')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("h2", _hoisted_10, _toDisplayString(_ctx.$t('package.trends.title')) + " — " + _toDisplayString(activeMetricDef.value?.label), 1 /* TEXT */), _createTextVNode("\n\n    " + "\n    "), _createElementVNode("div", {
        role: "region",
        "aria-labelledby": "trends-chart-title",
        class: _normalizeClass(isMobile.value === false && _unref(width) > 0 ? 'min-h-[567px]' : 'min-h-[260px]')
      }, [ (chartData.value.dataset) ? (_openBlock(), _createBlock(_component_ClientOnly, { key: 0 }, {
            fallback: _withCtx(() => [
              _hoisted_20
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", {
                "data-pending": pending.value,
                "data-minimap-visible": maxDatapoints.value > 6
              }, [
                _createVNode(VueUiXy, {
                  dataset: normalisedDataset.value,
                  config: chartConfig.value,
                  class: "[direction:ltr]",
                  onZoomStart: setIsZoom,
                  onZoomEnd: setIsZoom,
                  onZoomReset: _cache[8] || (_cache[8] = ($event: any) => (isZoomed.value = false))
                }, {
                  svg: _withCtx(({ svg }) => [
                    (shouldRenderEstimationOverlay.value && !isEndDateOnPeriodEnd.value && !isZoomed.value)
                      ? (_openBlock(), _createElementBlock("g", {
                        key: 0,
                        innerHTML: drawEstimationLine(svg)
                      }))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n\n              "),
                    _createTextVNode("\n              "),
                    (!pending.value)
                      ? (_openBlock(), _createElementBlock("g", {
                        key: 0,
                        innerHTML: drawLastDatapointLabel(svg)
                      }))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n\n              "),
                    _createTextVNode("\n              "),
                    (svg.isPrintingSvg)
                      ? (_openBlock(), _createElementBlock("g", {
                        key: 0,
                        innerHTML: drawSvgPrintLegend(svg)
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
                    (pending.value)
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
                  "area-gradient": _withCtx(({ series: chartModalSeries, id: gradientId }) => [
                    _createElementVNode("linearGradient", {
                      id: gradientId,
                      x1: "0",
                      x2: "0",
                      y1: "0",
                      y2: "1"
                    }, [
                      _createElementVNode("stop", {
                        offset: "0%",
                        "stop-color": chartModalSeries.color,
                        "stop-opacity": "0.2"
                      }, null, 8 /* PROPS */, ["stop-color"]),
                      _createElementVNode("stop", {
                        offset: "100%",
                        "stop-color": _unref(colors).bg,
                        "stop-opacity": "0"
                      }, null, 8 /* PROPS */, ["stop-color"])
                    ], 8 /* PROPS */, ["id"])
                  ]),
                  legend: _withCtx(({ legend }) => [
                    _createElementVNode("div", { class: "flex gap-4 flex-wrap justify-center" }, [
                      (isMultiPackageMode.value)
                        ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(legend, (datapoint) => {
                            return (_openBlock(), _createElementBlock("button", {
                              key: datapoint.name,
                              "aria-pressed": datapoint.isSegregated,
                              "aria-label": datapoint.name,
                              type: "button",
                              class: "flex gap-1 place-items-center",
                              onClick: _cache[9] || (_cache[9] = ($event: any) => (datapoint.segregate()))
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
                                    fill: datapoint.color
                                  }, null, 8 /* PROPS */, ["fill"])
                                ])
                              ]),
                              _createElementVNode("span", {
                                style: _normalizeStyle({
                          textDecoration: datapoint.isSegregated ? 'line-through' : undefined,
                        })
                              }, _toDisplayString(datapoint.name), 5 /* TEXT, STYLE */)
                            ], 8 /* PROPS */, ["aria-pressed", "aria-label"]))
                          }), 128 /* KEYED_FRAGMENT */))
                        ], 64 /* STABLE_FRAGMENT */))
                        : (legend.length > 0)
                          ? (_openBlock(), _createElementBlock("div", {
                            key: 1,
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
                        : _createCommentVNode("v-if", true),
                      _createTextVNode("\n\n                " + "\n                " + "\n\n                " + "\n                "),
                      (supportsEstimation.value || hasDownloadAnomalies.value)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: "flex gap-1 place-items-center"
                        }, [
                          _createElementVNode("svg", {
                            viewBox: "0 0 20 2",
                            width: "20"
                          }, [
                            _createElementVNode("line", {
                              x1: "0",
                              y1: "1",
                              x2: "20",
                              y2: "1",
                              stroke: _unref(colors).fg,
                              "stroke-dasharray": "4",
                              "stroke-linecap": "round"
                            }, null, 8 /* PROPS */, ["stroke"])
                          ]),
                          _createElementVNode("span", _hoisted_11, _toDisplayString(_ctx.$t('package.trends.legend_estimation')), 1 /* TEXT */)
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
                      onClick: _cache[10] || (_cache[10] = (...args) => (_ctx.resetMinimap && _ctx.resetMinimap(...args)))
                    }, [
                      _hoisted_12
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
                    _hoisted_13
                  ]),
                  optionImg: _withCtx(() => [
                    _hoisted_14
                  ]),
                  optionSvg: _withCtx(() => [
                    _hoisted_15
                  ]),
                  "annotator-action-close": _withCtx(() => [
                    _hoisted_16
                  ]),
                  "annotator-action-color": _withCtx(({ color }) => [
                    _createElementVNode("span", {
                      class: "i-lucide:palette w-6 h-6",
                      style: { color: color },
                      "aria-hidden": "true"
                    })
                  ]),
                  "annotator-action-undo": _withCtx(() => [
                    _hoisted_17
                  ]),
                  "annotator-action-redo": _withCtx(() => [
                    _hoisted_18
                  ]),
                  "annotator-action-delete": _withCtx(() => [
                    _hoisted_19
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
                    _createTextVNode("\n            ")
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["dataset", "config"])
              ], 8 /* PROPS */, ["data-pending", "data-minimap-visible"])
            ]),
            _: 1 /* STABLE */
          })) : _createCommentVNode("v-if", true), (!chartData.value.dataset && !activeMetricState.value.pending) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "min-h-[260px] flex items-center justify-center text-fg-subtle font-mono text-sm"
          }, _toDisplayString(_ctx.$t('package.trends.no_data')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 2 /* CLASS */), (activeMetricState.value.pending) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          role: "status",
          "aria-live": "polite",
          class: "absolute top-1/2 inset-is-1/2 -translate-x-1/2 -translate-y-1/2 text-xs text-fg-subtle font-mono bg-bg/70 backdrop-blur px-3 py-2 rounded-md border border-border"
        }, _toDisplayString(_ctx.$t('package.trends.loading')), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 8 /* PROPS */, ["aria-busy"]))
}
}

})
