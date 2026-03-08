import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "M64,32C81.661,32 96,46.339 96,64C95.891,72.184 104,72 104,72C104,72 74.096,80 64,80C52.755,80 24,72 24,72C24,72 31.854,72.018 32,64C32,46.339 46.339,32 64,32Z", style: "fill: var(--MI_THEME-popup);" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkFollowButton from '@/components/MkFollowButton.vue'
import { userPage } from '@/filters/user.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { getUserMenu } from '@/utility/get-user-menu.js'
import number from '@/filters/number.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import { $i } from '@/i.js'
import { isFollowingVisibleForMe, isFollowersVisibleForMe } from '@/utility/isFfVisibleForMe.js'
import { getStaticImageUrl } from '@/utility/media-proxy.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserPopup',
  props: {
    showing: { type: Boolean, required: true },
    q: { type: [String, null], required: true },
    source: { type: null, required: true }
  },
  emits: ["closed", "mouseover", "mouseleave"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const zIndex = os.claimZIndex('middle');
const user = ref<Misskey.entities.UserDetailed | null>(null);
const top = ref(0);
const left = ref(0);
const error = ref(false);
function showMenu(ev: PointerEvent) {
	if (user.value == null) return;
	const { menu, cleanup } = getUserMenu(user.value);
	os.popupMenu(menu, ev.currentTarget ?? ev.target).finally(cleanup);
}
async function fetchUser() {
	if (typeof props.q === 'object') {
		user.value = props.q;
		error.value = false;
	} else {
		const query: Misskey.entities.UsersShowRequest = props.q.startsWith('@') ?
			Misskey.acct.parse(props.q.substring(1)) :
			{ userId: props.q };
		// @ts-expect-error payloadの引数側の型が正常に解決されない
		misskeyApi('users/show', query).then(res => {
			if (!props.showing) return;
			user.value = res;
			error.value = false;
		}, () => {
			error.value = true;
		});
	}
}
onMounted(() => {
	fetchUser();
	const rect = props.source.getBoundingClientRect();
	const x = ((rect.left + (props.source.offsetWidth / 2)) - (300 / 2)) + window.scrollX;
	const y = rect.top + props.source.offsetHeight + window.scrollY;
	top.value = y;
	left.value = x;
});

return (_ctx: any,_cache: any) => {
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkA = _resolveComponent("MkA")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(_Transition, {
      enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_popup_enterActive : '',
      leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_popup_leaveActive : '',
      enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_popup_enterFrom : '',
      leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_popup_leaveTo : '',
      appear: "",
      onAfterLeave: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      default: _withCtx(() => [
        (__props.showing)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(["_popup _shadow", _ctx.$style.root]),
            style: _normalizeStyle({ zIndex: _unref(zIndex), top: top.value + 'px', left: left.value + 'px' }),
            onMouseover: _cache[1] || (_cache[1] = () => { emit('mouseover'); }),
            onMouseleave: _cache[2] || (_cache[2] = () => { emit('mouseleave'); })
          }, [
            (error.value)
              ? (_openBlock(), _createBlock(_component_MkError, {
                key: 0,
                onRetry: _cache[3] || (_cache[3] = ($event: any) => (fetchUser()))
              }))
              : (user.value != null)
                ? (_openBlock(), _createElementBlock("div", { key: 1 }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.banner),
                    style: _normalizeStyle(user.value.bannerUrl ? { backgroundImage: `url(${_unref(prefer).s.disableShowingAnimatedImages ? _unref(getStaticImageUrl)(user.value.bannerUrl) : user.value.bannerUrl})` } : '')
                  }, [
                    (_unref($i) && _unref($i).id != user.value.id && user.value.isFollowed)
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.followed)
                      }, _toDisplayString(_unref(i18n).ts.followsYou), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true)
                  ], 4 /* STYLE */),
                  _createElementVNode("svg", {
                    viewBox: "0 0 128 128",
                    class: _normalizeClass(_ctx.$style.avatarBack)
                  }, [
                    _createElementVNode("g", { transform: "matrix(1.6,0,0,1.6,-38.4,-51.2)" }, [
                      _hoisted_1
                    ])
                  ]),
                  _createVNode(_component_MkA, { to: _unref(userPage)(user.value) }, {
                    default: _withCtx(() => [
                      _createVNode(_component_MkAvatar, {
                        class: _normalizeClass(_ctx.$style.avatar),
                        user: user.value,
                        indicator: ""
                      }, null, 8 /* PROPS */, ["user"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["to"]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.title)
                  }, [
                    _createVNode(_component_MkA, {
                      class: _normalizeClass(_ctx.$style.name),
                      to: _unref(userPage)(user.value)
                    }, {
                      default: _withCtx(() => [
                        _createVNode(_component_MkUserName, {
                          user: user.value,
                          nowrap: false
                        }, null, 8 /* PROPS */, ["user", "nowrap"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.username)
                    }, [
                      _createVNode(_component_MkAcct, { user: user.value }, null, 8 /* PROPS */, ["user"])
                    ])
                  ]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.description)
                  }, [
                    (user.value.description)
                      ? (_openBlock(), _createBlock(_component_Mfm, {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.mfm),
                        text: user.value.description,
                        author: user.value
                      }, null, 8 /* PROPS */, ["text", "author"]))
                      : (_openBlock(), _createElementBlock("div", {
                        key: 1,
                        style: "opacity: 0.7;"
                      }, _toDisplayString(_unref(i18n).ts.noAccountDescription), 1 /* TEXT */))
                  ]),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.status)
                  }, [
                    _createVNode(_component_MkA, {
                      class: _normalizeClass(_ctx.$style.statusItem),
                      to: _unref(userPage)(user.value, 'notes')
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("div", {
                          class: _normalizeClass(_ctx.$style.statusItemLabel)
                        }, _toDisplayString(_unref(i18n).ts.notes), 1 /* TEXT */),
                        _createElementVNode("div", null, _toDisplayString(number(user.value.notesCount)), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["to"]),
                    (_unref(isFollowingVisibleForMe)(user.value))
                      ? (_openBlock(), _createBlock(_component_MkA, {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.statusItem),
                        to: _unref(userPage)(user.value, 'following')
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.statusItemLabel)
                          }, _toDisplayString(_unref(i18n).ts.following), 1 /* TEXT */),
                          _createElementVNode("div", null, _toDisplayString(number(user.value.followingCount)), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"]))
                      : _createCommentVNode("v-if", true),
                    (_unref(isFollowersVisibleForMe)(user.value))
                      ? (_openBlock(), _createBlock(_component_MkA, {
                        key: 0,
                        class: _normalizeClass(_ctx.$style.statusItem),
                        to: _unref(userPage)(user.value, 'followers')
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.statusItemLabel)
                          }, _toDisplayString(_unref(i18n).ts.followers), 1 /* TEXT */),
                          _createElementVNode("div", null, _toDisplayString(number(user.value.followersCount)), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["to"]))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _createElementVNode("button", {
                    class: _normalizeClass(["_button", _ctx.$style.menu]),
                    onClick: showMenu
                  }, [
                    _hoisted_2
                  ]),
                  (_unref($i) && user.value.id != _unref($i).id)
                    ? (_openBlock(), _createBlock(MkFollowButton, {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.follow),
                      mini: "",
                      user: user.value,
                      "onUpdate:user": _cache[4] || (_cache[4] = ($event: any) => ((user).value = $event))
                    }, null, 8 /* PROPS */, ["user"]))
                    : _createCommentVNode("v-if", true)
                ]))
              : (_openBlock(), _createElementBlock("div", { key: 2 }, [
                _createVNode(_component_MkLoading)
              ]))
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass"]))
}
}

})
