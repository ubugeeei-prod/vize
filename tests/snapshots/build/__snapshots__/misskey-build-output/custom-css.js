import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, watch, computed } from 'vue'
import MkCodeEditor from '@/components/MkCodeEditor.vue'
import FormInfo from '@/components/MkInfo.vue'
import { isSafeMode } from '@@/js/config.js'
import * as os from '@/os.js'
import { unisonReload } from '@/utility/unison-reload.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { miLocalStorage } from '@/local-storage.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'custom-css',
  setup(__props) {

const localCustomCss = ref(miLocalStorage.getItem('customCss') ?? '');
async function apply() {
	miLocalStorage.setItem('customCss', localCustomCss.value);
	const { canceled } = await os.confirm({
		type: 'info',
		text: i18n.ts.reloadToApplySetting,
	});
	if (canceled) return;
	unisonReload();
}
watch(localCustomCss, async () => {
	await apply();
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.customCss,
	icon: 'ti ti-code',
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(FormInfo, { warn: "" }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.customCssWarn), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }), (_unref(isSafeMode)) ? (_openBlock(), _createBlock(FormInfo, {
          key: 0,
          warn: ""
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.customCssIsDisabledBecauseSafeMode), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : _createCommentVNode("v-if", true), _createVNode(MkCodeEditor, {
        manualSave: "",
        lang: "css",
        modelValue: localCustomCss.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((localCustomCss).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode("CSS")
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]) ]))
}
}

})
