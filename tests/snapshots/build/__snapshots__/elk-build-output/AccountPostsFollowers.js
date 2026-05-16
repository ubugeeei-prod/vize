import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "font-bold": "true", "text-base": "true" }, "Hidden")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "font-bold": "true", "text-base": "true" }, "Hidden")
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountPostsFollowers',
  props: {
    account: { type: null, required: true },
    isHoverCard: { type: Boolean, required: false }
  },
  setup(__props: any) {

const userSettings = useUserSettings()

return (_ctx: any,_cache: any) => {
  const _component_CommonLocalizedNumber = _resolveComponent("CommonLocalizedNumber")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "gap-5": ""
    }, [ _createVNode(_component_NuxtLink, {
        to: _ctx.getAccountRoute(__props.account),
        replace: "",
        "text-secondary": "",
        "exact-active-class": "text-primary"
      }, {
        default: _withCtx(({ isExactActive }) => [
          _createVNode(_component_CommonLocalizedNumber, {
            keypath: "account.posts_count",
            count: __props.account.statusesCount,
            "font-bold": "",
            class: _normalizeClass(isExactActive ? 'text-primary' : 'text-base')
          }, null, 8 /* PROPS */, ["count"])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["to"]), (!(__props.isHoverCard && _ctx.getPreferences(_unref(userSettings), 'hideFollowerCount'))) ? (_openBlock(), _createBlock(_component_NuxtLink, {
          key: 0,
          to: _ctx.getAccountFollowingRoute(__props.account),
          replace: "",
          "text-secondary": "",
          "exact-active-class": "text-primary"
        }, {
          default: _withCtx(({ isExactActive }) => [
            (!_ctx.getPreferences(_unref(userSettings), 'hideFollowerCount'))
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                (__props.account.followingCount >= 0)
                  ? (_openBlock(), _createBlock(_component_CommonLocalizedNumber, {
                    key: 0,
                    keypath: "account.following_count",
                    count: __props.account.followingCount,
                    "font-bold": "",
                    class: _normalizeClass(isExactActive ? 'text-primary' : 'text-base')
                  }, null, 8 /* PROPS */, ["count"]))
                  : (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    flex: "",
                    "gap-x-1": ""
                  }, [
                    _hoisted_1,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('account.following')), 1 /* TEXT */)
                  ]))
              ], 64 /* STABLE_FRAGMENT */))
              : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_ctx.$t('account.following')), 1 /* TEXT */))
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true), (!(__props.isHoverCard && _ctx.getPreferences(_unref(userSettings), 'hideFollowerCount'))) ? (_openBlock(), _createBlock(_component_NuxtLink, {
          key: 0,
          to: _ctx.getAccountFollowersRoute(__props.account),
          replace: "",
          "text-secondary": "",
          "exact-active-class": "text-primary"
        }, {
          default: _withCtx(({ isExactActive }) => [
            (!_ctx.getPreferences(_unref(userSettings), 'hideFollowerCount'))
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                (__props.account.followersCount >= 0)
                  ? (_openBlock(), _createBlock(_component_CommonLocalizedNumber, {
                    key: 0,
                    keypath: "account.followers_count",
                    count: __props.account.followersCount,
                    "font-bold": "",
                    class: _normalizeClass(isExactActive ? 'text-primary' : 'text-base')
                  }, null, 8 /* PROPS */, ["count"]))
                  : (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    flex: "",
                    "gap-x-1": ""
                  }, [
                    _hoisted_2,
                    _createElementVNode("span", null, _toDisplayString(_ctx.$t('account.followers')), 1 /* TEXT */)
                  ]))
              ], 64 /* STABLE_FRAGMENT */))
              : (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(_ctx.$t('account.followers')), 1 /* TEXT */))
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["to"])) : _createCommentVNode("v-if", true) ]))
}
}

})
