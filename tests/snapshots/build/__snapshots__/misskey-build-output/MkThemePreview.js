import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "m0 0h24v24h-24z", fill: "none", stroke: "none" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("path", { d: "m5 12h-2l9-9 9 9h-2" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("path", { d: "m5 12v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-7" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("path", { d: "m9 21v-6a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v6" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("path", { d: "m0 0h24v24h-24z", fill: "none", stroke: "none" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("path", { d: "m10 5a2 2 0 1 1 4 0 7 7 0 0 1 4 6v3a4 4 0 0 0 2 3h-16a4 4 0 0 0 2-3v-3a7 7 0 0 1 4-6" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("path", { d: "m9 17v1a3 3 0 0 0 6 0v-1" })
import { ref, watch } from 'vue'
import lightTheme from '@@/themes/_light.json5'
import darkTheme from '@@/themes/_dark.json5'
import type { Theme } from '@/theme.js'
import { compile } from '@/theme.js'
import { deepClone } from '@/utility/clone.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkThemePreview',
  props: {
    theme: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const themeVariables = ref<{
	bg: string;
	panel: string;
	fg: string;
	mention: string;
	hashtag: string;
	link: string;
	divider: string;
	accent: string;
	accentedBg: string;
	navBg: string;
	pageHeaderBg: string;
	success: string;
	warn: string;
	error: string;
}>({
	bg: 'var(--MI_THEME-bg)',
	panel: 'var(--MI_THEME-panel)',
	fg: 'var(--MI_THEME-fg)',
	mention: 'var(--MI_THEME-mention)',
	hashtag: 'var(--MI_THEME-hashtag)',
	link: 'var(--MI_THEME-link)',
	divider: 'var(--MI_THEME-divider)',
	accent: 'var(--MI_THEME-accent)',
	accentedBg: 'var(--MI_THEME-accentedBg)',
	navBg: 'var(--MI_THEME-navBg)',
	pageHeaderBg: 'var(--MI_THEME-pageHeaderBg)',
	success: 'var(--MI_THEME-success)',
	warn: 'var(--MI_THEME-warn)',
	error: 'var(--MI_THEME-error)',
});
watch(() => props.theme, (theme) => {
	if (theme == null) return;
	const _theme = deepClone(theme);
	if (_theme.base != null) {
		const base = [lightTheme, darkTheme].find(x => x.id === _theme.base);
		if (base) _theme.props = Object.assign({}, base.props, _theme.props);
	}
	const compiled = compile(_theme);
	themeVariables.value = {
		bg: compiled.bg ?? 'var(--MI_THEME-bg)',
		panel: compiled.panel ?? 'var(--MI_THEME-panel)',
		fg: compiled.fg ?? 'var(--MI_THEME-fg)',
		mention: compiled.mention ?? 'var(--MI_THEME-mention)',
		hashtag: compiled.hashtag ?? 'var(--MI_THEME-hashtag)',
		link: compiled.link ?? 'var(--MI_THEME-link)',
		divider: compiled.divider ?? 'var(--MI_THEME-divider)',
		accent: compiled.accent ?? 'var(--MI_THEME-accent)',
		accentedBg: compiled.accentedBg ?? 'var(--MI_THEME-accentedBg)',
		navBg: compiled.navBg ?? 'var(--MI_THEME-navBg)',
		pageHeaderBg: compiled.pageHeaderBg ?? 'var(--MI_THEME-pageHeaderBg)',
		success: compiled.success ?? 'var(--MI_THEME-success)',
		warn: compiled.warn ?? 'var(--MI_THEME-warn)',
		error: compiled.error ?? 'var(--MI_THEME-error)',
	};
}, { immediate: true });

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("svg", { viewBox: "0 0 200 150" }, [ _createElementVNode("g", { "fill-rule": "evenodd" }, [ _createElementVNode("rect", {
          width: "200",
          height: "150",
          fill: themeVariables.value.bg
        }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("rect", {
          width: "64",
          height: "150",
          fill: themeVariables.value.navBg
        }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("rect", {
          x: "64",
          width: "136",
          height: "41",
          fill: themeVariables.value.pageHeaderBg
        }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("path", {
          transform: "scale(.26458)",
          d: "m439.77 247.19c-43.673 0-78.832 35.157-78.832 78.83v249.98h407.06v-328.81z",
          fill: themeVariables.value.panel
        }, null, 8 /* PROPS */, ["fill"]) ]), _createElementVNode("circle", {
        cx: "32",
        cy: "83",
        r: "21",
        fill: themeVariables.value.accentedBg
      }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("g", null, [ _createElementVNode("rect", {
          x: "120",
          y: "88",
          width: "40",
          height: "6",
          ry: "3",
          fill: themeVariables.value.fg
        }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("rect", {
          x: "170",
          y: "88",
          width: "20",
          height: "6",
          ry: "3",
          fill: themeVariables.value.mention
        }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("rect", {
          x: "120",
          y: "108",
          width: "20",
          height: "6",
          ry: "3",
          fill: themeVariables.value.hashtag
        }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("rect", {
          x: "150",
          y: "108",
          width: "40",
          height: "6",
          ry: "3",
          fill: themeVariables.value.fg
        }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("rect", {
          x: "120",
          y: "128",
          width: "40",
          height: "6",
          ry: "3",
          fill: themeVariables.value.fg
        }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("rect", {
          x: "170",
          y: "128",
          width: "20",
          height: "6",
          ry: "3",
          fill: themeVariables.value.link
        }, null, 8 /* PROPS */, ["fill"]) ]), _createElementVNode("path", {
        d: "m65.498 40.892h137.7",
        stroke: themeVariables.value.divider,
        "stroke-width": "0.75"
      }, null, 8 /* PROPS */, ["stroke"]), _createElementVNode("g", {
        transform: "matrix(.60823 0 0 .60823 25.45 75.755)",
        fill: "none",
        stroke: themeVariables.value.accent,
        "stroke-linecap": "round",
        "stroke-linejoin": "round",
        "stroke-width": "2"
      }, [ _hoisted_1, _hoisted_2, _hoisted_3, _hoisted_4 ], 8 /* PROPS */, ["stroke"]), _createElementVNode("g", {
        transform: "matrix(.61621 0 0 .61621 25.354 117.92)",
        fill: "none",
        stroke: themeVariables.value.fg,
        "stroke-linecap": "round",
        "stroke-linejoin": "round",
        "stroke-width": "2"
      }, [ _hoisted_5, _hoisted_6, _hoisted_7 ], 8 /* PROPS */, ["stroke"]), _createElementVNode("circle", {
        cx: "32",
        cy: "32",
        r: "16",
        fill: themeVariables.value.accent
      }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("circle", {
        cx: "140",
        cy: "20",
        r: "6",
        fill: themeVariables.value.success
      }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("circle", {
        cx: "160",
        cy: "20",
        r: "6",
        fill: themeVariables.value.warn
      }, null, 8 /* PROPS */, ["fill"]), _createElementVNode("circle", {
        cx: "180",
        cy: "20",
        r: "6",
        fill: themeVariables.value.error
      }, null, 8 /* PROPS */, ["fill"]) ]))
}
}

})
