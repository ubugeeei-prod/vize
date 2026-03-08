import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import { definePage } from '@/page.js'
import MkAuthConfirm from '@/components/MkAuthConfirm.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'oauth',
  setup(__props) {

const transactionIdMeta = window.document.querySelector<HTMLMetaElement>('meta[name="misskey:oauth:transaction-id"]');
if (transactionIdMeta) {
	transactionIdMeta.remove();
}
const name = window.document.querySelector<HTMLMetaElement>('meta[name="misskey:oauth:client-name"]')?.content;
const logo = window.document.querySelector<HTMLMetaElement>('meta[name="misskey:oauth:client-logo"]')?.content;
const permissions = window.document.querySelector<HTMLMetaElement>('meta[name="misskey:oauth:scope"]')?.content.split(' ').filter((p): p is typeof Misskey.permissions[number] => (Misskey.permissions as readonly string[]).includes(p)) ?? [];
function doPost(token: string, decision: 'accept' | 'deny') {
	const form = window.document.createElement('form');
	form.action = '/oauth/decision';
	form.method = 'post';
	form.acceptCharset = 'utf-8';
	const loginToken = window.document.createElement('input');
	loginToken.type = 'hidden';
	loginToken.name = 'login_token';
	loginToken.value = token;
	form.appendChild(loginToken);
	const transactionId = window.document.createElement('input');
	transactionId.type = 'hidden';
	transactionId.name = 'transaction_id';
	transactionId.value = transactionIdMeta?.content ?? '';
	form.appendChild(transactionId);
	if (decision === 'deny') {
		const cancel = window.document.createElement('input');
		cancel.type = 'hidden';
		cancel.name = 'cancel';
		cancel.value = 'cancel';
		form.appendChild(cancel);
	}
	window.document.body.appendChild(form);
	form.submit();
}
function onAccept(token: string) {
	doPost(token, 'accept');
}
function onDeny(token: string) {
	doPost(token, 'deny');
}
definePage(() => ({
	title: 'OAuth',
	icon: 'ti ti-apps',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithAnimBg = _resolveComponent("PageWithAnimBg")

  return (_openBlock(), _createBlock(_component_PageWithAnimBg, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.formContainer)
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.form)
          }, [
            _createVNode(MkAuthConfirm, {
              ref: "authRoot",
              name: _unref(name),
              icon: _unref(logo),
              permissions: _unref(permissions),
              waitOnDeny: true,
              onAccept: onAccept,
              onDeny: onDeny
            }, null, 8 /* PROPS */, ["name", "icon", "permissions", "waitOnDeny"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
