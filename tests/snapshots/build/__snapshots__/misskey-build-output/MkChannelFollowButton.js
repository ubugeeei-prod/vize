import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-minus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkChannelFollowButton',
  props: {
    channel: { type: null, required: true },
    full: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props
const isFollowing = ref(props.channel.isFollowing);
const wait = ref(false);
async function onClick() {
	wait.value = true;
	try {
		if (isFollowing.value) {
			await misskeyApi('channels/unfollow', {
				channelId: props.channel.id,
			});
			isFollowing.value = false;
		} else {
			await misskeyApi('channels/follow', {
				channelId: props.channel.id,
			});
			isFollowing.value = true;
		}
	} catch (err) {
		console.error(err);
	} finally {
		wait.value = false;
	}
}

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("button", {
      class: _normalizeClass(["_button", [_ctx.$style.root, { [_ctx.$style.wait]: wait.value, [_ctx.$style.active]: isFollowing.value, [_ctx.$style.full]: __props.full }]]),
      disabled: wait.value,
      onClick: onClick
    }, [ (!wait.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (isFollowing.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.text)
                }, _toDisplayString(_unref(i18n).ts.unfollow), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _hoisted_1 ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.text)
                }, _toDisplayString(_unref(i18n).ts.follow), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _hoisted_2 ], 64 /* STABLE_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (__props.full) ? (_openBlock(), _createElementBlock("span", {
              key: 0,
              class: _normalizeClass(_ctx.$style.text)
            }, _toDisplayString(_unref(i18n).ts.processing), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createVNode(_component_MkLoading, { em: true }, null, 8 /* PROPS */, ["em"]) ], 64 /* STABLE_FRAGMENT */)) ], 10 /* CLASS, PROPS */, ["disabled"]))
}
}

})
