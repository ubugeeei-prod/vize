import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers, withKeys as _withKeys } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "_indicatorCircle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "_indicatorCircle" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "_indicatorCircle" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-right ti-fw" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-right ti-fw" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "_indicatorCircle" })

import { computed, defineAsyncComponent, inject, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef, unref, watch, shallowRef } from 'vue';
import type { MenuItem, InnerMenuItem, MenuPending, MenuAction, MenuSwitch, MenuRadio, MenuRadioOption, MenuParent } from '@/types/menu.js';
import type { Keymap } from '@/utility/hotkey.js';
import MkSwitchButton from '@/components/MkSwitch.button.vue';
import * as os from '@/os.js';
import { i18n } from '@/i18n.js';
import { isTouchUsing } from '@/utility/touch.js';
import { isFocusable } from '@/utility/focus.js';
import { getNodeOrNull } from '@/utility/get-dom-node-or-null.js';

const childrenCache = new WeakMap<MenuParent, MenuItem[]>();

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMenu',
  props: {
    items: { type: Array, required: true },
    asDrawer: { type: Boolean, required: false },
    align: { type: String, required: false },
    width: { type: Number, required: false },
    maxHeight: { type: Number, required: false }
  },
  emits: ["close", "hide"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const XChild = defineAsyncComponent(() => import('./MkMenu.child.vue'));
const big = isTouchUsing;
const isNestingMenu = inject<boolean>('isNestingMenu', false);
const itemsEl = useTemplateRef('itemsEl');
const items2 = ref<InnerMenuItem[]>();
const child = useTemplateRef('child');
const keymap = {
	'up|k|shift+tab': {
		allowRepeat: true,
		callback: () => focusUp(),
	},
	'down|j|tab': {
		allowRepeat: true,
		callback: () => focusDown(),
	},
	'esc': {
		allowRepeat: true,
		callback: () => close(false),
	},
} as const satisfies Keymap;
const childShowingItem = ref<MenuItem | null>();
let preferClick = isTouchUsing || props.asDrawer;
watch(() => props.items, () => {
	const items = [...props.items].filter(item => item !== undefined) as (NonNullable<MenuItem> | MenuPending)[];
	for (let i = 0; i < items.length; i++) {
		const item = items[i];
		if ('then' in item) { // if item is Promise
			items[i] = { type: 'pending' };
			item.then(actualItem => {
				if (items2.value?.[i]) items2.value[i] = actualItem;
			});
		}
	}
	items2.value = items as InnerMenuItem[];
}, {
	immediate: true,
});
const childMenu = ref<MenuItem[] | null>();
const childTarget = shallowRef<HTMLElement>();
function closeChild() {
	childMenu.value = null;
	childShowingItem.value = null;
}
function childActioned() {
	closeChild();
	close(true);
}
let childCloseTimer: null | number = null;
function onItemMouseEnter() {
	childCloseTimer = window.setTimeout(() => {
		closeChild();
	}, 300);
}
function onItemMouseLeave() {
	if (childCloseTimer) window.clearTimeout(childCloseTimer);
}
async function showRadioOptions(item: MenuRadio, ev: MouseEvent | PointerEvent | KeyboardEvent) {
	const children: MenuItem[] = Object.keys(item.options).map<MenuRadioOption>(key => {
		const value = item.options[key];
		return {
			type: 'radioOption',
			text: key,
			action: () => {
				if ('value' in item.ref) {
					item.ref.value = value;
				} else {
					// @ts-expect-error リアクティビティは保たれる
					item.ref = value;
				}
			},
			active: computed(() => {
				if ('value' in item.ref) {
					return item.ref.value === value;
				} else {
					return item.ref === value;
				}
			}),
		};
	});
	if (props.asDrawer) {
		os.popupMenu(children, ev.currentTarget ?? ev.target).finally(() => {
			close(false);
		});
		emit('hide');
	} else {
		childTarget.value = (ev.currentTarget ?? ev.target) as HTMLElement;
		childMenu.value = children;
		childShowingItem.value = item;
	}
}
async function showChildren(item: MenuParent, ev: MouseEvent | PointerEvent | KeyboardEvent) {
	ev.stopPropagation();
	const children: MenuItem[] = await (async () => {
		if (childrenCache.has(item)) {
			return childrenCache.get(item)!;
		} else {
			if (typeof item.children === 'function') {
				return Promise.resolve(item.children());
			} else {
				return item.children;
			}
		}
	})();
	childrenCache.set(item, children);
	if (props.asDrawer) {
		os.popupMenu(children, ev.currentTarget ?? ev.target).finally(() => {
			close(false);
		});
		emit('hide');
	} else {
		childTarget.value = (ev.currentTarget ?? ev.target) as HTMLElement;
		// これでもリアクティビティは保たれる
		childMenu.value = children;
		childShowingItem.value = item;
	}
}
function clicked(fn: MenuAction, ev: PointerEvent, doClose = true) {
	fn(ev);
	if (!doClose) return;
	close(true);
}
function close(actioned = false) {
	disposeHandlers();
	nextTick(() => {
		closeChild();
		emit('close', actioned);
	});
}
function switchItem(item: MenuSwitch & { ref: any }) {
	if (item.disabled !== undefined && (typeof item.disabled === 'boolean' ? item.disabled : item.disabled.value)) return;
	item.ref = !item.ref;
}
function focusUp() {
	if (disposed) return;
	if (!itemsEl.value?.contains(window.document.activeElement)) return;
	const focusableElements = Array.from(itemsEl.value.children).filter(isFocusable);
	const activeIndex = focusableElements.findIndex(el => el === window.document.activeElement);
	const targetIndex = (activeIndex !== -1 && activeIndex !== 0) ? (activeIndex - 1) : (focusableElements.length - 1);
	const targetElement = focusableElements.at(targetIndex) ?? itemsEl.value;
	targetElement.focus();
}
function focusDown() {
	if (disposed) return;
	if (!itemsEl.value?.contains(window.document.activeElement)) return;
	const focusableElements = Array.from(itemsEl.value.children).filter(isFocusable);
	const activeIndex = focusableElements.findIndex(el => el === window.document.activeElement);
	const targetIndex = (activeIndex !== -1 && activeIndex !== (focusableElements.length - 1)) ? (activeIndex + 1) : 0;
	const targetElement = focusableElements.at(targetIndex) ?? itemsEl.value;
	targetElement.focus();
}
const onGlobalFocusin = (ev: FocusEvent) => {
	if (disposed) return;
	if (itemsEl.value?.parentElement?.contains(getNodeOrNull(ev.target))) return;
	nextTick(() => {
		if (itemsEl.value != null && isFocusable(itemsEl.value)) {
			itemsEl.value.focus({ preventScroll: true });
			nextTick(() => focusDown());
		}
	});
};
const onGlobalMousedown = (ev: MouseEvent) => {
	if (disposed) return;
	if (childTarget.value?.contains(getNodeOrNull(ev.target))) return;
	if (child.value?.checkHit(ev)) return;
	closeChild();
};
const setupHandlers = () => {
	if (!isNestingMenu) {
		window.document.addEventListener('focusin', onGlobalFocusin, { passive: true });
	}
	window.document.addEventListener('mousedown', onGlobalMousedown, { passive: true });
};
let disposed = false;
const disposeHandlers = () => {
	disposed = true;
	if (!isNestingMenu) {
		window.document.removeEventListener('focusin', onGlobalFocusin);
	}
	window.document.removeEventListener('mousedown', onGlobalMousedown);
};
onMounted(() => {
	setupHandlers();
	if (!isNestingMenu) {
		nextTick(() => itemsEl.value?.focus({ preventScroll: true }));
	}
});
onBeforeUnmount(() => {
	disposeHandlers();
});

return (_ctx: any,_cache: any) => {
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _directive_hotkey = _resolveDirective("hotkey")

  return (_openBlock(), _createElementBlock("div", {
      role: "menu",
      class: _normalizeClass({
  		[_ctx.$style.root]: true,
  		[_ctx.$style.center]: __props.align === 'center',
  		[_ctx.$style.big]: _unref(big),
  		[_ctx.$style.asDrawer]: __props.asDrawer,
  		[_ctx.$style.widthSpecified]: __props.width != null,
  	}),
      onFocusinPassive: _cache[0] || (_cache[0] = _withModifiers(() => {}, ["stop"]))
    }, [ _createElementVNode("div", {
        ref_key: "itemsEl", ref: itemsEl,
        tabindex: "0",
        class: _normalizeClass(["_popup _shadow", _ctx.$style.menu]),
        style: _normalizeStyle({
  			width: (__props.width && !__props.asDrawer) ? `${__props.width}px` : '',
  			maxHeight: __props.maxHeight ? `min(${__props.maxHeight}px, calc(100dvh - 32px))` : 'calc(100dvh - 32px)',
  		}),
        onKeydown: _cache[1] || (_cache[1] = _withModifiers(() => {}, ["stop"])),
        onContextmenu: _cache[2] || (_cache[2] = _withModifiers(() => {}, ["self","prevent"]))
      }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList((items2.value ?? []), (item) => {
          return (_openBlock(), _createElementBlock(_Fragment, null, [
            (item.type === 'divider')
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                role: "separator",
                tabindex: "-1",
                class: _normalizeClass(_ctx.$style.divider)
              }))
              : (item.type === 'label')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  role: "menuitem",
                  tabindex: "-1",
                  class: _normalizeClass([_ctx.$style.label])
                }, [
                  _createElementVNode("span", null, _toDisplayString(item.text), 1 /* TEXT */)
                ]))
              : (item.type === 'pending')
                ? (_openBlock(), _createElementBlock("span", {
                  key: 2,
                  role: "menuitem",
                  tabindex: "0",
                  class: _normalizeClass([_ctx.$style.pending, _ctx.$style.item])
                }, [
                  _createElementVNode("span", null, [
                    _createVNode(_component_MkEllipsis)
                  ])
                ]))
              : (item.type === 'component')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 3,
                  role: "menuitem",
                  tabindex: "-1"
                }, [
                  _createVNode(_resolveDynamicComponent(item.component), _mergeProps(item.props, {  }), null, 16 /* FULL_PROPS */)
                ]))
              : (item.type === 'link')
                ? (_openBlock(), _createBlock(_component_MkA, {
                  key: 4,
                  role: "menuitem",
                  tabindex: "0",
                  class: _normalizeClass(['_button', _ctx.$style.item]),
                  to: item.to,
                  onClickPassive: _cache[3] || (_cache[3] = ($event: any) => (close(true))),
                  onMouseenterPassive: onItemMouseEnter,
                  onMouseleavePassive: onItemMouseLeave
                }, {
                  default: _withCtx(() => [
                    (item.icon)
                      ? (_openBlock(), _createElementBlock("i", {
                        key: 0,
                        class: _normalizeClass(["ti-fw", [_ctx.$style.icon, item.icon]])
                      }))
                      : _createCommentVNode("v-if", true),
                    (item.avatar)
                      ? (_openBlock(), _createBlock(_component_MkAvatar, {
                        key: 0,
                        user: item.avatar,
                        class: _normalizeClass(_ctx.$style.avatar)
                      }, null, 8 /* PROPS */, ["user"]))
                      : _createCommentVNode("v-if", true),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.item_content)
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.item_content_text)
                      }, [
                        _createElementVNode("div", {
                          class: _normalizeClass(_ctx.$style.item_content_text_title)
                        }, _toDisplayString(item.text), 1 /* TEXT */),
                        (item.caption)
                          ? (_openBlock(), _createElementBlock("div", {
                            key: 0,
                            class: _normalizeClass(_ctx.$style.item_content_text_caption)
                          }, _toDisplayString(item.caption), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true)
                      ]),
                      (item.indicate)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          class: _normalizeClass(["_blink", _ctx.$style.indicator])
                        }, [
                          _hoisted_1
                        ]))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["to"]))
              : (item.type === 'a')
                ? (_openBlock(), _createElementBlock("a", {
                  key: 5,
                  role: "menuitem",
                  tabindex: "0",
                  class: _normalizeClass(['_button', _ctx.$style.item]),
                  href: item.href,
                  target: item.target,
                  rel: item.target === '_blank' ? 'noopener noreferrer' : undefined,
                  download: item.download,
                  onClickPassive: _cache[4] || (_cache[4] = ($event: any) => (close(true))),
                  onMouseenterPassive: onItemMouseEnter,
                  onMouseleavePassive: onItemMouseLeave
                }, [
                  (item.icon)
                    ? (_openBlock(), _createElementBlock("i", {
                      key: 0,
                      class: _normalizeClass(["ti-fw", [_ctx.$style.icon, item.icon]])
                    }))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.item_content)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.item_content_text)
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.item_content_text_title)
                      }, _toDisplayString(item.text), 1 /* TEXT */),
                      (item.caption)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.item_content_text_caption)
                        }, _toDisplayString(item.caption), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ]),
                    (item.indicate)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: _normalizeClass(["_blink", _ctx.$style.indicator])
                      }, [
                        _hoisted_2
                      ]))
                      : _createCommentVNode("v-if", true)
                  ])
                ]))
              : (item.type === 'user')
                ? (_openBlock(), _createElementBlock("button", {
                  key: 6,
                  role: "menuitem",
                  tabindex: "0",
                  class: _normalizeClass(['_button', _ctx.$style.item, { [_ctx.$style.active]: item.active }]),
                  onClick: _cache[5] || (_cache[5] = _withModifiers(($event: any) => (item.active ? close(false) : clicked(item.action, $event)), ["prevent"])),
                  onMouseenterPassive: onItemMouseEnter,
                  onMouseleavePassive: onItemMouseLeave
                }, [
                  _createVNode(_component_MkAvatar, {
                    user: item.user,
                    class: _normalizeClass(_ctx.$style.avatar)
                  }, null, 8 /* PROPS */, ["user"]),
                  _createVNode(_component_MkUserName, { user: item.user }, null, 8 /* PROPS */, ["user"]),
                  (item.indicate)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.item_content)
                    }, [
                      _createElementVNode("span", {
                        class: _normalizeClass(["_blink", _ctx.$style.indicator])
                      }, [
                        _hoisted_3
                      ])
                    ]))
                    : _createCommentVNode("v-if", true)
                ]))
              : (item.type === 'switch')
                ? (_openBlock(), _createElementBlock("button", {
                  key: 7,
                  role: "menuitemcheckbox",
                  tabindex: "0",
                  class: _normalizeClass(['_button', _ctx.$style.item]),
                  disabled: unref(item.disabled),
                  onClick: _cache[6] || (_cache[6] = _withModifiers(($event: any) => (switchItem(item)), ["prevent"])),
                  onMouseenterPassive: onItemMouseEnter,
                  onMouseleavePassive: onItemMouseLeave
                }, [
                  (item.icon)
                    ? (_openBlock(), _createElementBlock("i", {
                      key: 0,
                      class: _normalizeClass(["ti-fw", [_ctx.$style.icon, item.icon]])
                    }))
                    : (_openBlock(), _createBlock(MkSwitchButton, {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.switchButton),
                      checked: item.ref,
                      disabled: item.disabled,
                      onToggle: _cache[7] || (_cache[7] = ($event: any) => (switchItem(item)))
                    }, null, 8 /* PROPS */, ["checked", "disabled"])),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.item_content)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass([_ctx.$style.item_content_text, { [_ctx.$style.switchText]: !item.icon }])
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.item_content_text_title)
                      }, _toDisplayString(item.text), 1 /* TEXT */),
                      (item.caption)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.item_content_text_caption)
                        }, _toDisplayString(item.caption), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ], 2 /* CLASS */),
                    (item.icon)
                      ? (_openBlock(), _createBlock(MkSwitchButton, {
                        key: 0,
                        class: _normalizeClass([_ctx.$style.switchButton, _ctx.$style.caret]),
                        checked: item.ref,
                        disabled: item.disabled,
                        onToggle: _cache[8] || (_cache[8] = ($event: any) => (switchItem(item)))
                      }, null, 8 /* PROPS */, ["checked", "disabled"]))
                      : _createCommentVNode("v-if", true)
                  ])
                ]))
              : (item.type === 'radio')
                ? (_openBlock(), _createElementBlock("button", {
                  key: 8,
                  role: "menuitem",
                  tabindex: "0",
                  class: _normalizeClass(['_button', _ctx.$style.item, _ctx.$style.parent, { [_ctx.$style.active]: childShowingItem.value === item }]),
                  disabled: unref(item.disabled),
                  onMouseenter: _cache[9] || (_cache[9] = _withModifiers(($event: any) => (_unref(preferClick) ? null : showRadioOptions(item, $event)), ["prevent"])),
                  onKeydown: _cache[10] || (_cache[10] = _withKeys(_withModifiers(($event: any) => (_unref(preferClick) ? null : showRadioOptions(item, $event)), ["prevent"]), ["enter"])),
                  onClick: _cache[11] || (_cache[11] = _withModifiers(($event: any) => (!_unref(preferClick) ? null : showRadioOptions(item, $event)), ["prevent"]))
                }, [
                  (item.icon)
                    ? (_openBlock(), _createElementBlock("i", {
                      key: 0,
                      class: _normalizeClass(["ti-fw", [_ctx.$style.icon, item.icon]]),
                      style: "pointer-events: none;"
                    }))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.item_content)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.item_content_text),
                      style: "pointer-events: none;"
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.item_content_text_title)
                      }, _toDisplayString(item.text), 1 /* TEXT */),
                      (item.caption)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.item_content_text_caption)
                        }, _toDisplayString(item.caption), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ]),
                    _createElementVNode("span", {
                      class: _normalizeClass(_ctx.$style.caret),
                      style: "pointer-events: none;"
                    }, [
                      _hoisted_4
                    ])
                  ])
                ]))
              : (item.type === 'radioOption')
                ? (_openBlock(), _createElementBlock("button", {
                  key: 9,
                  role: "menuitemradio",
                  tabindex: "0",
                  class: _normalizeClass(['_button', _ctx.$style.item, _ctx.$style.radio, { [_ctx.$style.active]: unref(item.active) }]),
                  onClick: _cache[12] || (_cache[12] = _withModifiers(($event: any) => (unref(item.active) ? null : clicked(item.action, $event, false)), ["prevent"])),
                  onMouseenterPassive: onItemMouseEnter,
                  onMouseleavePassive: onItemMouseLeave
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.icon)
                  }, [
                    _createElementVNode("span", {
                      class: _normalizeClass([_ctx.$style.radioIcon, { [_ctx.$style.radioChecked]: unref(item.active) }])
                    }, null, 2 /* CLASS */)
                  ]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.item_content)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.item_content_text)
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.item_content_text_title)
                      }, _toDisplayString(item.text), 1 /* TEXT */),
                      (item.caption)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.item_content_text_caption)
                        }, _toDisplayString(item.caption), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ])
                  ])
                ]))
              : (item.type === 'parent')
                ? (_openBlock(), _createElementBlock("button", {
                  key: 10,
                  role: "menuitem",
                  tabindex: "0",
                  class: _normalizeClass(['_button', _ctx.$style.item, _ctx.$style.parent, { [_ctx.$style.active]: childShowingItem.value === item }]),
                  onMouseenter: _cache[13] || (_cache[13] = _withModifiers(($event: any) => (_unref(preferClick) ? null : showChildren(item, $event)), ["prevent"])),
                  onKeydown: _cache[14] || (_cache[14] = _withKeys(_withModifiers(($event: any) => (_unref(preferClick) ? null : showChildren(item, $event)), ["prevent"]), ["enter"])),
                  onClick: _cache[15] || (_cache[15] = _withModifiers(($event: any) => (!_unref(preferClick) ? null : showChildren(item, $event)), ["prevent"]))
                }, [
                  (item.icon)
                    ? (_openBlock(), _createElementBlock("i", {
                      key: 0,
                      class: _normalizeClass(["ti-fw", [_ctx.$style.icon, item.icon]]),
                      style: "pointer-events: none;"
                    }))
                    : _createCommentVNode("v-if", true),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.item_content)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.item_content_text),
                      style: "pointer-events: none;"
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.item_content_text_title)
                      }, _toDisplayString(item.text), 1 /* TEXT */),
                      (item.caption)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.item_content_text_caption)
                        }, _toDisplayString(item.caption), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ]),
                    _createElementVNode("span", {
                      class: _normalizeClass(_ctx.$style.caret),
                      style: "pointer-events: none;"
                    }, [
                      _hoisted_5
                    ])
                  ])
                ]))
              : (_openBlock(), _createElementBlock("button", {
                key: 11,
                role: "menuitem",
                tabindex: "0",
                class: _normalizeClass(['_button', _ctx.$style.item, { [_ctx.$style.danger]: item.danger, [_ctx.$style.active]: unref(item.active) }]),
                onClick: _cache[16] || (_cache[16] = _withModifiers(($event: any) => (unref(item.active) ? close(false) : clicked(item.action, $event)), ["prevent"])),
                onMouseenterPassive: onItemMouseEnter,
                onMouseleavePassive: onItemMouseLeave
              }, [
                (item.icon)
                  ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: _normalizeClass(["ti-fw", [_ctx.$style.icon, item.icon]])
                  }))
                  : _createCommentVNode("v-if", true),
                (item.avatar)
                  ? (_openBlock(), _createBlock(_component_MkAvatar, {
                    key: 0,
                    user: item.avatar,
                    class: _normalizeClass(_ctx.$style.avatar)
                  }, null, 8 /* PROPS */, ["user"]))
                  : _createCommentVNode("v-if", true),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.item_content)
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.item_content_text)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.item_content_text_title)
                    }, _toDisplayString(item.text), 1 /* TEXT */),
                    (item.caption)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.item_content_text_caption)
                      }, _toDisplayString(item.caption), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ]),
                  (item.indicate)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: _normalizeClass(["_blink", _ctx.$style.indicator])
                    }, [
                      _hoisted_6
                    ]))
                    : _createCommentVNode("v-if", true)
                ])
              ]))
          ], 64 /* STABLE_FRAGMENT */))
        }), 256 /* UNKEYED_FRAGMENT */)), (items2.value == null || items2.value.length === 0) ? (_openBlock(), _createElementBlock("span", {
            key: 0,
            tabindex: "-1",
            class: _normalizeClass([_ctx.$style.none, _ctx.$style.item])
          }, [ _createElementVNode("span", null, _toDisplayString(i18n.ts.none), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ], 36 /* STYLE, NEED_HYDRATION */), (childMenu.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createVNode(XChild, {
            ref_key: "child", ref: child,
            items: childMenu.value,
            anchorElement: childTarget.value,
            rootElement: _unref(itemsEl),
            onActioned: childActioned,
            onClosed: closeChild
          }, null, 8 /* PROPS */, ["items", "anchorElement", "rootElement"]) ])) : _createCommentVNode("v-if", true) ], 34 /* CLASS, NEED_HYDRATION */))
}
}

})
