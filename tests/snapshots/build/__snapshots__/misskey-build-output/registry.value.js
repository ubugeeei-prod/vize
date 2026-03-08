import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { watch, computed, ref } from 'vue'
import JSON5 from 'json5'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkCodeEditor from '@/components/MkCodeEditor.vue'
import FormSplit from '@/components/form/split.vue'
import FormInfo from '@/components/MkInfo.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'registry.value',
  props: {
    path: { type: String, required: true },
    domain: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const scope = computed(() => props.path.split('/').slice(0, -1));
const key = computed(() => props.path.split('/').at(-1)!);
const value = ref<any>(null);
const valueForEditor = ref<string>('');
function fetchValue() {
	misskeyApi('i/registry/get-detail', {
		scope: scope.value,
		key: key.value,
		domain: props.domain === '@' ? null : props.domain,
	}).then(res => {
		value.value = res;
		valueForEditor.value = JSON5.stringify(res.value, null, '\t');
	});
}
async function save() {
	try {
		JSON5.parse(valueForEditor.value);
	} catch (err) {
		os.alert({
			type: 'error',
			text: i18n.ts.invalidValue,
		});
		return;
	}
	os.confirm({
		type: 'warning',
		text: i18n.ts.saveConfirm,
	}).then(({ canceled }) => {
		if (canceled) return;
		os.apiWithDialog('i/registry/set', {
			scope: scope.value,
			key: key.value,
			value: JSON5.parse(valueForEditor.value),
			domain: props.domain === '@' ? null : props.domain,
		});
	});
}
function del() {
	os.confirm({
		type: 'warning',
		text: i18n.ts.deleteConfirm,
	}).then(({ canceled }) => {
		if (canceled) return;
		os.apiWithDialog('i/registry/remove', {
			scope: scope.value,
			key: key.value,
			domain: props.domain === '@' ? null : props.domain,
		});
	});
}
watch(() => props.path, fetchValue, { immediate: true });
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.registry,
	icon: 'ti ti-adjustments',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 16px;"
        }, [
          _createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(FormInfo, { warn: "" }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.editTheseSettingsMayBreakAccount), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            (value.value)
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                _createVNode(FormSplit, null, {
                  default: _withCtx(() => [
                    _createVNode(MkKeyValue, null, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._registry.domain), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        _createTextVNode(_toDisplayString(props.domain === '@' ? _unref(i18n).ts.system : props.domain.toUpperCase()), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkKeyValue, null, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._registry.scope), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        _createTextVNode(_toDisplayString(scope.value.join('/')), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(MkKeyValue, null, {
                      key: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._registry.key), 1 /* TEXT */)
                      ]),
                      value: _withCtx(() => [
                        _createTextVNode(_toDisplayString(key.value), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkCodeEditor, {
                  lang: "json5",
                  modelValue: valueForEditor.value,
                  "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((valueForEditor).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.value) + " (JSON)", 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"]),
                _createVNode(MkButton, {
                  primary: "",
                  onClick: save
                }, {
                  default: _withCtx(() => [
                    _hoisted_1,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkKeyValue, null, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.updatedAt), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    _createVNode(_component_MkTime, {
                      time: value.value.updatedAt,
                      mode: "detail"
                    }, null, 8 /* PROPS */, ["time"])
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkButton, {
                  danger: "",
                  onClick: del
                }, {
                  default: _withCtx(() => [
                    _hoisted_2,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ], 64 /* STABLE_FRAGMENT */))
              : _createCommentVNode("v-if", true)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
