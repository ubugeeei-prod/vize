import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"

import tinycolor from 'tinycolor2'
import QRCodeStyling from 'qr-code-styling'
import { computed, ref, shallowRef, watch, onMounted, onUnmounted, useTemplateRef } from 'vue'
import { url, host } from '@@/js/config.js'
import type { Directive } from 'vue'
import { instance } from '@/instance.js'
import { ensureSignin } from '@/i.js'
import { userPage, userName } from '@/filters/user.js'
import misskeysvg from '/client-assets/misskey.svg'
import { getStaticImageUrl } from '@/utility/media-proxy.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'qr.show',
  setup(__props) {

const $i = ensureSignin();
const acct = computed(() => `@${$i.username}@${host}`);
const userProfileUrl = computed(() => userPage($i, undefined, true));
const shareData = computed(() => ({
	title: i18n.tsx._qr.shareTitle({ name: userName($i), acct: acct.value }),
	text: i18n.ts._qr.shareText,
	url: userProfileUrl.value,
}));
const canShare = computed(() => navigator.canShare && navigator.canShare(shareData.value));
const qrCodeEl = useTemplateRef('qrCodeEl');
const qrColor = computed(() => tinycolor(instance.themeColor ?? '#86b300'));
const qrHsl = computed(() => qrColor.value.toHsl());
function share() {
	if (!canShare.value) return;
	return navigator.share(shareData.value);
}
const qrCodeInstance = new QRCodeStyling({
	width: 600,
	height: 600,
	margin: 42,
	type: 'canvas',
	data: `${url}/users/${$i.id}`,
	image: instance.iconUrl ? getStaticImageUrl(instance.iconUrl) : '/favicon.ico',
	qrOptions: {
		typeNumber: 0,
		mode: 'Byte',
		errorCorrectionLevel: 'H',
	},
	imageOptions: {
		hideBackgroundDots: true,
		imageSize: 0.3,
		margin: 16,
		crossOrigin: 'anonymous',
	},
	dotsOptions: {
		type: 'dots',
		color: tinycolor(`hsl(${qrHsl.value.h}, 100, 18)`).toRgbString(),
	},
	cornersDotOptions: {
		type: 'dot',
	},
	cornersSquareOptions: {
		type: 'extra-rounded',
	},
	backgroundOptions: {
		color: tinycolor(`hsl(${qrHsl.value.h}, 100, 97)`).toRgbString(),
	},
});
onMounted(() => {
	if (qrCodeEl.value != null) {
		qrCodeInstance.append(qrCodeEl.value);
	}
});
//#region flip
const THRESHOLD = -3;
// @ts-expect-error TS(2339)
const deviceMotionPermissionNeeded = window.DeviceMotionEvent && typeof window.DeviceMotionEvent.requestPermission === 'function';
const flipEls: Set<Element> = new Set();
const flip = ref(false);
function handleOrientationChange(event: DeviceOrientationEvent) {
	const isUpsideDown = event.beta ? event.beta < THRESHOLD : false;
	flip.value = isUpsideDown;
}
watch(flip, (newState) => {
	flipEls.forEach(el => {
		el.classList.toggle('_qrShowFlipFliped', newState);
	});
});
function requestDeviceMotion() {
	if (!deviceMotionPermissionNeeded) return;
	// @ts-expect-error TS(2339)
	window.DeviceMotionEvent.requestPermission()
		.then((response: string) => {
			if (response === 'granted') {
				window.addEventListener('deviceorientation', handleOrientationChange);
			}
		})
		.catch(console.error);
}
onMounted(() => {
	window.addEventListener('deviceorientation', handleOrientationChange);
});
onUnmounted(() => {
	window.removeEventListener('deviceorientation', handleOrientationChange);
});
const vFlip = {
	mounted(el: Element) {
		flipEls.add(el);
		el.classList.add('_qrShowFlip');
	},
	unmounted(el: Element) {
		el.classList.remove('_qrShowFlip');
		flipEls.delete(el);
	},
} as Directive;
//#endregion

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkCondensedLine = _resolveComponent("MkCondensedLine")
  const _directive_flip = _resolveDirective("flip")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createElementVNode("div", {
        class: _normalizeClass([_ctx.$style.content])
      }, [ _createElementVNode("div", {
          ref_key: "qrCodeEl", ref: qrCodeEl,
          style: _normalizeStyle({
  				'cursor': canShare.value ? 'pointer' : 'default',
  			}),
          class: _normalizeClass(_ctx.$style.qr),
          onClick: share
        }, null, 4 /* STYLE */), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.user)
        }, [ _createVNode(_component_MkAvatar, {
            class: _normalizeClass(_ctx.$style.avatar),
            user: _unref($i),
            indicator: false
          }, null, 8 /* PROPS */, ["user", "indicator"]), _createElementVNode("div", null, [ _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.name)
            }, [ _createVNode(_component_MkCondensedLine, { minScale: 2 / 3 }, {
                default: _withCtx(() => [
                  _createVNode(_component_MkUserName, {
                    user: _unref($i),
                    nowrap: true
                  }, null, 8 /* PROPS */, ["user", "nowrap"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["minScale"]) ]), _createElementVNode("div", null, [ _createVNode(_component_MkCondensedLine, { minScale: 2 / 3 }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(acct.value), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["minScale"]) ]) ]) ]), (_unref(deviceMotionPermissionNeeded)) ? (_openBlock(), _createElementBlock("img", {
            key: 0,
            class: _normalizeClass(_ctx.$style.logo),
            src: misskeysvg,
            alt: "Misskey Logo",
            onClick: requestDeviceMotion
          })) : (_openBlock(), _createElementBlock("img", {
            key: 1,
            class: _normalizeClass(_ctx.$style.logo),
            src: misskeysvg,
            alt: "Misskey Logo"
          })) ]) ]))
}
}

})
