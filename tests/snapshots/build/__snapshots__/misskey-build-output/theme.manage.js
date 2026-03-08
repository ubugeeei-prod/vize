import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { computed, ref } from 'vue'
import JSON5 from 'json5'
import type { Theme } from '@/theme.js'
import MkTextarea from '@/components/MkTextarea.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkInput from '@/components/MkInput.vue'
import MkButton from '@/components/MkButton.vue'
import { getBuiltinThemesRef, getThemesRef, removeTheme } from '@/theme.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import type { MkSelectItem } from '@/components/MkSelect.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'theme.manage',
  setup(__props) {

const installedThemes = getThemesRef();
const builtinThemes = getBuiltinThemesRef();
const {
	model: selectedThemeId,
	def: selectedThemeIdDef,
} = useMkSelect({
	items: computed<MkSelectItem<string | null>[]>(() => [{
		type: 'group',
		label: i18n.ts._theme.installedThemes,
		items: installedThemes.value.map(x => ({ label: x.name, value: x.id })),
	}, {
		type: 'group',
		label: i18n.ts._theme.builtinThemes,
		items: builtinThemes.value.map(x => ({ label: x.name, value: x.id })),
	}]),
	initialValue: null,
});
const themes = computed(() => [...installedThemes.value, ...builtinThemes.value]);
const selectedTheme = computed(() => {
	if (selectedThemeId.value == null) return null;
	return themes.value.find(x => x.id === selectedThemeId.value);
});
const selectedThemeCode = computed(() => {
	if (selectedTheme.value == null) return null;
	return JSON5.stringify(selectedTheme.value, null, '\t');
});
function copyThemeCode() {
	copyToClipboard(selectedThemeCode.value);
}
function uninstall() {
	removeTheme(selectedTheme.value as Theme);
	installedThemes.value = installedThemes.value.filter(t => t.id !== selectedThemeId.value);
	selectedThemeId.value = null;
	os.success();
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts._theme.manage,
	icon: 'ti ti-tool',
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(MkSelect, {
        items: _unref(selectedThemeIdDef),
        modelValue: _unref(selectedThemeId),
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((selectedThemeId).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.theme), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["items", "modelValue"]), (selectedTheme.value != null) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(MkInput, {
            readonly: "",
            modelValue: selectedTheme.value.author
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.author), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), (selectedTheme.value.desc) ? (_openBlock(), _createBlock(MkTextarea, {
              key: 0,
              readonly: "",
              modelValue: selectedTheme.value.desc
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.description), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"])) : _createCommentVNode("v-if", true), _createVNode(MkTextarea, {
            readonly: "",
            tall: "",
            modelValue: selectedThemeCode.value
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._theme.code), 1 /* TEXT */)
            ]),
            caption: _withCtx(() => [
              _createElementVNode("button", {
                class: "_textButton",
                onClick: _cache[1] || (_cache[1] = ($event: any) => (copyThemeCode()))
              }, _toDisplayString(_unref(i18n).ts.copy), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), (!_unref(builtinThemes).some((t) => t.id == selectedTheme.value.id)) ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              danger: "",
              onClick: _cache[2] || (_cache[2] = ($event: any) => (uninstall()))
            }, {
              default: _withCtx(() => [
                _hoisted_1,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.uninstall), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
