import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
import tinycolor from "tinycolor2";
import QRCodeStyling from "qr-code-styling";
import { computed, ref, watch, onMounted, onUnmounted, useTemplateRef } from "vue";
import { url, host } from "@@/js/config.js";
import { instance } from "@/instance.js";
import { ensureSignin } from "@/i.js";
import { userPage, userName } from "@/filters/user.js";
import misskeysvg from "/client-assets/misskey.svg";
import { getStaticImageUrl } from "@/utility/media-proxy.js";
import { i18n } from "@/i18n.js";
export default {
  __name: "qr.show",
  setup(__props) {
    const $i = ensureSignin();
    const acct = computed(() => `@${$i.username}@${host}`);
    const userProfileUrl = computed(() => userPage($i, undefined, true));
    const shareData = computed(() => ({
      title: i18n.tsx._qr.shareTitle({
        name: userName($i),
        acct: acct.value
      }),
      text: i18n.ts._qr.shareText,
      url: userProfileUrl.value
    }));
    const canShare = computed(() => navigator.canShare && navigator.canShare(shareData.value));
    const qrCodeEl = useTemplateRef("qrCodeEl");
    const qrColor = computed(() => tinycolor(instance.themeColor ?? "#86b300"));
    const qrHsl = computed(() => qrColor.value.toHsl());
    function share() {
      if (!canShare.value) return;
      return navigator.share(shareData.value);
    }
    const qrCodeInstance = new QRCodeStyling({
      width: 600,
      height: 600,
      margin: 42,
      type: "canvas",
      data: `${url}/users/${$i.id}`,
      image: instance.iconUrl ? getStaticImageUrl(instance.iconUrl) : "/favicon.ico",
      qrOptions: {
        typeNumber: 0,
        mode: "Byte",
        errorCorrectionLevel: "H"
      },
      imageOptions: {
        hideBackgroundDots: true,
        imageSize: .3,
        margin: 16,
        crossOrigin: "anonymous"
      },
      dotsOptions: {
        type: "dots",
        color: tinycolor(`hsl(${qrHsl.value.h}, 100, 18)`).toRgbString()
      },
      cornersDotOptions: { type: "dot" },
      cornersSquareOptions: { type: "extra-rounded" },
      backgroundOptions: { color: tinycolor(`hsl(${qrHsl.value.h}, 100, 97)`).toRgbString() }
    });
    onMounted(() => {
      if (qrCodeEl.value != null) {
        qrCodeInstance.append(qrCodeEl.value);
      }
    });
    //#region flip
    const THRESHOLD = -3;
    // @ts-expect-error TS(2339)
    const deviceMotionPermissionNeeded = window.DeviceMotionEvent && typeof window.DeviceMotionEvent.requestPermission === "function";
    const flipEls = new Set();
    const flip = ref(false);
    function handleOrientationChange(event) {
      const isUpsideDown = event.beta ? event.beta < THRESHOLD : false;
      flip.value = isUpsideDown;
    }
    watch(flip, (newState) => {
      flipEls.forEach((el) => {
        el.classList.toggle("_qrShowFlipFliped", newState);
      });
    });
    function requestDeviceMotion() {
      if (!deviceMotionPermissionNeeded) return;
      // @ts-expect-error TS(2339)
      window.DeviceMotionEvent.requestPermission().then((response) => {
        if (response === "granted") {
          window.addEventListener("deviceorientation", handleOrientationChange);
        }
      }).catch(console.error);
    }
    onMounted(() => {
      window.addEventListener("deviceorientation", handleOrientationChange);
    });
    onUnmounted(() => {
      window.removeEventListener("deviceorientation", handleOrientationChange);
    });
    const vFlip = {
      mounted(el) {
        flipEls.add(el);
        el.classList.add("_qrShowFlip");
      },
      unmounted(el) {
        el.classList.remove("_qrShowFlip");
        flipEls.delete(el);
      }
    };
    //#endregion
    return (_ctx, _cache) => {
      const _component_MkAvatar = _resolveComponent("MkAvatar");
      const _component_MkUserName = _resolveComponent("MkUserName");
      const _component_MkCondensedLine = _resolveComponent("MkCondensedLine");
      const _directive_flip = vFlip;
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createElementVNode(
          "div",
          { class: _normalizeClass([_ctx.$style.content]) },
          [
            _withDirectives(_createElementVNode(
              "div",
              {
                ref_key: "qrCodeEl",
                ref: qrCodeEl,
                style: _normalizeStyle({ "cursor": canShare.value ? "pointer" : "default" }),
                class: _normalizeClass(_ctx.$style.qr),
                onClick: share
              },
              null,
              6
              /* CLASS, STYLE */
            ), [[_directive_flip]]),
            _withDirectives(_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.user) },
              [_createVNode(_component_MkAvatar, {
                class: _normalizeClass(_ctx.$style.avatar),
                user: _unref($i),
                indicator: false
              }, null, 10, ["user", "indicator"]), _createElementVNode("div", null, [_createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.name) },
                [_createVNode(_component_MkCondensedLine, { minScale: 2 / 3 }, {
                  default: _withCtx(() => [_createVNode(_component_MkUserName, {
                    user: _unref($i),
                    nowrap: true
                  }, null, 8, ["user", "nowrap"])]),
                  _: 1
                }, 8, ["minScale"])],
                2
                /* CLASS */
              ), _createElementVNode("div", null, [_createVNode(_component_MkCondensedLine, { minScale: 2 / 3 }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(acct.value),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["minScale"])])])],
              2
              /* CLASS */
            ), [[_directive_flip]]),
            _unref(deviceMotionPermissionNeeded) ? _withDirectives((_openBlock(), _createElementBlock("img", {
              key: 0,
              class: _normalizeClass(_ctx.$style.logo),
              src: misskeysvg,
              alt: "Misskey Logo",
              onClick: requestDeviceMotion
            }, null, 10, ["src"])), [[_directive_flip]]) : _withDirectives((_openBlock(), _createElementBlock("img", {
              key: 1,
              class: _normalizeClass(_ctx.$style.logo),
              src: misskeysvg,
              alt: "Misskey Logo"
            }, null, 10, ["src"])), [[_directive_flip]])
          ],
          2
          /* CLASS */
        )],
        2
        /* CLASS */
      );
    };
  }
};
