import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-caret-down" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings-2" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-caret-down" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings-2" })
import { defineAsyncComponent, ref, useTemplateRef } from 'vue'
import XCommon from './_common_/common.vue'
import { genId } from '@/utility/id.js'
import XSidebar from '@/ui/_common_/navbar.vue'
import XNavbarH from '@/ui/_common_/navbar-h.vue'
import XMobileFooterMenu from '@/ui/_common_/mobile-footer-menu.vue'
import XTitlebar from '@/ui/_common_/titlebar.vue'
import XPreferenceRestore from '@/ui/_common_/PreferenceRestore.vue'
import XReloadSuggestion from '@/ui/_common_/ReloadSuggestion.vue'
import * as os from '@/os.js'
import { $i } from '@/i.js'
import { i18n } from '@/i18n.js'
import { deviceKind } from '@/utility/device-kind.js'
import { prefer } from '@/preferences.js'
import { store } from '@/store.js'
import XMainColumn from '@/ui/deck/main-column.vue'
import XTlColumn from '@/ui/deck/tl-column.vue'
import XAntennaColumn from '@/ui/deck/antenna-column.vue'
import XListColumn from '@/ui/deck/list-column.vue'
import XChannelColumn from '@/ui/deck/channel-column.vue'
import XNotificationsColumn from '@/ui/deck/notifications-column.vue'
import XWidgetsColumn from '@/ui/deck/widgets-column.vue'
import XMentionsColumn from '@/ui/deck/mentions-column.vue'
import XDirectColumn from '@/ui/deck/direct-column.vue'
import XRoleTimelineColumn from '@/ui/deck/role-timeline-column.vue'
import XChatColumn from '@/ui/deck/chat-column.vue'
import MkInfo from '@/components/MkInfo.vue'
import { mainRouter } from '@/router.js'
import { columns, layout, columnTypes, switchProfileMenu, addColumn as addColumnToStore, deleteProfile as deleteProfile_ } from '@/deck.js'
import { shouldSuggestRestoreBackup } from '@/preferences/utility.js'
import { shouldSuggestReload } from '@/utility/reload-suggest.js'
import { startTour } from '@/utility/tour.js'
import { closeTip } from '@/tips.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'deck',
  setup(__props) {

const XStatusBars = defineAsyncComponent(() => import('@/ui/_common_/statusbars.vue'));
const XAnnouncements = defineAsyncComponent(() => import('@/ui/_common_/announcements.vue'));
const columnComponents = {
	main: XMainColumn,
	widgets: XWidgetsColumn,
	notifications: XNotificationsColumn,
	tl: XTlColumn,
	list: XListColumn,
	channel: XChannelColumn,
	antenna: XAntennaColumn,
	mentions: XMentionsColumn,
	direct: XDirectColumn,
	roleTimeline: XRoleTimelineColumn,
	chat: XChatColumn,
};
mainRouter.navHook = (path, flag): boolean => {
	if (flag === 'forcePage') return false;
	const noMainColumn = !columns.value.some(x => x.type === 'main');
	if (prefer.s['deck.navWindow'] || noMainColumn) {
		os.pageWindow(path);
		return true;
	}
	return false;
};
const isMobile = ref(window.innerWidth <= 500);
window.addEventListener('resize', () => {
	isMobile.value = window.innerWidth <= 500;
});
// ポインターイベント非対応用に初期値はUAから出す
const snapScroll = ref(deviceKind === 'smartphone' || deviceKind === 'tablet');
const withWallpaper = prefer.s['deck.wallpaper'] != null;
const drawerMenuShowing = ref(false);
const widgetsShowing = ref(false);
const gap = prefer.r['deck.columnGap'];
/*
const route = 'TODO';
watch(route, () => {
	drawerMenuShowing.value = false;
});
*/
function showSettings() {
	os.pageWindow('/settings/deck');
}
const columnsEl = useTemplateRef('columnsEl');
const addColumnButtonEl = useTemplateRef('addColumnButtonEl');
const settingsButtonEl = useTemplateRef('settingsButtonEl');
const swicthProfileButtonEl = useTemplateRef('swicthProfileButtonEl');
async function addColumn(ev: PointerEvent) {
	const { canceled, result: column } = await os.select({
		title: i18n.ts._deck.addColumn,
		items: columnTypes.filter(column => column !== 'chat' || $i == null || $i.policies.chatAvailability !== 'unavailable').map(column => ({
			value: column, label: i18n.ts._deck._columns[column],
		})),
	});
	if (canceled || column == null) return;
	addColumnToStore({
		type: column,
		id: genId(),
		name: null,
		width: 330,
		soundSetting: { type: null, volume: 1 },
	});
}
function onContextmenu(ev: PointerEvent) {
	os.contextMenu([{
		text: i18n.ts._deck.addColumn,
		action: addColumn,
	}], ev);
}
// タッチでスクロールしてるときはスナップスクロールを有効にする
function pointerEvent(ev: PointerEvent) {
	snapScroll.value = ev.pointerType === 'touch';
}
window.document.addEventListener('pointerdown', pointerEvent, { passive: true });
function onWheel(ev: WheelEvent) {
	// WheelEvent はマウスからしか発火しないのでスナップスクロールは無効化する
	snapScroll.value = false;
	if (ev.deltaX === 0 && columnsEl.value != null) {
		columnsEl.value.scrollLeft += ev.deltaY;
	}
}
async function deleteProfile() {
	if (prefer.s['deck.profile'] == null) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.deleteAreYouSure({ x: prefer.s['deck.profile'] }),
	});
	if (canceled) return;
	await deleteProfile_(prefer.s['deck.profile']);
	os.success();
}
function showTour() {
	if (addColumnButtonEl.value == null ||
		settingsButtonEl.value == null ||
		swicthProfileButtonEl.value == null) {
		return;
	}
	startTour([{
		element: addColumnButtonEl.value,
		title: i18n.ts._deck._howToUse.addColumn_title,
		description: i18n.ts._deck._howToUse.addColumn_description,
	}, {
		element: settingsButtonEl.value,
		title: i18n.ts._deck._howToUse.settings_title,
		description: i18n.ts._deck._howToUse.settings_description,
	}, {
		element: swicthProfileButtonEl.value,
		title: i18n.ts._deck._howToUse.switchProfile_title,
		description: i18n.ts._deck._howToUse.switchProfile_description,
	}]).then(() => {
		closeTip('deck');
	});
}
window.document.documentElement.style.overflowY = 'hidden';
window.document.documentElement.style.scrollBehavior = 'auto';

return (_ctx: any,_cache: any) => {
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root])
    }, [ (_unref(prefer).r.showTitlebar.value) ? (_openBlock(), _createBlock(XTitlebar, {
          key: 0,
          style: "flex-shrink: 0;"
        })) : _createCommentVNode("v-if", true), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.nonTitlebarArea)
      }, [ (!isMobile.value && _unref(prefer).r['deck.navbarPosition'].value === 'left') ? (_openBlock(), _createBlock(XSidebar, { key: 0 })) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          class: _normalizeClass([_ctx.$style.main, { [_ctx.$style.withWallpaper]: _unref(withWallpaper), [_ctx.$style.withSidebarAndTitlebar]: !isMobile.value && _unref(prefer).r['deck.navbarPosition'].value === 'left' && _unref(prefer).r.showTitlebar.value }]),
          style: _normalizeStyle({ backgroundImage: _unref(prefer).s['deck.wallpaper'] != null ? `url(${ _unref(prefer).s['deck.wallpaper'] })` : '' })
        }, [ (!isMobile.value && _unref(prefer).r['deck.navbarPosition'].value === 'top') ? (_openBlock(), _createBlock(XNavbarH, {
              key: 0,
              acrylic: _unref(withWallpaper)
            }, null, 8 /* PROPS */, ["acrylic"])) : _createCommentVNode("v-if", true), (_unref(shouldSuggestReload)) ? (_openBlock(), _createBlock(XReloadSuggestion, { key: 0 })) : _createCommentVNode("v-if", true), (_unref(shouldSuggestRestoreBackup)) ? (_openBlock(), _createBlock(XPreferenceRestore, { key: 0 })) : _createCommentVNode("v-if", true), (_unref($i)) ? (_openBlock(), _createBlock(XAnnouncements, { key: 0 })) : _createCommentVNode("v-if", true), _createVNode(XStatusBars), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.columnsWrapper)
          }, [ _createElementVNode("div", {
              ref_key: "columnsEl", ref: columnsEl,
              class: _normalizeClass([_ctx.$style.columns, { [_ctx.$style.center]: _unref(prefer).r['deck.columnAlign'].value === 'center', [_ctx.$style.snapScroll]: snapScroll.value }]),
              onContextmenu: _withModifiers(onContextmenu, ["self","prevent"]),
              onWheelPassive: _withModifiers(onWheel, ["self"])
            }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(layout), (ids) => {
                return (_openBlock(), _createElementBlock("section", { class: _normalizeClass(_ctx.$style.section), style: _normalizeStyle(_unref(columns).filter(c => ids.includes(c.id)).some(c => c.flexible) ? { flex: 1, minWidth: '350px' } : { width: Math.max(..._unref(columns).filter(c => ids.includes(c.id)).map(c => c.width)) + 'px' }), onWheelPassive: _withModifiers(onWheel, ["self"]) }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(ids, (id) => {
                    return (_openBlock(), _createBlock(_resolveDynamicComponent(columnComponents[_unref(columns).find((c) => c.id === id).type] ?? XTlColumn), {
                      key: id,
                      is: columnComponents[_unref(columns).find((c) => c.id === id).type] ?? XTlColumn,
                      ref: id,
                      class: _normalizeClass({ '_shadow': _unref(withWallpaper) }),
                      column: _unref(columns).find((c) => c.id === id),
                      isStacked: ids.length > 1,
                      onHeaderWheel: onWheel
                    }, null, 522 /* CLASS, PROPS, NEED_PATCH */, ["is", "column", "isStacked"]))
                  }), 128 /* KEYED_FRAGMENT */))
                ], 36 /* STYLE, NEED_HYDRATION */))
              }), 256 /* UNKEYED_FRAGMENT */)), (_unref(layout).length === 0) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(["_panel _gaps", _ctx.$style.onboarding])
                }, [ _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._deck.introduction), 1 /* TEXT */), _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._deck.introduction2), 1 /* TEXT */), (!_unref(store).r.tips.value.deck) ? (_openBlock(), _createBlock(MkInfo, {
                      key: 0,
                      closable: "",
                      onClose: _cache[0] || (_cache[0] = ($event: any) => (_unref(closeTip)('deck')))
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("button", {
                          class: "_textButton",
                          onClick: showTour
                        }, _toDisplayString(_unref(i18n).ts._deck.showHowToUse), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ], 34 /* CLASS, NEED_HYDRATION */), (_unref(prefer).r['deck.menuPosition'].value === 'right') ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.sideMenu)
              }, [ _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.sideMenuTop)
                }, [ _createElementVNode("button", {
                    ref_key: "swicthProfileButtonEl", ref: swicthProfileButtonEl,
                    class: _normalizeClass(["_button", _ctx.$style.sideMenuButton]),
                    onClick: _cache[1] || (_cache[1] = (...args) => (switchProfileMenu && switchProfileMenu(...args)))
                  }, [ _hoisted_1 ], 512 /* NEED_PATCH */), _createElementVNode("button", {
                    class: _normalizeClass(["_button", _ctx.$style.sideMenuButton]),
                    onClick: deleteProfile
                  }, [ _hoisted_2 ]) ]), _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.sideMenuMiddle)
                }, [ _createElementVNode("button", {
                    ref_key: "addColumnButtonEl", ref: addColumnButtonEl,
                    class: _normalizeClass(["_button", _ctx.$style.sideMenuButton]),
                    onClick: addColumn
                  }, [ _hoisted_3 ], 512 /* NEED_PATCH */) ]), _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.sideMenuBottom)
                }, [ _createElementVNode("button", {
                    ref_key: "settingsButtonEl", ref: settingsButtonEl,
                    class: _normalizeClass(["_button", _ctx.$style.sideMenuButton]),
                    onClick: showSettings
                  }, [ _hoisted_4 ], 512 /* NEED_PATCH */) ]) ])) : _createCommentVNode("v-if", true) ]), (_unref(prefer).r['deck.menuPosition'].value === 'bottom') ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.bottomMenu)
            }, [ _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.bottomMenuLeft)
              }, [ _createElementVNode("button", {
                  ref_key: "swicthProfileButtonEl", ref: swicthProfileButtonEl,
                  class: _normalizeClass(["_button", _ctx.$style.bottomMenuButton]),
                  onClick: _cache[2] || (_cache[2] = (...args) => (switchProfileMenu && switchProfileMenu(...args)))
                }, [ _hoisted_5 ], 512 /* NEED_PATCH */), _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.bottomMenuButton]),
                  onClick: deleteProfile
                }, [ _hoisted_6 ]) ]), _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.bottomMenuMiddle)
              }, [ _createElementVNode("button", {
                  ref_key: "addColumnButtonEl", ref: addColumnButtonEl,
                  class: _normalizeClass(["_button", _ctx.$style.bottomMenuButton]),
                  onClick: addColumn
                }, [ _hoisted_7 ], 512 /* NEED_PATCH */) ]), _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.bottomMenuRight)
              }, [ _createElementVNode("button", {
                  ref_key: "settingsButtonEl", ref: settingsButtonEl,
                  class: _normalizeClass(["_button", _ctx.$style.bottomMenuButton]),
                  onClick: showSettings
                }, [ _hoisted_8 ], 512 /* NEED_PATCH */) ]) ])) : _createCommentVNode("v-if", true), (!isMobile.value && _unref(prefer).r['deck.navbarPosition'].value === 'bottom') ? (_openBlock(), _createBlock(XNavbarH, {
              key: 0,
              acrylic: _unref(withWallpaper)
            }, null, 8 /* PROPS */, ["acrylic"])) : _createCommentVNode("v-if", true), (isMobile.value) ? (_openBlock(), _createBlock(XMobileFooterMenu, {
              key: 0,
              drawerMenuShowing: drawerMenuShowing.value,
              "onUpdate:drawerMenuShowing": _cache[3] || (_cache[3] = ($event: any) => ((drawerMenuShowing).value = $event)),
              widgetsShowing: widgetsShowing.value,
              "onUpdate:widgetsShowing": _cache[4] || (_cache[4] = ($event: any) => ((widgetsShowing).value = $event))
            }, null, 8 /* PROPS */, ["drawerMenuShowing", "widgetsShowing"])) : _createCommentVNode("v-if", true) ], 6 /* CLASS, STYLE */) ]), _createVNode(XCommon, {
        drawerMenuShowing: drawerMenuShowing.value,
        "onUpdate:drawerMenuShowing": _cache[5] || (_cache[5] = ($event: any) => ((drawerMenuShowing).value = $event)),
        widgetsShowing: widgetsShowing.value,
        "onUpdate:widgetsShowing": _cache[6] || (_cache[6] = ($event: any) => ((widgetsShowing).value = $event))
      }, null, 8 /* PROPS */, ["drawerMenuShowing", "widgetsShowing"]) ]))
}
}

})
