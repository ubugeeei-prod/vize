import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-hourglass-empty" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-minus" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { onBeforeUnmount, onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { host } from '@@/js/config.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { useStream } from '@/stream.js'
import { i18n } from '@/i18n.js'
import { claimAchievement } from '@/utility/achievements.js'
import { pleaseLogin } from '@/utility/please-login.js'
import { $i } from '@/i.js'
import { prefer } from '@/preferences.js'
import { haptic } from '@/utility/haptic.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFollowButton',
  props: {
    user: { type: null, required: true },
    full: { type: Boolean, required: false, default: false },
    large: { type: Boolean, required: false, default: false }
  },
  emits: ["update:user"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const isFollowing = ref(props.user.isFollowing);
const hasPendingFollowRequestFromYou = ref(props.user.hasPendingFollowRequestFromYou);
const wait = ref(false);
const connection = useStream().useChannel('main');
if (props.user.isFollowing == null && $i) {
	misskeyApi('users/show', {
		userId: props.user.id,
	})
		.then(onFollowChange);
}
function onFollowChange(user: Misskey.entities.UserDetailed) {
	if (user.id === props.user.id) {
		isFollowing.value = user.isFollowing;
		hasPendingFollowRequestFromYou.value = user.hasPendingFollowRequestFromYou;
	}
}
async function onClick() {
	const isLoggedIn = await pleaseLogin({
		openOnRemote: {
			type: 'web',
			path: `/@${props.user.username}@${props.user.host ?? host}`,
		},
	});
	if (!isLoggedIn) return;
	wait.value = true;
	haptic();
	try {
		if (isFollowing.value) {
			const { canceled } = await os.confirm({
				type: 'warning',
				text: i18n.tsx.unfollowConfirm({ name: props.user.name || props.user.username }),
			});
			if (canceled) {
				wait.value = false;
				return;
			}
			await misskeyApi('following/delete', {
				userId: props.user.id,
			});
		} else if (hasPendingFollowRequestFromYou.value) {
			const { canceled } = await os.confirm({
				type: 'question',
				text: i18n.tsx.cancelFollowRequestConfirm({ name: props.user.name || props.user.username }),
			});
			if (canceled) {
				wait.value = false;
				return;
			}
			await misskeyApi('following/requests/cancel', {
				userId: props.user.id,
			});
			hasPendingFollowRequestFromYou.value = false;
		} else {
			if (prefer.s.alwaysConfirmFollow) {
				const { canceled } = await os.confirm({
					type: 'question',
					text: i18n.tsx.followConfirm({ name: props.user.name || props.user.username }),
				});
				if (canceled) {
					wait.value = false;
					return;
				}
			}
			await misskeyApi('following/create', {
				userId: props.user.id,
				withReplies: prefer.s.defaultFollowWithReplies,
			});
			emit('update:user', {
				...props.user,
				withReplies: prefer.s.defaultFollowWithReplies,
			});
			hasPendingFollowRequestFromYou.value = true;
			if ($i == null) {
				wait.value = false;
				return;
			}
			claimAchievement('following1');
			if ($i.followingCount >= 10) {
				claimAchievement('following10');
			}
			if ($i.followingCount >= 50) {
				claimAchievement('following50');
			}
			if ($i.followingCount >= 100) {
				claimAchievement('following100');
			}
			if ($i.followingCount >= 300) {
				claimAchievement('following300');
			}
		}
	} catch (err) {
		console.error(err);
	} finally {
		wait.value = false;
	}
}
onMounted(() => {
	connection.on('follow', onFollowChange);
	connection.on('unfollow', onFollowChange);
});
onBeforeUnmount(() => {
	connection.dispose();
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("button", {
      class: _normalizeClass(["_button", [_ctx.$style.root, { [_ctx.$style.wait]: wait.value, [_ctx.$style.active]: isFollowing.value || hasPendingFollowRequestFromYou.value, [_ctx.$style.full]: __props.full, [_ctx.$style.large]: __props.large }]]),
      disabled: wait.value,
      onClick: onClick
    }, [ (!wait.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (hasPendingFollowRequestFromYou.value && __props.user.isLocked) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.text)
                }, _toDisplayString(_unref(i18n).ts.followRequestPending), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _hoisted_1 ], 64 /* STABLE_FRAGMENT */)) : (hasPendingFollowRequestFromYou.value && !__props.user.isLocked) ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.text)
                  }, _toDisplayString(_unref(i18n).ts.processing), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createVNode(_component_MkLoading, {
                  em: true,
                  colored: false
                }, null, 8 /* PROPS */, ["em", "colored"]) ], 64 /* STABLE_FRAGMENT */)) : (isFollowing.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.text)
                  }, _toDisplayString(_unref(i18n).ts.youFollowing), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _hoisted_2 ], 64 /* STABLE_FRAGMENT */)) : (!isFollowing.value && __props.user.isLocked) ? (_openBlock(), _createElementBlock(_Fragment, { key: 3 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.text)
                  }, _toDisplayString(_unref(i18n).ts.followRequest), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _hoisted_3 ], 64 /* STABLE_FRAGMENT */)) : (!isFollowing.value && !__props.user.isLocked) ? (_openBlock(), _createElementBlock(_Fragment, { key: 4 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.text)
                  }, _toDisplayString(_unref(i18n).ts.follow), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _hoisted_4 ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              class: _normalizeClass(_ctx.$style.text)
            }, _toDisplayString(_unref(i18n).ts.processing), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createVNode(_component_MkLoading, {
            em: true,
            colored: false
          }, null, 8 /* PROPS */, ["em", "colored"]) ], 64 /* STABLE_FRAGMENT */)) ], 10 /* CLASS, PROPS */, ["disabled"]))
}
}

})
