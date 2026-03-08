import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, toHandlers as _toHandlers, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { style: "animation: dev-ticker-blink 2s infinite;" }, "DEV BUILD")
const _hoisted_3 = { style: "animation: dev-ticker-blink 2s infinite;" }
const _hoisted_4 = { style: "animation: dev-ticker-blink 2s infinite;" }
import { defineAsyncComponent, ref, TransitionGroup } from 'vue'
import * as Misskey from 'misskey-js'
import { swInject } from './sw-inject.js'
import XNotification from './notification.vue'
import { isSafeMode } from '@@/js/config.js'
import { popups } from '@/os.js'
import { unisonReload } from '@/utility/unison-reload.js'
import { miLocalStorage } from '@/local-storage.js'
import { pendingApiRequestsCount } from '@/utility/misskey-api.js'
import * as sound from '@/utility/sound.js'
import { $i } from '@/i.js'
import { useStream } from '@/stream.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import { globalEvents } from '@/events.js'
import { store } from '@/store.js'
import XNavbar from '@/ui/_common_/navbar.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'common',
  props: {
    "drawerMenuShowing": {},
    "drawerMenuShowingModifiers": {},
    "widgetsShowing": {},
    "widgetsShowingModifiers": {},
  },
  emits: ["update:drawerMenuShowing", "update:widgetsShowing"],
  setup(__props) {

const drawerMenuShowing = _useModel(__props, "drawerMenuShowing")
const widgetsShowing = _useModel(__props, "widgetsShowing")
const XStreamIndicator = defineAsyncComponent(() => import('./stream-indicator.vue'));
const XWidgets = defineAsyncComponent(() => import('./widgets.vue'));
const dev = _DEV_;
const notifications = ref<Misskey.entities.Notification[]>([]);
function onNotification(notification: Misskey.entities.Notification, isClient = false) {
	if (window.document.visibilityState === 'visible') {
		if (!isClient && notification.type !== 'test') {
			// サーバーサイドのテスト通知の際は自動で既読をつけない（テストできないので）
			if (store.s.realtimeMode) {
				useStream().send('readNotification');
			}
		}
		notifications.value.unshift(notification);
		window.setTimeout(() => {
			if (notifications.value.length > 3) notifications.value.pop();
		}, 500);
		window.setTimeout(() => {
			notifications.value = notifications.value.filter(x => x.id !== notification.id);
		}, 6000);
	}
	sound.playMisskeySfx('notification');
}
function exitSafeMode() {
	miLocalStorage.removeItem('isSafeMode');
	const url = new URL(window.location.href);
	url.searchParams.delete('safemode');
	unisonReload(url.toString());
}
if ($i) {
	if (store.s.realtimeMode) {
		const connection = useStream().useChannel('main');
		connection.on('notification', onNotification);
	}
	globalEvents.on('clientNotification', notification => onNotification(notification, true));
	if ('serviceWorker' in navigator) {
		swInject();
	}
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createVNode(_Transition, {
        enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_menuDrawerBg_enterActive : '',
        leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_menuDrawerBg_leaveActive : '',
        enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_menuDrawerBg_enterFrom : '',
        leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_menuDrawerBg_leaveTo : ''
      }, {
        default: _withCtx(() => [
          (drawerMenuShowing.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(["_modalBg", _ctx.$style.menuDrawerBg]),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (drawerMenuShowing.value = false)),
              onTouchstartPassive: _cache[1] || (_cache[1] = ($event: any) => (drawerMenuShowing.value = false))
            }))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]), _createVNode(_Transition, {
        enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_menuDrawer_enterActive : '',
        leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_menuDrawer_leaveActive : '',
        enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_menuDrawer_enterFrom : '',
        leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_menuDrawer_leaveTo : ''
      }, {
        default: _withCtx(() => [
          (drawerMenuShowing.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.menuDrawer)
            }, [
              _createVNode(XNavbar, {
                style: "height: 100%;",
                asDrawer: true,
                showWidgetButton: false
              }, null, 8 /* PROPS */, ["asDrawer", "showWidgetButton"])
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]), _createVNode(_Transition, {
        enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_widgetsDrawerBg_enterActive : '',
        leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_widgetsDrawerBg_leaveActive : '',
        enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_widgetsDrawerBg_enterFrom : '',
        leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_widgetsDrawerBg_leaveTo : ''
      }, {
        default: _withCtx(() => [
          (widgetsShowing.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(["_modalBg", _ctx.$style.widgetsDrawerBg]),
              onClick: _cache[2] || (_cache[2] = ($event: any) => (widgetsShowing.value = false)),
              onTouchstartPassive: _cache[3] || (_cache[3] = ($event: any) => (widgetsShowing.value = false))
            }))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]), _createVNode(_Transition, {
        enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_widgetsDrawer_enterActive : '',
        leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_widgetsDrawer_leaveActive : '',
        enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_widgetsDrawer_enterFrom : '',
        leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_widgetsDrawer_leaveTo : ''
      }, {
        default: _withCtx(() => [
          (widgetsShowing.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.widgetsDrawer)
            }, [
              _createElementVNode("button", {
                class: _normalizeClass(["_button", _ctx.$style.widgetsCloseButton]),
                onClick: _cache[4] || (_cache[4] = ($event: any) => (widgetsShowing.value = false))
              }, [
                _hoisted_1
              ]),
              _createVNode(XWidgets)
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(popups), (popup) => {
        return (_openBlock(), _createBlock(_resolveDynamicComponent(popup.component), _mergeProps(popup.props, _toHandlers(popup.events, true), {
          key: popup.id,
          is: popup.component
        }), null, 16 /* FULL_PROPS */, ["is"]))
      }), 128 /* KEYED_FRAGMENT */)), _createVNode(_resolveDynamicComponent(_unref(prefer).s.animation ? _unref(TransitionGroup) : 'div'), {
        tag: "div",
        class: _normalizeClass([_ctx.$style.notifications, {
  		[_ctx.$style.notificationsPosition_leftTop]: _unref(prefer).s.notificationPosition === 'leftTop',
  		[_ctx.$style.notificationsPosition_leftBottom]: _unref(prefer).s.notificationPosition === 'leftBottom',
  		[_ctx.$style.notificationsPosition_rightTop]: _unref(prefer).s.notificationPosition === 'rightTop',
  		[_ctx.$style.notificationsPosition_rightBottom]: _unref(prefer).s.notificationPosition === 'rightBottom',
  		[_ctx.$style.notificationsStackAxis_vertical]: _unref(prefer).s.notificationStackAxis === 'vertical',
  		[_ctx.$style.notificationsStackAxis_horizontal]: _unref(prefer).s.notificationStackAxis === 'horizontal',
  	}]),
        moveClass: _ctx.$style.transition_notification_move,
        enterActiveClass: _ctx.$style.transition_notification_enterActive,
        leaveActiveClass: _ctx.$style.transition_notification_leaveActive,
        enterFromClass: _ctx.$style.transition_notification_enterFrom,
        leaveToClass: _ctx.$style.transition_notification_leaveTo
      }, {
        default: _withCtx(() => [
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(notifications.value, (notification) => {
            return (_openBlock(), _createElementBlock("div", {
              key: notification.id,
              class: _normalizeClass(_ctx.$style.notification)
            }, [
              _createVNode(XNotification, { notification: notification }, null, 8 /* PROPS */, ["notification"])
            ]))
          }), 128 /* KEYED_FRAGMENT */))
        ]),
        _: 1 /* STABLE */
      }, 10 /* CLASS, PROPS */, ["moveClass", "enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]), _createVNode(XStreamIndicator), (_unref(pendingApiRequestsCount) > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          id: "wait"
        })) : _createCommentVNode("v-if", true), (_unref(dev)) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          id: "devTicker"
        }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true), (_unref($i) && _unref($i).isBot) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          id: "botWarn"
        }, [ _createElementVNode("span", _hoisted_3, _toDisplayString(_unref(i18n).ts.loggedInAsBot), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), (_unref(isSafeMode)) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          id: "safemodeWarn"
        }, [ _createElementVNode("span", _hoisted_4, _toDisplayString(_unref(i18n).ts.safeModeEnabled), 1 /* TEXT */), _createTextVNode("&nbsp;\n\t"), _createElementVNode("button", {
            class: "_textButton",
            style: "pointer-events: all;",
            onClick: exitSafeMode
          }, _toDisplayString(_unref(i18n).ts.turnItOff), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
