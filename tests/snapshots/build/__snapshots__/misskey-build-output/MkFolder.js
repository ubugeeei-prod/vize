import { defineComponent as _defineComponent } from 'vue'
import { Teleport as _Teleport, KeepAlive as _KeepAlive, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"

import { nextTick, onMounted, ref, useTemplateRef, watch } from 'vue'
import { prefer } from '@/preferences.js'
import { getBgColor } from '@/utility/get-bg-color.js'
import { pageFolderTeleportCount, popup } from '@/os.js'
import MkFolderPage from '@/components/MkFolderPage.vue'
import { deviceKind } from '@/utility/device-kind.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFolder',
  props: {
    defaultOpen: { type: Boolean, required: false, default: false },
    maxHeight: { type: Number, required: false, default: null },
    withSpacer: { type: Boolean, required: false, default: true },
    spacerMin: { type: Number, required: false, default: 14 },
    spacerMax: { type: Number, required: false, default: 22 },
    canPage: { type: Boolean, required: false, default: true }
  },
  emits: ["opened", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const rootEl = useTemplateRef('rootEl');
const asPage = props.canPage && deviceKind === 'smartphone' && prefer.s['experimental.enableFolderPageView'];
const bgSame = ref(false);
const opened = ref(asPage ? false : props.defaultOpen);
const openedAtLeastOnce = ref(opened.value);
//#region interpolate-sizeに対応していないブラウザ向け（TODO: 主要ブラウザが対応したら消す）
function enter(el: Element) {
	if (CSS.supports('interpolate-size', 'allow-keywords')) return;
	if (!(el instanceof HTMLElement)) return;
	const elementHeight = el.getBoundingClientRect().height;
	el.style.height = '0';
	el.offsetHeight; // reflow
	el.style.height = `${Math.min(elementHeight, props.maxHeight ?? Infinity)}px`;
}
function afterEnter(el: Element) {
	if (CSS.supports('interpolate-size', 'allow-keywords')) return;
	if (!(el instanceof HTMLElement)) return;
	el.style.height = '';
}
function leave(el: Element) {
	if (CSS.supports('interpolate-size', 'allow-keywords')) return;
	if (!(el instanceof HTMLElement)) return;
	const elementHeight = el.getBoundingClientRect().height;
	el.style.height = `${elementHeight}px`;
	el.offsetHeight; // reflow
	el.style.height = '0';
}
function afterLeave(el: Element) {
	if (CSS.supports('interpolate-size', 'allow-keywords')) return;
	if (!(el instanceof HTMLElement)) return;
	el.style.height = '';
}
//#endregion
let pageId = pageFolderTeleportCount.value;
pageFolderTeleportCount.value += 1000;
async function toggle(ev: PointerEvent) {
	if (asPage && !opened.value) {
		pageId++;
		const { dispose } = await popup(MkFolderPage, {
			pageId,
		}, {
			closed: () => {
				opened.value = false;
				dispose();
			},
		});
	}
	if (!opened.value) {
		openedAtLeastOnce.value = true;
	}
	nextTick(() => {
		opened.value = !opened.value;
	});
}
onMounted(() => {
	const computedStyle = getComputedStyle(window.document.documentElement);
	const parentBg = getBgColor(rootEl.value?.parentElement) ?? 'transparent';
	const myBg = computedStyle.getPropertyValue('--MI_THEME-panel');
	bgSame.value = parentBg === myBg;
});
watch(opened, (isOpened) => {
	if (isOpened) {
		emit('opened');
	} else {
		emit('closed');
	}
}, { flush: 'post' });

return (_ctx: any,_cache: any) => {
  const _component_MkCondensedLine = _resolveComponent("MkCondensedLine")
  const _component_MkStickyContainer = _resolveComponent("MkStickyContainer")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass(_ctx.$style.root),
      role: "group",
      "aria-expanded": opened.value
    }, [ _createVNode(_component_MkStickyContainer, null, {
        header: _withCtx(() => [
          _createElementVNode("button", {
            class: _normalizeClass(["_button", [_ctx.$style.header, { [_ctx.$style.opened]: opened.value }]]),
            role: "button",
            "data-cy-folder-header": "",
            onClick: toggle
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.headerIcon)
            }, [
              _renderSlot(_ctx.$slots, "icon")
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.headerText)
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.headerTextMain)
              }, [
                _createVNode(_component_MkCondensedLine, { minScale: 2 / 3 }, {
                  default: _withCtx(() => [
                    _renderSlot(_ctx.$slots, "label")
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["minScale"])
              ]),
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.headerTextSub)
              }, [
                _renderSlot(_ctx.$slots, "caption")
              ])
            ]),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.headerRight)
            }, [
              _createElementVNode("span", {
                class: _normalizeClass(_ctx.$style.headerRightText)
              }, [
                _renderSlot(_ctx.$slots, "suffix")
              ]),
              (_unref(asPage))
                ? (_openBlock(), _createElementBlock("i", {
                  key: 0,
                  class: "ti ti-chevron-right icon"
                }))
                : (opened.value)
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-chevron-up icon"
                  }))
                : (_openBlock(), _createElementBlock("i", {
                  key: 2,
                  class: "ti ti-chevron-down icon"
                }))
            ])
          ], 2 /* CLASS */)
        ]),
        default: _withCtx(() => [
          (_unref(asPage))
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              (opened.value)
                ? (_openBlock(), _createBlock(_Teleport, {
                  key: 0,
                  defer: "",
                  to: `#v-${_unref(pageId)}-header`
                }, [_renderSlot(_ctx.$slots, "label")], 8 /* PROPS */, ["to"]))
                : _createCommentVNode("v-if", true),
              (opened.value)
                ? (_openBlock(), _createBlock(_Teleport, {
                  key: 0,
                  defer: "",
                  to: `#v-${_unref(pageId)}-body`
                }, [_createVNode(_component_MkStickyContainer, null, {
                  header: _withCtx(() => [
                    (_ctx.$slots.header)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.inBodyHeader)
                      }, [
                        _renderSlot(_ctx.$slots, "header")
                      ]))
                      : _createCommentVNode("v-if", true)
                  ]),
                  footer: _withCtx(() => [
                    (_ctx.$slots.footer)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.inBodyFooter)
                      }, [
                        _renderSlot(_ctx.$slots, "footer")
                      ]))
                      : _createCommentVNode("v-if", true)
                  ]),
                  default: _withCtx(() => [
                    (__props.withSpacer)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: "_spacer",
                        style: _normalizeStyle({ '--MI_SPACER-min': props.spacerMin + 'px', '--MI_SPACER-max': props.spacerMax + 'px' })
                      }, [
                        _renderSlot(_ctx.$slots, "default")
                      ]))
                      : (_openBlock(), _createElementBlock("div", { key: 1 }, [
                        _renderSlot(_ctx.$slots, "default")
                      ]))
                  ]),
                  _: 1 /* STABLE */
                })], 8 /* PROPS */, ["to"]))
                : _createCommentVNode("v-if", true)
            ]))
            : (openedAtLeastOnce.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                class: _normalizeClass([_ctx.$style.body, { [_ctx.$style.bgSame]: bgSame.value }]),
                style: _normalizeStyle({ maxHeight: __props.maxHeight ? `${__props.maxHeight}px` : undefined, overflow: __props.maxHeight ? `auto` : undefined }),
                "aria-hidden": !opened.value
              }, [
                _createVNode(_Transition, {
                  enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_toggle_enterActive : '',
                  leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_toggle_leaveActive : '',
                  enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_toggle_enterFrom : '',
                  leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_toggle_leaveTo : '',
                  onEnter: enter,
                  onAfterEnter: afterEnter,
                  onLeave: leave,
                  onAfterLeave: afterLeave
                }, {
                  default: _withCtx(() => [
                    _createVNode(_KeepAlive, null, [
                      _withDirectives(_createElementVNode("div", null, [
                        _createVNode(_component_MkStickyContainer, null, {
                          header: _withCtx(() => [
                            (_ctx.$slots.header)
                              ? (_openBlock(), _createElementBlock("div", {
                                key: 0,
                                class: _normalizeClass(_ctx.$style.inBodyHeader)
                              }, [
                                _renderSlot(_ctx.$slots, "header")
                              ]))
                              : _createCommentVNode("v-if", true)
                          ]),
                          footer: _withCtx(() => [
                            (_ctx.$slots.footer)
                              ? (_openBlock(), _createElementBlock("div", {
                                key: 0,
                                class: _normalizeClass(_ctx.$style.inBodyFooter)
                              }, [
                                _renderSlot(_ctx.$slots, "footer")
                              ]))
                              : _createCommentVNode("v-if", true)
                          ]),
                          default: _withCtx(() => [
                            (__props.withSpacer)
                              ? (_openBlock(), _createElementBlock("div", {
                                key: 0,
                                class: "_spacer",
                                style: _normalizeStyle({ '--MI_SPACER-min': props.spacerMin + 'px', '--MI_SPACER-max': props.spacerMax + 'px' })
                              }, [
                                _renderSlot(_ctx.$slots, "default")
                              ]))
                              : (_openBlock(), _createElementBlock("div", { key: 1 }, [
                                _renderSlot(_ctx.$slots, "default")
                              ]))
                          ]),
                          _: 1 /* STABLE */
                        })
                      ], 512 /* NEED_PATCH */), [
                        [_vShow, opened.value]
                      ])
                    ], 1024 /* DYNAMIC_SLOTS */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"])
              ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }) ], 8 /* PROPS */, ["aria-expanded"]))
}
}

})
