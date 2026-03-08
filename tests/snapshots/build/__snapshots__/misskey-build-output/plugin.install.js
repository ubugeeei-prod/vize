import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { nextTick, ref, computed } from 'vue'
import MkCodeEditor from '@/components/MkCodeEditor.vue'
import MkButton from '@/components/MkButton.vue'
import FormInfo from '@/components/MkInfo.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { installPlugin } from '@/plugin.js'
import { useRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'plugin.install',
  setup(__props) {

const router = useRouter();
const code = ref<string | null>(null);
async function install() {
	if (!code.value) return;
	try {
		await installPlugin(code.value);
		os.success();
		code.value = null;
		router.push('/settings/plugin');
	} catch (err: any) {
		os.alert({
			type: 'error',
			title: 'Install failed',
			text: err.toString() ?? null,
		});
	}
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts._plugin.install,
	icon: 'ti ti-download',
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(FormInfo, { warn: "" }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._plugin.installWarn), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), _createVNode(MkCodeEditor, {
        lang: "is",
        modelValue: code.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((code).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.code), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createElementVNode("div", null, [ _createVNode(MkButton, {
          disabled: code.value == null || code.value.trim() === '',
          primary: "",
          inline: "",
          onClick: install
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.install), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["disabled"]) ]) ]))
}
}

})
