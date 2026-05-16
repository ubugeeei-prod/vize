import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = { id: "interface-tc", "font-medium": "true" }
import type { ThemeColors } from '~/composables/settings'
import { THEME_COLORS } from '~/constants'

export default /*@__PURE__*/_defineComponent({
  __name: 'SettingsThemeColors',
  async setup(__props) {

let __temp: any, __restore: any

const themes = await import('~/constants/themes.json').then((r) => {
  const map = new Map<'dark' | 'light', [string, ThemeColors][]>([['dark', []], ['light', []]])
  const themes = r.default as [string, ThemeColors][]
  for (const [key, theme] of themes) {
    map.get('dark')!.push([key, theme])
    map.get('light')!.push([key, {
      ...theme,
      '--c-primary': `color-mix(in srgb, ${theme['--c-primary']}, black 25%)`,
    }])
  }
  return map
})
const settings = useUserSettings()
const media = useMediaQuery('(prefers-color-scheme: dark)')
const colorMode = useColorMode()
const useThemes = shallowRef<[string, ThemeColors][]>([])
watch(() => colorMode.preference, (cm) => {
  const dark = cm === 'dark' || (cm === 'system' && media.value)
  const newThemes = dark ? themes.get('dark')! : themes.get('light')!
  const key = settings.value.themeColors?.['--theme-color-name'] || THEME_COLORS.defaultTheme
  for (const [k, theme] of newThemes) {
    if (k === key) {
      settings.value.themeColors = theme
      break
    }
  }
  useThemes.value = newThemes
}, { immediate: true, flush: 'post' })
const currentTheme = computed(() => settings.value.themeColors?.['--theme-color-name'] || THEME_COLORS.defaultTheme)
function updateTheme(theme: ThemeColors) {
  settings.value.themeColors = theme
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("section", { "space-y-2": "" }, [ _createElementVNode("h2", _hoisted_1, _toDisplayString(_ctx.$t('settings.interface.theme_color')), 1 /* TEXT */), _createElementVNode("div", {
        flex: "~ gap4 wrap",
        p2: "",
        role: "group",
        "aria-labelledby": "interface-tc"
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(useThemes.value, ([key, theme]) => {
          return (_openBlock(), _createElementBlock("button", {
            key: key,
            style: {
            '--rgb-primary': theme['--rgb-primary'],
            'background': theme['--c-primary'],
            '--local-ring-color': theme['--c-primary'],
          },
            type: "button",
            class: _normalizeClass(currentTheme.value === theme['--theme-color-name'] ? 'ring-2' : 'scale-90'),
            "aria-pressed": currentTheme.value === theme['--theme-color-name'] ? 'true' : 'false',
            title: theme['--theme-color-name'],
            "w-8": "",
            "h-8": "",
            "rounded-full": "",
            "transition-all": "",
            ring: "$local-ring-color offset-3 offset-$c-bg-base",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (updateTheme(_ctx.theme)))
          }, 10 /* CLASS, PROPS */, ["aria-pressed", "title"]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
