import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { style: "text-align: center;" }
import { markRaw } from 'vue'
import { i18n } from '@/i18n.js'
import MkFolder from '@/components/MkFolder.vue'
import XUser from '@/components/MkUserSetupDialog.User.vue'
import MkPagination from '@/components/MkPagination.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserSetupDialog.Follow',
  setup(__props) {

const pinnedUsersPaginator = markRaw(new Paginator('pinned-users', {
	noPaging: true,
	limit: 10,
}));
const popularUsersPaginator = markRaw(new Paginator('users', {
	limit: 10,
	noPaging: true,
	params: {
		state: 'alive',
		origin: 'local',
		sort: '+follower',
	},
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps" }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_unref(i18n).ts._initialAccountSetting.followUsers), 1 /* TEXT */), _createVNode(MkFolder, { defaultOpen: true }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.recommended), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(MkPagination, { paginator: _unref(pinnedUsersPaginator) }, {
            default: _withCtx(({ items }) => [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.users)
              }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                  return (_openBlock(), _createBlock(XUser, {
                    key: item.id,
                    user: item
                  }, null, 8 /* PROPS */, ["user"]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["paginator"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["defaultOpen"]), _createVNode(MkFolder, { defaultOpen: true }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.popularUsers), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createVNode(MkPagination, { paginator: _unref(popularUsersPaginator) }, {
            default: _withCtx(({ items }) => [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.users)
              }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {
                  return (_openBlock(), _createBlock(XUser, {
                    key: item.id,
                    user: item
                  }, null, 8 /* PROPS */, ["user"]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["paginator"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["defaultOpen"]) ]))
}
}

})
