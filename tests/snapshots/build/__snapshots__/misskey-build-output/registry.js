import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import JSON5 from 'json5'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import FormLink from '@/components/form/link.vue'
import FormSection from '@/components/form/section.vue'
import MkButton from '@/components/MkButton.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'registry',
  setup(__props) {

const scopesWithDomain = ref<Misskey.entities.IRegistryScopesWithDomainResponse | null>(null);
function fetchScopes() {
	misskeyApi('i/registry/scopes-with-domain').then(res => {
		scopesWithDomain.value = res;
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
		},
	});
	if (canceled) return;
	os.apiWithDialog('i/registry/set', {
		scope: result.scope.split('/'),
		key: result.key,
		value: JSON5.parse(result.value),
	}).then(() => {
		fetchScopes();
	});
}
fetchScopes();
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
          _createVNode(MkButton, {
            primary: "",
            onClick: createKey
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._registry.createKey), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          (scopesWithDomain.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps_m"
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(scopesWithDomain.value, (domain) => {
                return (_openBlock(), _createBlock(FormSection, { key: domain.domain ?? 'system' }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(domain.domain ? domain.domain.toUpperCase() : _unref(i18n).ts.system), 1 /* TEXT */)
                  ]),
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(domain.scopes, (scope) => {
                        return (_openBlock(), _createBlock(FormLink, { to: `/registry/keys/${domain.domain ?? '@'}/${scope.join('/')}`, class: "_monospace" }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(scope.length === 0 ? '(root)' : scope.join('/')), 1 /* TEXT */)
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
                      }), 256 /* UNKEYED_FRAGMENT */))
                    ])
                  ]),
                  _: 2 /* DYNAMIC */
                }, 1024 /* DYNAMIC_SLOTS */))
              }), 128 /* KEYED_FRAGMENT */))
            ]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
