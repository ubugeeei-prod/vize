import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-user" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-external-link" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-usb", style: "font-size: medium;" })
import { ref } from 'vue'
import { toUnicode } from 'punycode.js'
import { query, extractDomain } from '@@/js/url.js'
import { host as configHost } from '@@/js/config.js'
import type { OpenOnRemoteOptions } from '@/utility/please-login.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkInfo from '@/components/MkInfo.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkSignin.input',
  props: {
    message: { type: String, required: false, default: '' },
    openOnRemote: { type: null, required: false, default: undefined },
    initialUsername: { type: String, required: false, default: undefined }
  },
  emits: ["usernameSubmitted", "passkeyClick"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const host = toUnicode(configHost);
const username = ref(props.initialUsername ?? '');
//#region Open on remote
function openRemote(options: OpenOnRemoteOptions, targetHost?: string): void {
	switch (options.type) {
		case 'web':
		case 'lookup': {
			let _path: string;
			if (options.type === 'lookup') {
				// TODO: v2024.7.0以降が浸透してきたら正式なURLに変更する▼
				// _path = `/lookup?uri=${encodeURIComponent(_path)}`;
				_path = `/authorize-follow?acct=${encodeURIComponent(options.url)}`;
			} else {
				_path = options.path;
			}
			if (targetHost) {
				window.open(`https://${targetHost}${_path}`, '_blank', 'noopener');
			} else {
				window.open(`https://misskey-hub.net/mi-web/?path=${encodeURIComponent(_path)}`, '_blank', 'noopener');
			}
			break;
		}
		case 'share': {
			const params = query(options.params);
			if (targetHost) {
				window.open(`https://${targetHost}/share?${params}`, '_blank', 'noopener');
			} else {
				window.open(`https://misskey-hub.net/share/?${params}`, '_blank', 'noopener');
			}
			break;
		}
	}
}
async function specifyHostAndOpenRemote(options: OpenOnRemoteOptions): Promise<void> {
	const { canceled, result: hostTemp } = await os.inputText({
		title: i18n.ts.inputHostName,
		placeholder: 'misskey.example.com',
	});
	if (canceled) return;
	let targetHost: string | null = hostTemp;
	// ドメイン部分だけを取り出す
	targetHost = extractDomain(targetHost ?? '');
	if (targetHost == null) {
		os.alert({
			type: 'error',
			title: i18n.ts.invalidValue,
			text: i18n.ts.tryAgain,
		});
		return;
	}
	openRemote(options, targetHost);
}
//#endregion

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.wrapper),
      "data-cy-signin-page-input": ""
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.root)
      }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.avatar)
        }, [ _hoisted_1 ]), _createTextVNode("\n\n\t\t" + "\n\t\t"), (__props.message) ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(__props.message), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })) : _createCommentVNode("v-if", true), _createTextVNode("\n\n\t\t" + "\n\t\t"), (__props.openOnRemote) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_gaps_m"
          }, [ _createElementVNode("div", { class: "_gaps_s" }, [ _createVNode(MkButton, {
                type: "button",
                rounded: "",
                primary: "",
                style: "margin: 0 auto;",
                onClick: _cache[0] || (_cache[0] = ($event: any) => (openRemote(__props.openOnRemote)))
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.continueOnRemote), 1 /* TEXT */),
                  _createTextVNode(" "),
                  _hoisted_2
                ]),
                _: 1 /* STABLE */
              }), _createElementVNode("button", {
                type: "button",
                class: _normalizeClass(["_button", _ctx.$style.instanceManualSelectButton]),
                onClick: _cache[1] || (_cache[1] = ($event: any) => (specifyHostAndOpenRemote(__props.openOnRemote)))
              }, _toDisplayString(_unref(i18n).ts.specifyServerHost), 1 /* TEXT */) ]), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.orHr)
            }, [ _createElementVNode("p", {
                class: _normalizeClass(_ctx.$style.orMsg)
              }, _toDisplayString(_unref(i18n).ts.or), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n\t\t" + "\n\t\t"), _createElementVNode("form", {
          class: "_gaps_s",
          onSubmit: _cache[2] || (_cache[2] = _withModifiers(($event: any) => (emit('usernameSubmitted', username.value)), ["prevent"]))
        }, [ _createVNode(MkInput, {
            placeholder: _unref(i18n).ts.username,
            type: "text",
            pattern: "^[a-zA-Z0-9_]+$",
            spellcheck: false,
            autocomplete: "username webauthn",
            autofocus: "",
            required: "",
            "data-cy-signin-username": "",
            modelValue: username.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((username).value = $event))
          }, {
            prefix: _withCtx(() => [
              _createTextVNode("@")
            ]),
            suffix: _withCtx(() => [
              _createTextVNode("@" + _toDisplayString(_unref(host)), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["placeholder", "spellcheck", "modelValue"]), _createVNode(MkButton, {
            type: "submit",
            large: "",
            primary: "",
            rounded: "",
            style: "margin: 0 auto;",
            "data-cy-signin-page-input-continue": ""
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.continue), 1 /* TEXT */),
              _createTextVNode(" "),
              _hoisted_3
            ]),
            _: 1 /* STABLE */
          }) ], 32 /* NEED_HYDRATION */), _createTextVNode("\n\n\t\t" + "\n\t\t"), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.orHr)
        }, [ _createElementVNode("p", {
            class: _normalizeClass(_ctx.$style.orMsg)
          }, _toDisplayString(_unref(i18n).ts.or), 1 /* TEXT */) ]), _createElementVNode("div", null, [ _createVNode(MkButton, {
            type: "submit",
            style: "margin: auto auto;",
            large: "",
            rounded: "",
            primary: "",
            gradate: "",
            onClick: _cache[4] || (_cache[4] = ($event: any) => (emit('passkeyClick', $event)))
          }, {
            default: _withCtx(() => [
              _hoisted_4,
              _createTextVNode(_toDisplayString(_unref(i18n).ts.signinWithPasskey), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }) ]) ]) ]))
}
}

})
