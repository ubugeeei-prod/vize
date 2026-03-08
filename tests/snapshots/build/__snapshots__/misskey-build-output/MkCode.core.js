import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, normalizeClass as _normalizeClass, unref as _unref } from "vue"

import { computed, ref, watch } from 'vue'
import { bundledLanguagesInfo } from 'shiki/langs'
import type { BundledLanguage } from 'shiki/langs'
import { getHighlighter, getTheme } from '@/utility/code-highlighter.js'
import { store } from '@/store.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkCode.core',
  props: {
    code: { type: String, required: true },
    lang: { type: String, required: false },
    codeEditor: { type: Boolean, required: false, default: false },
    withOuterStyle: { type: Boolean, required: false, default: true }
  },
  async setup(__props: any) {

let __temp: any, __restore: any

const props = __props
const highlighter = await getHighlighter();
const darkMode = store.r.darkMode;
const codeLang = ref<BundledLanguage | 'aiscript'>('js');
const [lightThemeName, darkThemeName] = await Promise.all([
	getTheme('light', true),
	getTheme('dark', true),
]);
const html = computed(() => highlighter.codeToHtml(props.code, {
	lang: codeLang.value,
	themes: {
		fallback: 'dark-plus',
		light: lightThemeName,
		dark: darkThemeName,
	},
	defaultColor: false,
	cssVariablePrefix: '--shiki-',
}));
async function fetchLanguage(to: string): Promise<void> {
	const language = to as BundledLanguage;
	// Check for the loaded languages, and load the language if it's not loaded yet.
	if (!highlighter.getLoadedLanguages().includes(language)) {
		// Check if the language is supported by Shiki
		const bundles = bundledLanguagesInfo.filter((bundle) => {
			// Languages are specified by their id, they can also have aliases (i. e. "js" and "javascript")
			return bundle.id === language || bundle.aliases?.includes(language);
		});
		if (bundles.length > 0) {
			if (_DEV_) console.log(`Loading language: ${language}`);
;(
  ([__temp,__restore] = _withAsyncContext(() => highlighter.loadLanguage(bundles[0].import))),
  await __temp,
  __restore()
)
			codeLang.value = language;
		} else {
			codeLang.value = 'js';
		}
	} else {
		codeLang.value = language;
	}
}
watch(() => props.lang, (to) => {
	if (codeLang.value === to || !to) return;
	return new Promise((resolve) => {
		fetchLanguage(to).then(() => resolve);
	});
}, { immediate: true });

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.codeBlockRoot, {
  		[_ctx.$style.codeEditor]: __props.codeEditor,
  		[_ctx.$style.outerStyle]: !__props.codeEditor && __props.withOuterStyle,
  		[_ctx.$style.dark]: _unref(darkMode),
  		[_ctx.$style.light]: !_unref(darkMode),
  	}]),
      innerHTML: html.value
    }, null, 10 /* CLASS, PROPS */, ["innerHTML"]))
}
}

})
