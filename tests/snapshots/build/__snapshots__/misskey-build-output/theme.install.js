import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { ref, computed } from 'vue'
import MkCodeEditor from '@/components/MkCodeEditor.vue'
import MkButton from '@/components/MkButton.vue'
import { parseThemeCode, previewTheme, installTheme } from '@/theme.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { useRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'theme.install',
  setup(__props) {

const router = useRouter();
const installThemeCode = ref<string | null>(null);
async function install(code: string): Promise<void> {
	try {
		const theme = parseThemeCode(code);
		await installTheme(code);
		os.alert({
			type: 'success',
			text: i18n.tsx._theme.installed({ name: theme.name }),
		});
		installThemeCode.value = null;
		router.push('/settings/theme');
	} catch (err: any) {
		switch (err.message.toLowerCase()) {
			case 'this theme is already installed':
				os.alert({
					type: 'info',
					text: i18n.ts._theme.alreadyInstalled,
				});
				break;
			default:
				os.alert({
					type: 'error',
					text: i18n.ts._theme.invalid,
				});
				break;
		}
		console.error(err);
	}
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts._theme.install,
	icon: 'ti ti-download',
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(MkCodeEditor, {
        lang: "json5",
        modelValue: installThemeCode.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((installThemeCode).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.code), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createElementVNode("div", { class: "_buttons" }, [ _createVNode(MkButton, {
          disabled: installThemeCode.value == null || installThemeCode.value.trim() === '',
          inline: "",
          onClick: _cache[1] || (_cache[1] = () => _unref(previewTheme)(installThemeCode.value))
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.preview), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["disabled"]), _createVNode(MkButton, {
          disabled: installThemeCode.value == null || installThemeCode.value.trim() === '',
          primary: "",
          inline: "",
          onClick: _cache[2] || (_cache[2] = () => install(installThemeCode.value))
        }, {
          default: _withCtx(() => [
            _hoisted_2,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.install), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["disabled"]) ]) ]))
}
}

})
