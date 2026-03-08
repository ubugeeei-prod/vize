import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { ref } from 'vue'
import { instanceName } from '@@/js/config.js'
import { $i } from '@/i.js'
import MkButton from '@/components/MkButton.vue'
import { instance } from '@/instance.js'
import { apiWithDialog, promiseDialog, alert } from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { getAccounts } from '@/accounts.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPushNotificationAllowButton',
  props: {
    primary: { type: Boolean, required: false },
    gradate: { type: Boolean, required: false },
    rounded: { type: Boolean, required: false },
    inline: { type: Boolean, required: false },
    link: { type: Boolean, required: false },
    to: { type: String, required: false },
    autofocus: { type: Boolean, required: false },
    wait: { type: Boolean, required: false },
    danger: { type: Boolean, required: false },
    full: { type: Boolean, required: false },
    showOnlyToRegister: { type: Boolean, required: false }
  },
  setup(__props: any, { expose: __expose }) {

// ServiceWorker registration
const registration = ref<ServiceWorkerRegistration | undefined>();
// If this browser supports push notification
const supported = ref(false);
// If this browser has already subscribed to push notification
const pushSubscription = ref<PushSubscription | null>(null);
const pushRegistrationInServer = ref<{ state?: string; key?: string; userId: string; endpoint: string; sendReadMessage: boolean; } | undefined>();
async function subscribe() {
	if (!registration.value || !supported.value || !instance.swPublickey) return;
	if ('Notification' in window) {
		let permission = Notification.permission;
		if (Notification.permission === 'default') {
			permission = await promiseDialog(Notification.requestPermission(), null, null, i18n.ts.pleaseAllowPushNotification);
		}
		if (permission !== 'granted') {
			alert({
				type: 'error',
				title: i18n.ts.browserPushNotificationDisabled,
				text: i18n.tsx.browserPushNotificationDisabledDescription({ serverName: instanceName }),
			});
			return;
		}
	}
	// SEE: https://developer.mozilla.org/en-US/docs/Web/API/PushManager/subscribe#Parameters
	await promiseDialog(registration.value.pushManager.subscribe({
		userVisibleOnly: true,
		applicationServerKey: urlBase64ToUint8Array(instance.swPublickey),
	})
		.then(async subscription => {
			pushSubscription.value = subscription;
			// Register
			pushRegistrationInServer.value = await misskeyApi('sw/register', {
				endpoint: subscription.endpoint,
				auth: encode(subscription.getKey('auth')),
				publickey: encode(subscription.getKey('p256dh')),
			});
		}, async err => { // When subscribe failed
			// 通知が許可されていなかったとき
			if (err?.name === 'NotAllowedError') {
				console.info('User denied the notification permission request.');
				return;
			}
			// 違うapplicationServerKey (または gcm_sender_id)のサブスクリプションが
			// 既に存在していることが原因でエラーになった可能性があるので、
			// そのサブスクリプションを解除しておく
			// （これは実行されなさそうだけど、おまじない的に古い実装から残してある）
			await unsubscribe();
		}), null, null);
}
async function unsubscribe() {
	if (!pushSubscription.value) return;
	const endpoint = pushSubscription.value.endpoint;
	const accounts = await getAccounts();
	pushRegistrationInServer.value = undefined;
	if ($i && accounts.length >= 2) {
		apiWithDialog('sw/unregister', {
			endpoint,
		}, $i.token);
	} else {
		pushSubscription.value.unsubscribe();
		apiWithDialog('sw/unregister', {
			endpoint,
		}, null);
		pushSubscription.value = null;
	}
}
function encode(buffer: ArrayBuffer | null) {
	return btoa(String.fromCharCode(...(buffer != null ? new Uint8Array(buffer) : [])));
}
/**
 * Convert the URL safe base64 string to a Uint8Array
 * @param base64String base64 string
 */
function urlBase64ToUint8Array(base64String: string): BufferSource {
	const padding = '='.repeat((4 - base64String.length % 4) % 4);
	const base64 = (base64String + padding)
		.replace(/-/g, '+')
		.replace(/_/g, '/');
	const rawData = window.atob(base64);
	const outputArray = new Uint8Array(rawData.length);
	for (let i = 0; i < rawData.length; ++i) {
		outputArray[i] = rawData.charCodeAt(i);
	}
	return outputArray;
}
if (navigator.serviceWorker == null) {
	// TODO: よしなに？
} else {
	navigator.serviceWorker.ready.then(async swr => {
		registration.value = swr;
		pushSubscription.value = await registration.value.pushManager.getSubscription();
		if (instance.swPublickey && ('PushManager' in window) && $i && $i.token) {
			supported.value = true;
			if (pushSubscription.value) {
				const res = await misskeyApi('sw/show-registration', {
					endpoint: pushSubscription.value.endpoint,
				});
				if (res) {
					pushRegistrationInServer.value = res;
				}
			}
		}
	});
}
__expose({
	pushRegistrationInServer: pushRegistrationInServer,
})

return (_ctx: any,_cache: any) => {
  return (supported.value && !pushRegistrationInServer.value)
      ? (_openBlock(), _createBlock(MkButton, {
        key: 0,
        type: "button",
        primary: "",
        gradate: __props.gradate,
        rounded: __props.rounded,
        inline: __props.inline,
        autofocus: __props.autofocus,
        wait: __props.wait,
        full: __props.full,
        onClick: subscribe
      }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.subscribePushNotification), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["gradate", "rounded", "inline", "autofocus", "wait", "full"]))
      : (!__props.showOnlyToRegister && (_unref($i) ? pushRegistrationInServer.value : pushSubscription.value))
        ? (_openBlock(), _createBlock(MkButton, {
          key: 1,
          type: "button",
          primary: false,
          gradate: __props.gradate,
          rounded: __props.rounded,
          inline: __props.inline,
          autofocus: __props.autofocus,
          wait: __props.wait,
          full: __props.full,
          onClick: unsubscribe
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.unsubscribePushNotification), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["primary", "gradate", "rounded", "inline", "autofocus", "wait", "full"]))
      : (_unref($i) && pushRegistrationInServer.value)
        ? (_openBlock(), _createBlock(MkButton, {
          key: 2,
          disabled: "",
          rounded: __props.rounded,
          inline: __props.inline,
          wait: __props.wait,
          full: __props.full
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.pushNotificationAlreadySubscribed), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["rounded", "inline", "wait", "full"]))
      : (!supported.value)
        ? (_openBlock(), _createBlock(MkButton, {
          key: 3,
          disabled: "",
          rounded: __props.rounded,
          inline: __props.inline,
          wait: __props.wait,
          full: __props.full
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.pushNotificationNotSupported), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["rounded", "inline", "wait", "full"]))
      : _createCommentVNode("v-if", true)
}
}

})
