import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { watch, computed, ref } from 'vue'
import JSON5 from 'json5'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import FormLink from '@/components/form/link.vue'
import FormSection from '@/components/form/section.vue'
import MkButton from '@/components/MkButton.vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import FormSplit from '@/components/form/split.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'registry.keys',
  props: {
    path: { type: String, required: true },
    domain: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const scope = computed(() => props.path ? props.path.split('/') : []);
const keys = ref<[string, string][]>([]);
function fetchKeys() {
	misskeyApi('i/registry/keys-with-type', {
		scope: scope.value,
		domain: props.domain === '@' ? null : props.domain,
	}).then(res => {
		keys.value = Object.entries(res).sort((a, b) => a[0].localeCompare(b[0]));
	});
}
async function createKey() {
	const { canceled, result } = await os.form(i18n.ts._registry.createKey, {
		key: {
			type: 'string',
			label: i18n.ts._registry.key,
		},
		value: {
			type: 'string',
			multiline: true,
			label: i18n.ts.value,
		},
		scope: {
			type: 'string',
			label: i18n.ts._registry.scope,
			default: scope.value.join('/'),
		},
	});
	if (canceled) return;
	os.apiWithDialog('i/registry/set', {
		scope: result.scope.split('/'),
		key: result.key,
		value: JSON5.parse(result.value),
	}).then(() => {
		fetchKeys();
	});
}
watch(() => props.path, fetchKeys, { immediate: true });
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.registry,
	icon: 'ti ti-adjustments',
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
          style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 16px;"
        }, [
          _createElementVNode("div", { class: "_gaps_m" }, [
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
                })
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkButton, {
              primary: "",
              onClick: createKey
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._registry.createKey), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            (keys.value)
              ? (_openBlock(), _createBlock(FormSection, { key: 0 }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._registry.keys), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(keys.value, (key) => {
                      return (_openBlock(), _createBlock(FormLink, { to: `/registry/value/${props.domain}/${scope.value.join('/')}/${key[0]}`, class: "_monospace" }, {
                        suffix: _withCtx(() => [
                          _createTextVNode(_toDisplayString(key[1].toUpperCase()), 1 /* TEXT */)
                        ]),
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(key[0]), 1 /* TEXT */)
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                    }), 256 /* UNKEYED_FRAGMENT */))
                  ])
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
