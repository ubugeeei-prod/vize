import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'auth.form',
  props: {
    session: { type: null, required: true }
  },
  emits: ["accepted", "denied"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const app = computed(() => props.session.app);
const permissions = computed(() => {
	return props.session.app.permission.filter((p): p is typeof Misskey.permissions[number] => typeof p === 'string');
});
const name = computed(() => {
	const el = window.document.createElement('div');
	el.textContent = app.value.name;
	return el.innerHTML;
});
function cancel() {
	//misskeyApi('auth/deny', {
	//	token: props.session.token,
	//}).then(() => {
	//	emit('denied');
	//});
	emit('denied');
}
function accept() {
	misskeyApi('auth/accept', {
		token: props.session.token,
	}).then(() => {
		emit('accepted');
	});
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("section", null, [ (permissions.value.length > 0) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("p", null, _toDisplayString(_unref(i18n).tsx._auth.permission({ name: name.value })), 1 /* TEXT */), _createElementVNode("ul", null, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(permissions.value, (p) => {
              return (_openBlock(), _createElementBlock("li", { key: p }, _toDisplayString(_unref(i18n).ts._permissions[p] ?? p), 1 /* TEXT */))
            }), 128 /* KEYED_FRAGMENT */)) ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", null, _toDisplayString(_unref(i18n).tsx._auth.shareAccess({ name: `${name.value} (${app.value.id})` })), 1 /* TEXT */), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.buttons)
      }, [ _createVNode(MkButton, {
          inline: "",
          onClick: cancel
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.cancel), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }), _createVNode(MkButton, {
          inline: "",
          primary: "",
          onClick: accept
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.accept), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }) ]) ]))
}
}

})
