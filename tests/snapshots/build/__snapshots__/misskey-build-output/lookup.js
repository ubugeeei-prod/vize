import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, ref } from 'vue'
import * as Misskey from 'misskey-js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { mainRouter } from '@/router.js'
import MkButton from '@/components/MkButton.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'lookup',
  setup(__props) {

const state = ref<'fetching' | 'done'>('fetching');
function _fetch_() {
	const params = new URL(window.location.href).searchParams;
	// acctのほうはdeprecated
	let uri = params.get('uri') ?? params.get('acct');
	if (uri == null) {
		state.value = 'done';
		return;
	}
	let promise: Promise<unknown>;
	if (uri.startsWith('https://')) {
		promise = misskeyApi('ap/show', {
			uri,
		}).then(res => {
			if (res.type === 'User') {
				mainRouter.replace('/@:acct/:page?', {
					params: {
						acct: res.object.host != null ? `${res.object.username}@${res.object.host}` : res.object.username,
					},
				});
			} else if (res.type === 'Note') {
				mainRouter.replace('/notes/:noteId/:initialTab?', {
					params: {
						noteId: res.object.id,
					},
				});
			} else {
				os.alert({
					type: 'error',
					text: 'Not a user',
				});
			}
		});
	} else {
		if (uri.startsWith('acct:')) {
			uri = uri.slice(5);
		}
		promise = misskeyApi('users/show', Misskey.acct.parse(uri)).then(user => {
			mainRouter.replace('/@:acct/:page?', {
				params: {
					acct: user.host != null ? `${user.username}@${user.host}` : user.username,
				},
			});
		});
	}
	os.promiseDialog(promise, null, null, i18n.ts.fetchingAsApObject);
}
function close(): void {
	window.close();
	// 閉じなければ100ms後タイムラインに
	window.setTimeout(() => {
		window.location.href = '/';
	}, 100);
}
function goToMisskey(): void {
	window.location.href = '/';
}
_fetch_();
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage({
	title: i18n.ts.lookup,
	icon: 'ti ti-world-search',
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          (state.value === 'done')
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_buttonsCenter"
            }, [
              _createVNode(MkButton, { onClick: close }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.close), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, { onClick: goToMisskey }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.goToMisskey), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "_fullInfo"
            }, [
              _createVNode(_component_MkLoading)
            ]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
