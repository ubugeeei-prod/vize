import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import MkAuthConfirm from '@/components/MkAuthConfirm.vue'
import { i18n } from '@/i18n.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'miauth',
  props: {
    session: { type: String, required: true },
    callback: { type: String, required: false },
    name: { type: String, required: false },
    icon: { type: String, required: false },
    permission: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const _permissions = computed(() => {
	return (props.permission ? props.permission.split(',').filter((p): p is typeof Misskey.permissions[number] => (Misskey.permissions as readonly string[]).includes(p)) : []);
});
const authRoot = useTemplateRef('authRoot');
async function onAccept(token: string) {
	await misskeyApi('miauth/gen-token', {
		session: props.session,
		name: props.name,
		iconUrl: props.icon,
		permission: _permissions.value,
	}, token).then(() => {
		if (props.callback && props.callback !== '') {
			const cbUrl = new URL(props.callback);
			if (['javascript:', 'file:', 'data:', 'mailto:', 'tel:', 'vbscript:'].includes(cbUrl.protocol)) throw new Error('invalid url');
			cbUrl.searchParams.set('session', props.session);
			window.location.href = cbUrl.toString();
		} else {
			authRoot.value?.showUI('success');
		}
	}).catch(() => {
		authRoot.value?.showUI('failed');
	});
}
function onDeny() {
	authRoot.value?.showUI('denied');
}
definePage(() => ({
	title: 'MiAuth',
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
              ref_key: "authRoot", ref: authRoot,
              name: __props.name,
              icon: __props.icon || undefined,
              permissions: _permissions.value,
              onAccept: onAccept,
              onDeny: onDeny
            }, {
              consentAdditionalInfo: _withCtx(() => [
                (__props.callback != null)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass(["_gaps_s", _ctx.$style.redirectRoot])
                  }, [
                    _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._auth.byClickingYouWillBeRedirectedToThisUrl), 1 /* TEXT */),
                    _createElementVNode("div", {
                      class: _normalizeClass(["_monospace", _ctx.$style.redirectUrl])
                    }, _toDisplayString(__props.callback), 1 /* TEXT */)
                  ]))
                  : _createCommentVNode("v-if", true)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["name", "icon", "permissions"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
