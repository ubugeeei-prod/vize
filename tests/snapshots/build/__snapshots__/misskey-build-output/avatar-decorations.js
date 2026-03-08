import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, computed, defineAsyncComponent } from 'vue'
import * as Misskey from 'misskey-js'
import { ensureSignin } from '@/i.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'avatar-decorations',
  setup(__props) {

const $i = ensureSignin();
const avatarDecorations = ref<Misskey.entities.AdminAvatarDecorationsListResponse>([]);
function load() {
	misskeyApi('admin/avatar-decorations/list').then(_avatarDecorations => {
		avatarDecorations.value = _avatarDecorations;
	});
}
load();
async function add(ev: PointerEvent) {
	const { dispose } = await os.popupAsyncWithDialog(import('./avatar-decoration-edit-dialog.vue').then(x => x.default), {
	}, {
		done: result => {
			if (result.created) {
				avatarDecorations.value.unshift(result.created);
			}
		},
		closed: () => dispose(),
	});
}
async function edit(avatarDecoration: Misskey.entities.AdminAvatarDecorationsListResponse[number]) {
	const { dispose } = await os.popupAsyncWithDialog(import('./avatar-decoration-edit-dialog.vue').then(x => x.default), {
		avatarDecoration: avatarDecoration,
	}, {
		done: result => {
			if (result.updated) {
				const index = avatarDecorations.value.findIndex(x => x.id === avatarDecoration.id);
				avatarDecorations.value[index] = {
					...avatarDecorations.value[index],
					...result.updated,
				};
			} else if (result.deleted) {
				avatarDecorations.value = avatarDecorations.value.filter(x => x.id !== avatarDecoration.id);
			}
		},
		closed: () => dispose(),
	});
}
const headerActions = computed(() => [{
	asFullButton: true,
	icon: 'ti ti-plus',
	text: i18n.ts.add,
	handler: add,
}]);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.avatarDecorations,
	icon: 'ti ti-sparkles',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkCondensedLine = _resolveComponent("MkCondensedLine")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 900px;"
        }, [
          _createElementVNode("div", { class: "_gaps" }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.decorations)
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(avatarDecorations.value, (avatarDecoration) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: avatarDecoration.id,
                  class: _normalizeClass(_ctx.$style.decoration),
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (edit(avatarDecoration)))
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.decorationName)
                  }, [
                    _createVNode(_component_MkCondensedLine, { minScale: 0.5 }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(avatarDecoration.name), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["minScale"])
                  ]),
                  _createVNode(_component_MkAvatar, {
                    style: "width: 60px; height: 60px;",
                    user: _unref($i),
                    decorations: [{ url: avatarDecoration.url }],
                    forceShowDecoration: ""
                  }, null, 8 /* PROPS */, ["user", "decorations"])
                ]))
              }), 128 /* KEYED_FRAGMENT */))
            ])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
