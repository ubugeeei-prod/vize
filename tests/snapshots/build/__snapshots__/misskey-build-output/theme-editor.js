import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
import { watch, ref, computed } from 'vue'
import { toUnicode } from 'punycode.js'
import tinycolor from 'tinycolor2'
import JSON5 from 'json5'
import lightTheme from '@@/themes/_light.json5'
import darkTheme from '@@/themes/_dark.json5'
import { host } from '@@/js/config.js'
import type { Theme } from '@/theme.js'
import { genId } from '@/utility/id.js'
import MkButton from '@/components/MkButton.vue'
import MkCodeEditor from '@/components/MkCodeEditor.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkFolder from '@/components/MkFolder.vue'
import { ensureSignin } from '@/i.js'
import { addTheme, applyTheme } from '@/theme.js'
import * as os from '@/os.js'
import { store } from '@/store.js'
import { i18n } from '@/i18n.js'
import { useLeaveGuard } from '@/composables/use-leave-guard.js'
import { definePage } from '@/page.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'theme-editor',
  setup(__props) {

const $i = ensureSignin();
const bgColors = [
	{ color: '#f5f5f5', kind: 'light', forPreview: '#f5f5f5' },
	{ color: '#f0eee9', kind: 'light', forPreview: '#f3e2b9' },
	{ color: '#e9eff0', kind: 'light', forPreview: '#bfe3e8' },
	{ color: '#f0e9ee', kind: 'light', forPreview: '#f1d1e8' },
	{ color: '#dce2e0', kind: 'light', forPreview: '#a4dccc' },
	{ color: '#e2e0dc', kind: 'light', forPreview: '#d8c7a5' },
	{ color: '#d5dbe0', kind: 'light', forPreview: '#b0cae0' },
	{ color: '#dad5d5', kind: 'light', forPreview: '#d6afaf' },
	{ color: '#2b2b2b', kind: 'dark', forPreview: '#444444' },
	{ color: '#362e29', kind: 'dark', forPreview: '#735c4d' },
	{ color: '#303629', kind: 'dark', forPreview: '#506d2f' },
	{ color: '#293436', kind: 'dark', forPreview: '#258192' },
	{ color: '#2e2936', kind: 'dark', forPreview: '#504069' },
	{ color: '#252722', kind: 'dark', forPreview: '#3c462f' },
	{ color: '#212525', kind: 'dark', forPreview: '#303e3e' },
	{ color: '#191919', kind: 'dark', forPreview: '#272727' },
] as const;
const accentColors = ['#e36749', '#f29924', '#98c934', '#34c9a9', '#34a1c9', '#606df7', '#8d34c9', '#e84d83'];
const fgColors = [
	{ color: 'none', forLight: '#5f5f5f', forDark: '#dadada', forPreview: null },
	{ color: 'red', forLight: '#7f6666', forDark: '#e4d1d1', forPreview: '#ca4343' },
	{ color: 'yellow', forLight: '#736955', forDark: '#e0d5c0', forPreview: '#d49923' },
	{ color: 'green', forLight: '#586d5b', forDark: '#d1e4d4', forPreview: '#4cbd5c' },
	{ color: 'cyan', forLight: '#5d7475', forDark: '#d1e3e4', forPreview: '#2abdc3' },
	{ color: 'blue', forLight: '#676880', forDark: '#d1d2e4', forPreview: '#7275d8' },
	{ color: 'pink', forLight: '#84667d', forDark: '#e4d1e0', forPreview: '#b12390' },
];
const theme = ref<Theme>({
	id: genId(),
	name: 'untitled',
	author: `@${$i.username}@${toUnicode(host)}`,
	base: 'light',
	props: lightTheme.props,
});
const description = ref<string | null>(null);
const themeCode = ref<string>('');
const changed = ref(false);
useLeaveGuard(changed);
function showPreview() {
	os.pageWindow('/preview');
}
function setBgColor(color: typeof bgColors[number]) {
	if (theme.value.base !== color.kind) {
		const base = color.kind === 'dark' ? darkTheme : lightTheme;
		for (const prop of Object.keys(base.props)) {
			if (prop === 'accent') continue;
			if (prop === 'fg') continue;
			theme.value.props[prop] = base.props[prop];
		}
	}
	theme.value.base = color.kind;
	theme.value.props.bg = color.color;
	if (theme.value.props.fg) {
		const matchedFgColor = fgColors.find(x => [tinycolor(x.forLight).toRgbString(), tinycolor(x.forDark).toRgbString()].includes(tinycolor(theme.value.props.fg).toRgbString()));
		if (matchedFgColor) setFgColor(matchedFgColor);
	}
}
function setAccentColor(color: string) {
	theme.value.props.accent = color;
}
function setFgColor(color: typeof fgColors[number]) {
	theme.value.props.fg = theme.value.base === 'light' ? color.forLight : color.forDark;
}
function apply() {
	themeCode.value = JSON5.stringify(theme.value, null, '\t');
	applyTheme(theme.value, false);
	changed.value = true;
}
function applyThemeCode() {
	let parsed;
	try {
		parsed = JSON5.parse(themeCode.value);
	} catch (err) {
		os.alert({
			type: 'error',
			text: i18n.ts._theme.invalid,
		});
		return;
	}
	theme.value = parsed;
}
async function saveAs() {
	const { canceled, result: name } = await os.inputText({
		title: i18n.ts.name,
		minLength: 1,
	});
	if (canceled) return;
	theme.value.id = genId();
	theme.value.name = name;
	if (description.value) theme.value.desc = description.value;
	await addTheme(theme.value);
	applyTheme(theme.value);
	if (store.s.darkMode) {
		prefer.commit('darkTheme', theme.value);
	} else {
		prefer.commit('lightTheme', theme.value);
	}
	changed.value = false;
	os.alert({
		type: 'success',
		text: i18n.tsx._theme.installed({ name: theme.value.name }),
	});
}
watch(theme, apply, { deep: true });
const headerActions = computed(() => [{
	asFullButton: true,
	icon: 'ti ti-eye',
	text: i18n.ts.preview,
	handler: showPreview,
}, {
	asFullButton: true,
	icon: 'ti ti-check',
	text: i18n.ts.saveAs,
	handler: saveAs,
}]);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.themeEditor,
	icon: 'ti ti-palette',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          _createElementVNode("div", { class: "cwepdizn _gaps_m" }, [
            _createVNode(MkFolder, { defaultOpen: true }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.backgroundColor), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", { class: "cwepdizn-colors" }, [
                  _createElementVNode("div", { class: "row" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(bgColors).filter(x => x.kind === 'light'), (color) => {
                      return (_openBlock(), _createElementBlock("button", { class: _normalizeClass(["color _button", { active: theme.value.props.bg === color.color }]), onClick: _cache[0] || (_cache[0] = ($event: any) => (setBgColor(color))) }, [
                        _createElementVNode("div", {
                          class: "preview",
                          style: _normalizeStyle({ background: color.forPreview })
                        }, null, 4 /* STYLE */)
                      ], 2 /* CLASS */))
                    }), 256 /* UNKEYED_FRAGMENT */))
                  ]),
                  _createElementVNode("div", { class: "row" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(bgColors).filter(x => x.kind === 'dark'), (color) => {
                      return (_openBlock(), _createElementBlock("button", { class: _normalizeClass(["color _button", { active: theme.value.props.bg === color.color }]), onClick: _cache[1] || (_cache[1] = ($event: any) => (setBgColor(color))) }, [
                        _createElementVNode("div", {
                          class: "preview",
                          style: _normalizeStyle({ background: color.forPreview })
                        }, null, 4 /* STYLE */)
                      ], 2 /* CLASS */))
                    }), 256 /* UNKEYED_FRAGMENT */))
                  ])
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["defaultOpen"]),
            _createVNode(MkFolder, { defaultOpen: true }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.accentColor), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", { class: "cwepdizn-colors" }, [
                  _createElementVNode("div", { class: "row" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(accentColors, (color) => {
                      return (_openBlock(), _createElementBlock("button", { class: _normalizeClass(["color rounded _button", { active: theme.value.props.accent === color }]), onClick: _cache[2] || (_cache[2] = ($event: any) => (setAccentColor(color))) }, [
                        _createElementVNode("div", {
                          class: "preview",
                          style: _normalizeStyle({ background: color })
                        }, null, 4 /* STYLE */)
                      ], 2 /* CLASS */))
                    }), 256 /* UNKEYED_FRAGMENT */))
                  ])
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["defaultOpen"]),
            _createVNode(MkFolder, { defaultOpen: true }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.textColor), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", { class: "cwepdizn-colors" }, [
                  _createElementVNode("div", { class: "row" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(fgColors, (color) => {
                      return (_openBlock(), _createElementBlock("button", { class: _normalizeClass(["color char _button", { active: (theme.value.props.fg === color.forLight) || (theme.value.props.fg === color.forDark) }]), onClick: _cache[3] || (_cache[3] = ($event: any) => (setFgColor(color))) }, [
                        _createElementVNode("div", {
                          class: "preview",
                          style: _normalizeStyle({ color: color.forPreview ? color.forPreview : theme.value.base === 'light' ? '#5f5f5f' : '#dadada' })
                        }, "A", 4 /* STYLE */)
                      ], 2 /* CLASS */))
                    }), 256 /* UNKEYED_FRAGMENT */))
                  ])
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["defaultOpen"]),
            _createVNode(MkFolder, { defaultOpen: false }, {
              icon: _withCtx(() => [
                _hoisted_1
              ]),
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.editCode), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", { class: "_gaps_m" }, [
                  _createVNode(MkCodeEditor, {
                    lang: "json5",
                    modelValue: themeCode.value,
                    "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((themeCode).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.code), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"]),
                  _createVNode(MkButton, {
                    primary: "",
                    onClick: applyThemeCode
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.apply), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["defaultOpen"]),
            _createVNode(MkFolder, { defaultOpen: false }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.addDescription), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", { class: "_gaps_m" }, [
                  _createVNode(MkTextarea, {
                    modelValue: description.value,
                    "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((description).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.description), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["modelValue"])
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["defaultOpen"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
