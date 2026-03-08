import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, renderList as _renderList, mergeProps as _mergeProps, normalizeClass as _normalizeClass, toHandlers as _toHandlers, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "_indicatorCircle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "_indicatorCircle" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil ti-fw" })
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'
import { openInstanceMenu } from './common.js'
import * as os from '@/os.js'
import { navbarItemDef } from '@/navbar.js'
import MkButton from '@/components/MkButton.vue'
import { instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import { getAccountMenu } from '@/accounts.js'
import { $i } from '@/i.js'
import { getHTMLElementOrNull } from '@/utility/get-dom-node-or-null.js'
const WINDOW_THRESHOLD = 1400;

export default /*@__PURE__*/_defineComponent({
  __name: 'navbar-h',
  props: {
    acrylic: { type: Boolean, required: false }
  },
  setup(__props: any) {

const props = __props
const settingsWindowed = ref(window.innerWidth > WINDOW_THRESHOLD);
const menu = ref(prefer.s.menu);
// const menuDisplay = store.model('menuDisplay');
const otherNavItemIndicated = computed<boolean>(() => {
	for (const def in navbarItemDef) {
		if (menu.value.includes(def)) continue;
		if (navbarItemDef[def].indicated) return true;
	}
	return false;
});
async function more(ev: PointerEvent) {
	const target = getHTMLElementOrNull(ev.currentTarget ?? ev.target);
	if (!target) return;
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkLaunchPad.vue').then(x => x.default), {
		anchorElement: target,
		anchor: { x: 'center', y: 'bottom' },
	}, {
		closed: () => dispose(),
	});
}
async function openAccountMenu(ev: PointerEvent) {
	const menuItems = await getAccountMenu({
		withExtraOperation: true,
	});
	os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
}
onMounted(() => {
	window.addEventListener('resize', () => {
		settingsWindowed.value = (window.innerWidth >= WINDOW_THRESHOLD);
	}, { passive: true });
});

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _directive_click_anime = _resolveDirective("click-anime")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root, __props.acrylic ? _ctx.$style.acrylic : null])
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.body)
      }, [ _createElementVNode("div", null, [ _createElementVNode("button", {
            class: _normalizeClass(["_button", [_ctx.$style.item, _ctx.$style.instance]]),
            onClick: _cache[0] || (_cache[0] = (...args) => (openInstanceMenu && openInstanceMenu(...args)))
          }, [ _createElementVNode("img", {
              class: _normalizeClass(_ctx.$style.instanceIcon),
              src: _unref(instance).iconUrl ?? '/favicon.ico',
              draggable: "false"
            }, null, 8 /* PROPS */, ["src"]) ]), _createVNode(_component_MkA, {
            class: _normalizeClass(_ctx.$style.item),
            activeClass: _ctx.$style.active,
            to: "/",
            exact: ""
          }, {
            default: _withCtx(() => [
              _createElementVNode("i", {
                class: _normalizeClass(["ti ti-home ti-fw", _ctx.$style.itemIcon])
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["activeClass"]), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(menu.value, (item) => {
            return (_openBlock(), _createElementBlock(_Fragment, null, [
              (item === '-')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.divider)
                }))
                : (_unref(navbarItemDef)[item] && (_unref(navbarItemDef)[item].show == null || _unref(navbarItemDef)[item].show.value !== false))
                  ? (_openBlock(), _createBlock(_resolveDynamicComponent(_unref(navbarItemDef)[item].to ? 'MkA' : 'button'), _mergeProps(_toHandlers(_unref(navbarItemDef)[item].action ? { click: _unref(navbarItemDef)[item].action } : {}, true), {
                    key: 1,
                    is: _unref(navbarItemDef)[item].to ? 'MkA' : 'button',
                    class: _normalizeClass(["_button", _ctx.$style.item]),
                    activeClass: _ctx.$style.active,
                    to: _unref(navbarItemDef)[item].to
                  }), {
                    default: _withCtx(() => [
                      _createElementVNode("i", {
                        class: _normalizeClass(["ti-fw", [_ctx.$style.itemIcon, _unref(navbarItemDef)[item].icon]])
                      }, null, 2 /* CLASS */),
                      (_unref(navbarItemDef)[item].indicated)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          class: _normalizeClass(["_blink", _ctx.$style.indicator])
                        }, [
                          _hoisted_1
                        ]))
                        : _createCommentVNode("v-if", true)
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 16 /* FULL_PROPS */, ["activeClass", "to"]))
                : _createCommentVNode("v-if", true)
            ], 64 /* STABLE_FRAGMENT */))
          }), 256 /* UNKEYED_FRAGMENT */)), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.divider)
          }), (_unref($i) && (_unref($i).isAdmin || _unref($i).isModerator)) ? (_openBlock(), _createBlock(_component_MkA, {
              key: 0,
              class: "item",
              activeClass: _ctx.$style.active,
              to: "/admin",
              behavior: settingsWindowed.value ? 'window' : null
            }, {
              default: _withCtx(() => [
                _createElementVNode("i", {
                  class: _normalizeClass(["ti ti-dashboard ti-fw", _ctx.$style.itemIcon])
                })
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["activeClass", "behavior"])) : _createCommentVNode("v-if", true), _createElementVNode("button", {
            class: _normalizeClass(["_button", _ctx.$style.item]),
            onClick: more
          }, [ _createElementVNode("i", {
              class: _normalizeClass(["ti ti-dots ti-fw", _ctx.$style.itemIcon])
            }), (otherNavItemIndicated.value) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: _normalizeClass(["_blink", _ctx.$style.indicator])
              }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.right)
        }, [ _createVNode(_component_MkA, {
            class: _normalizeClass(_ctx.$style.item),
            activeClass: _ctx.$style.active,
            to: "/settings",
            behavior: settingsWindowed.value ? 'window' : null
          }, {
            default: _withCtx(() => [
              _createElementVNode("i", {
                class: _normalizeClass(["ti ti-settings ti-fw", _ctx.$style.itemIcon])
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["activeClass", "behavior"]), (_unref($i)) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: _normalizeClass(["_button", [_ctx.$style.item, _ctx.$style.account]]),
              onClick: openAccountMenu
            }, [ _createVNode(_component_MkAvatar, {
                user: _unref($i),
                class: _normalizeClass(_ctx.$style.avatar)
              }, null, 8 /* PROPS */, ["user"]), _createVNode(_component_MkAcct, {
                class: _normalizeClass(_ctx.$style.acct),
                user: _unref($i)
              }, null, 8 /* PROPS */, ["user"]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.post),
            onClick: _cache[1] || (_cache[1] = ($event: any) => (os.post()))
          }, [ _createVNode(MkButton, {
              class: _normalizeClass(_ctx.$style.postButton),
              gradate: "",
              rounded: ""
            }, {
              default: _withCtx(() => [
                _hoisted_3
              ]),
              _: 1 /* STABLE */
            }) ]) ]) ]) ], 2 /* CLASS */))
}
}

})
