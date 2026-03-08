import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, createSlots as _createSlots, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { ref, computed, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import MkPagination from '@/components/MkPagination.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'apps',
  setup(__props) {

const paginator = markRaw(new Paginator('i/apps', {
	limit: 100,
	noPaging: true,
	params: {
		sort: '+lastUsedAt',
	},
}));
function revoke(token: Misskey.entities.IAppsResponse[number]) {
	misskeyApi('i/revoke-token', { tokenId: token.id }).then(() => {
		paginator.reload();
	});
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.installedApps,
	icon: 'ti ti-plug',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkTime = _resolveComponent("MkTime")

  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(MkPagination, { paginator: _unref(paginator) }, {
        empty: _withCtx(() => [
          _createVNode(_component_MkResult, { type: "empty" })
        ]),
        default: _withCtx(({items}) => [
          _createElementVNode("div", { class: "_gaps" }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (token) => {
              return (_openBlock(), _createBlock(MkFolder, {
                key: token.id,
                defaultOpen: true
              }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                {
                  name: "icon",
                  fn: _withCtx(() => [
                    (token.iconUrl)
                      ? (_openBlock(), _createElementBlock("img", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.appIcon),
                        src: token.iconUrl,
                        alt: ""
                      }))
                      : (_openBlock(), _createElementBlock("i", {
                        key: 1,
                        class: "ti ti-plug"
                      }))
                  ])
                },
                {
                  name: "label",
                  fn: _withCtx(() => [
                    _createTextVNode(_toDisplayString(token.name), 1 /* TEXT */)
                  ])
                },
                {
                  name: "caption",
                  fn: _withCtx(() => [
                    _createTextVNode(_toDisplayString(token.description), 1 /* TEXT */)
                  ])
                },
                (token.lastUsedAt != null)
                  ? {
                    name: "suffix",
                    fn: _withCtx(() => [
                      _createVNode(_component_MkTime, { time: token.lastUsedAt }, null, 8 /* PROPS */, ["time"])
                    ]),
                    key: "0"
                  }
                : undefined,
                {
                  name: "footer",
                  fn: _withCtx(() => [
                    _createVNode(MkButton, {
                      danger: "",
                      onClick: _cache[0] || (_cache[0] = ($event: any) => (revoke(token)))
                    }, {
                      default: _withCtx(() => [
                        _hoisted_1,
                        _createTextVNode(" "),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    })
                  ])
                }
              ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen"]))
            }), 128 /* KEYED_FRAGMENT */))
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["paginator"]) ]))
}
}

})
