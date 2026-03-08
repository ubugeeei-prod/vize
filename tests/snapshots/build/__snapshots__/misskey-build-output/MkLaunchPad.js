import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "text" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "_indicatorCircle" })
const _hoisted_3 = { class: "text" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "_indicatorCircle" })
import { useTemplateRef } from 'vue'
import MkModal from '@/components/MkModal.vue'
import { navbarItemDef } from '@/navbar.js'
import { deviceKind } from '@/utility/device-kind.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkLaunchPad',
  props: {
    anchorElement: { type: null, required: false, default: null },
    anchor: { type: Object, required: false, default: () => ({ x: 'right', y: 'center' }) }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const preferedModalType = (deviceKind === 'desktop' && props.anchorElement != null) ? 'popup' :
	deviceKind === 'smartphone' ? 'drawer' :
	'dialog';
const modal = useTemplateRef('modal');
const menu = prefer.s.menu;
const items = Object.keys(navbarItemDef).filter(k => !menu.includes(k)).map(k => navbarItemDef[k]).filter(def => def.show == null ? true : def.show).map(def => ({
	type: def.to ? 'link' : 'button',
	text: def.title,
	icon: def.icon,
	to: def.to,
	action: def.action,
	indicate: def.indicated,
	indicateValue: def.indicateValue,
}));
function close() {
	modal.value?.close();
}

return (_ctx: any,_cache: any) => {
  const _component_MkA = _resolveComponent("MkA")
  const _directive_click_anime = _resolveDirective("click-anime")

  return (_openBlock(), _createBlock(MkModal, {
      ref_key: "modal", ref: modal,
      preferType: _unref(preferedModalType),
      anchor: __props.anchor,
      transparentBg: true,
      anchorElement: __props.anchorElement,
      onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(modal)?.close())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed'))),
      onEsc: _cache[2] || (_cache[2] = ($event: any) => (_unref(modal)?.close()))
    }, {
      default: _withCtx(({ type, maxHeight }) => [
        _createElementVNode("div", {
          class: _normalizeClass(["szkkfdyq _popup _shadow", { asDrawer: type === 'drawer' }]),
          style: { maxHeight: maxHeight ? maxHeight + 'px' : '' }
        }, [
          _createElementVNode("div", { class: "main" }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(items), (item) => {
              return (_openBlock(), _createElementBlock(_Fragment, { key: item.text }, [
                (item.action != null)
                  ? (_openBlock(), _createElementBlock("button", {
                    key: 0,
                    class: "_button item",
                    onClick: _cache[3] || (_cache[3] = ($event) => {
  	item.action($event);
  	close();
  })
                  }, [
                    _createElementVNode("i", {
                      class: _normalizeClass(["icon", item.icon])
                    }, null, 2 /* CLASS */),
                    _createElementVNode("div", _hoisted_1, _toDisplayString(item.text), 1 /* TEXT */),
                    (item.indicate && item.indicateValue)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "_indicateCounter indicatorWithValue"
                      }, _toDisplayString(item.indicateValue), 1 /* TEXT */))
                      : (item.indicate)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 1,
                          class: "indicator _blink"
                        }, [
                          _hoisted_2
                        ]))
                      : _createCommentVNode("v-if", true)
                  ]))
                  : (item.to != null)
                    ? (_openBlock(), _createBlock(_component_MkA, {
                      key: 1,
                      to: item.to,
                      class: "item",
                      onClickPassive: _cache[4] || (_cache[4] = ($event: any) => (close()))
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("i", {
                          class: _normalizeClass(["icon", item.icon])
                        }, null, 2 /* CLASS */),
                        _createElementVNode("div", _hoisted_3, _toDisplayString(item.text), 1 /* TEXT */),
                        (item.indicate && item.indicateValue)
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "_indicateCounter indicatorWithValue"
                          }, _toDisplayString(item.indicateValue), 1 /* TEXT */))
                          : (item.indicate)
                            ? (_openBlock(), _createElementBlock("span", {
                              key: 1,
                              class: "indicator _blink"
                            }, [
                              _hoisted_4
                            ]))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to"]))
                  : _createCommentVNode("v-if", true)
              ], 64 /* STABLE_FRAGMENT */))
            }), 128 /* KEYED_FRAGMENT */))
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["preferType", "anchor", "transparentBg", "anchorElement"]))
}
}

})
