import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue"

import { computed, watch, ref } from 'vue'
import * as Misskey from 'misskey-js'
import XFollowList from './follow-list.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'following',
  props: {
    acct: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const user = ref<null | Misskey.entities.UserDetailed>(null);
const error = ref<any>(null);
function fetchUser(): void {
	if (props.acct == null) return;
	user.value = null;
	misskeyApi('users/show', Misskey.acct.parse(props.acct)).then(u => {
		user.value = u;
	}).catch(err => {
		error.value = err;
	});
}
watch(() => props.acct, fetchUser, {
	immediate: true,
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.user,
	icon: 'ti ti-user',
	...user.value ? {
		title: user.value.name ? `${user.value.name} (@${user.value.username})` : `@${user.value.username}`,
		subtitle: i18n.ts.following,
		userName: user.value,
		avatar: user.value,
	} : {},
}));

return (_ctx: any,_cache: any) => {
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 1000px;"
        }, [
          _createVNode(_Transition, {
            name: "fade",
            mode: "out-in"
          }, {
            default: _withCtx(() => [
              (user.value)
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                  _createVNode(XFollowList, {
                    user: user.value,
                    type: "following"
                  }, null, 8 /* PROPS */, ["user"])
                ]))
                : (error.value)
                  ? (_openBlock(), _createBlock(_component_MkError, {
                    key: 1,
                    onRetry: _cache[0] || (_cache[0] = ($event: any) => (fetchUser()))
                  }))
                : (_openBlock(), _createBlock(_component_MkLoading, { key: 2 }))
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
