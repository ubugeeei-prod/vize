import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { onMounted, ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import XForm from './auth.form.vue'
import MkSignin from '@/components/MkSignin.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { $i } from '@/i.js'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'
import { login } from '@/accounts.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'auth',
  props: {
    token: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const state = ref<'waiting' | 'accepted' | 'fetch-session-error' | 'denied' | null>(null);
const session = ref<Misskey.entities.AuthSessionShowResponse | null>(null);
function accepted() {
	state.value = 'accepted';
	if (session.value && session.value.app.callbackUrl) {
		const url = new URL(session.value.app.callbackUrl);
		if (['javascript:', 'file:', 'data:', 'mailto:', 'tel:', 'vbscript:'].includes(url.protocol)) throw new Error('invalid url');
		window.location.href = `${session.value.app.callbackUrl}?token=${session.value.token}`;
	}
}
function onLogin(res: Misskey.entities.SigninFlowResponse & { finished: true }) {
	login(res.i);
}
onMounted(async () => {
	if (!$i) return;
	try {
		const result = await misskeyApi('auth/session/show', {
			token: props.token,
		});
		session.value = result;
		// 既に連携していた場合
		if (result.app.isAuthorized) {
			await misskeyApi('auth/accept', {
				token: result.token,
			});
			accepted();
		} else {
			state.value = 'waiting';
		}
	} catch (err) {
		state.value = 'fetch-session-error';
	}
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts._auth.shareAccessTitle,
	icon: 'ti ti-apps',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 500px;"
        }, [
          (state.value == 'fetch-session-error')
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              _createElementVNode("p", null, _toDisplayString(_unref(i18n).ts.somethingHappened), 1 /* TEXT */)
            ]))
            : (_unref($i) && !session.value)
              ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
                _createVNode(_component_MkLoading)
              ]))
            : (_unref($i) && session.value)
              ? (_openBlock(), _createElementBlock("div", { key: 2 }, [
                (state.value == 'waiting')
                  ? (_openBlock(), _createBlock(XForm, {
                    key: 0,
                    class: "form",
                    session: session.value,
                    onDenied: _cache[0] || (_cache[0] = ($event: any) => (state.value = 'denied')),
                    onAccepted: accepted
                  }, null, 8 /* PROPS */, ["session"]))
                  : _createCommentVNode("v-if", true),
                (state.value == 'denied')
                  ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                    _createElementVNode("h1", null, _toDisplayString(_unref(i18n).ts._auth.denied), 1 /* TEXT */)
                  ]))
                  : _createCommentVNode("v-if", true),
                (state.value == 'accepted' && session.value)
                  ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                    _createElementVNode("h1", null, _toDisplayString(session.value.app.isAuthorized ? _unref(i18n).ts._auth.alreadyAuthorized : _unref(i18n).ts._auth.accepted), 1 /* TEXT */),
                    (session.value.app.callbackUrl)
                      ? (_openBlock(), _createElementBlock("p", { key: 0 }, [
                        _toDisplayString(_unref(i18n).ts._auth.callback),
                        _createTextVNode("\n\t\t\t\t\t"),
                        _createVNode(_component_MkEllipsis)
                      ]))
                      : _createCommentVNode("v-if", true),
                    (!session.value.app.callbackUrl)
                      ? (_openBlock(), _createElementBlock("p", { key: 0 }, _toDisplayString(_unref(i18n).ts._auth.pleaseGoBack), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ]))
                  : _createCommentVNode("v-if", true)
              ]))
            : (_openBlock(), _createElementBlock("div", { key: 3 }, [
              _createElementVNode("p", {
                class: _normalizeClass(_ctx.$style.loginMessage)
              }, _toDisplayString(_unref(i18n).ts._auth.pleaseLogin), 1 /* TEXT */),
              _createVNode(MkSignin, { onLogin: onLogin })
            ]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
