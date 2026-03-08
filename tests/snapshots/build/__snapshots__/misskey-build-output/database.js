import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"

import { computed } from 'vue'
import MkKeyValue from '@/components/MkKeyValue.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import bytes from '@/filters/bytes.js'
import number from '@/filters/number.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'database',
  setup(__props) {

const databasePromiseFactory = () => misskeyApi('admin/get-table-stats').then(res => Object.entries(res).sort((a, b) => b[1].size - a[1].size));
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.database,
	icon: 'ti ti-database',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkSuspense = _resolveComponent("MkSuspense")
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
          _createVNode(_component_MkSuspense, { p: databasePromiseFactory }, {
            default: _withCtx(({ result: database }) => [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(database, (table) => {
                return (_openBlock(), _createBlock(MkKeyValue, {
                  key: table[0],
                  oneline: "",
                  style: "margin: 1em 0;"
                }, {
                  key: _withCtx(() => [
                    _createTextVNode(_toDisplayString(table[0]), 1 /* TEXT */)
                  ]),
                  value: _withCtx(() => [
                    _createTextVNode(_toDisplayString(bytes(table[1].size)) + " (" + _toDisplayString(number(table[1].count)) + " recs)", 1 /* TEXT */)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 1024 /* DYNAMIC_SLOTS */))
              }), 128 /* KEYED_FRAGMENT */))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["p"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
