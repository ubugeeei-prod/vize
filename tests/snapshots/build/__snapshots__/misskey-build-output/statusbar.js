import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { onMounted, ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import XStatusbar from './statusbar.statusbar.vue'
import { genId } from '@/utility/id.js'
import MkFolder from '@/components/MkFolder.vue'
import MkButton from '@/components/MkButton.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'statusbar',
  setup(__props) {

const statusbars = prefer.r.statusbars;
const userLists = ref<Misskey.entities.UserList[] | null>(null);
onMounted(() => {
	misskeyApi('users/lists/list').then(res => {
		userLists.value = res;
	});
});
async function add() {
	prefer.commit('statusbars', [...statusbars.value, {
		id: genId(),
		name: null,
		type: null,
		black: false,
		size: 'medium',
		props: {},
	}]);
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.statusbar,
	icon: 'ti ti-list',
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(statusbars), (x) => {
        return (_openBlock(), _createBlock(MkFolder, { key: x.id }, {
          label: _withCtx(() => [
            _createTextVNode(_toDisplayString(x.type ?? _unref(i18n).ts.notSet), 1 /* TEXT */)
          ]),
          suffix: _withCtx(() => [
            _createTextVNode(_toDisplayString(x.name), 1 /* TEXT */)
          ]),
          default: _withCtx(() => [
            _createVNode(XStatusbar, {
              _id: x.id,
              userLists: userLists.value
            }, null, 8 /* PROPS */, ["_id", "userLists"])
          ]),
          _: 2 /* DYNAMIC */
        }, 1024 /* DYNAMIC_SLOTS */))
      }), 128 /* KEYED_FRAGMENT */)), _createVNode(MkButton, {
        primary: "",
        onClick: add
      }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
