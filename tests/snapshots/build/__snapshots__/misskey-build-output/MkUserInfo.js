import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
import MkFollowButton from "@/components/MkFollowButton.vue";
import number from "@/filters/number.js";
import { userPage } from "@/filters/user.js";
import { i18n } from "@/i18n.js";
import { $i } from "@/i.js";
import { isFollowingVisibleForMe, isFollowersVisibleForMe } from "@/utility/isFfVisibleForMe.js";
import { getStaticImageUrl } from "@/utility/media-proxy.js";
import { prefer } from "@/preferences.js";
export default {
  __name: "MkUserInfo",
  props: { user: {
    type: null,
    required: true
  } },
  setup(__props) {
    return (_ctx, _cache) => {
      const _component_MkAvatar = _resolveComponent("MkAvatar");
      const _component_MkA = _resolveComponent("MkA");
      const _component_MkUserName = _resolveComponent("MkUserName");
      const _component_MkAcct = _resolveComponent("MkAcct");
      const _component_Mfm = _resolveComponent("Mfm");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(["_panel", _ctx.$style.root]) },
        [
          _createElementVNode(
            "div",
            {
              class: _normalizeClass(_ctx.$style.banner),
              style: _normalizeStyle(__props.user.bannerUrl ? { backgroundImage: `url(${_unref(prefer).s.disableShowingAnimatedImages ? _unref(getStaticImageUrl)(__props.user.bannerUrl) : __props.user.bannerUrl})` } : "")
            },
            null,
            6
            /* CLASS, STYLE */
          ),
          _createVNode(_component_MkA, { to: _unref(userPage)(__props.user) }, {
            default: _withCtx(() => [_createVNode(_component_MkAvatar, {
              class: _normalizeClass(_ctx.$style.avatar),
              user: __props.user,
              indicator: ""
            }, null, 10, ["user"])]),
            _: 1
          }, 8, ["to"]),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.title) },
            [_createVNode(_component_MkA, {
              class: _normalizeClass(_ctx.$style.name),
              to: _unref(userPage)(__props.user)
            }, {
              default: _withCtx(() => [_createVNode(_component_MkUserName, {
                user: __props.user,
                nowrap: false
              }, null, 8, ["user", "nowrap"])]),
              _: 1
            }, 10, ["to"]), _createElementVNode(
              "p",
              { class: _normalizeClass(_ctx.$style.username) },
              [_createVNode(_component_MkAcct, { user: __props.user }, null, 8, ["user"])],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          ),
          _unref($i) && _unref($i).id !== __props.user.id && __props.user.isFollowed ? (_openBlock(), _createElementBlock(
            "span",
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.followed)
            },
            _toDisplayString(_unref(i18n).ts.followsYou),
            3
            /* TEXT, CLASS */
          )) : _createCommentVNode("v-if", true),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.description) },
            [__props.user.description ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.mfm)
              },
              [_createVNode(_component_Mfm, {
                text: __props.user.description,
                author: __props.user
              }, null, 8, ["text", "author"])],
              2
              /* CLASS */
            )) : (_openBlock(), _createElementBlock(
              "span",
              {
                key: 1,
                style: "opacity: 0.7;"
              },
              _toDisplayString(_unref(i18n).ts.noAccountDescription),
              1
              /* TEXT */
            ))],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.status) },
            [
              _createVNode(_component_MkA, {
                class: _normalizeClass(_ctx.$style.statusItem),
                to: _unref(userPage)(__props.user, "notes")
              }, {
                default: _withCtx(() => [_createElementVNode(
                  "p",
                  { class: _normalizeClass(_ctx.$style.statusItemLabel) },
                  _toDisplayString(_unref(i18n).ts.notes),
                  3
                  /* TEXT, CLASS */
                ), _createElementVNode(
                  "span",
                  { class: _normalizeClass(_ctx.$style.statusItemValue) },
                  _toDisplayString(number(__props.user.notesCount)),
                  3
                  /* TEXT, CLASS */
                )]),
                _: 1
              }, 10, ["to"]),
              _unref(isFollowingVisibleForMe)(__props.user) ? (_openBlock(), _createBlock(_component_MkA, {
                key: 0,
                class: _normalizeClass(_ctx.$style.statusItem),
                to: _unref(userPage)(__props.user, "following")
              }, {
                default: _withCtx(() => [_createElementVNode(
                  "p",
                  { class: _normalizeClass(_ctx.$style.statusItemLabel) },
                  _toDisplayString(_unref(i18n).ts.following),
                  3
                  /* TEXT, CLASS */
                ), _createElementVNode(
                  "span",
                  { class: _normalizeClass(_ctx.$style.statusItemValue) },
                  _toDisplayString(number(__props.user.followingCount)),
                  3
                  /* TEXT, CLASS */
                )]),
                _: 1
              }, 10, ["to"])) : _createCommentVNode("v-if", true),
              _unref(isFollowersVisibleForMe)(__props.user) ? (_openBlock(), _createBlock(_component_MkA, {
                key: 0,
                class: _normalizeClass(_ctx.$style.statusItem),
                to: _unref(userPage)(__props.user, "followers")
              }, {
                default: _withCtx(() => [_createElementVNode(
                  "p",
                  { class: _normalizeClass(_ctx.$style.statusItemLabel) },
                  _toDisplayString(_unref(i18n).ts.followers),
                  3
                  /* TEXT, CLASS */
                ), _createElementVNode(
                  "span",
                  { class: _normalizeClass(_ctx.$style.statusItemValue) },
                  _toDisplayString(number(__props.user.followersCount)),
                  3
                  /* TEXT, CLASS */
                )]),
                _: 1
              }, 10, ["to"])) : _createCommentVNode("v-if", true)
            ],
            2
            /* CLASS */
          ),
          __props.user.id != _unref($i)?.id ? (_openBlock(), _createBlock(MkFollowButton, {
            key: 0,
            class: _normalizeClass(_ctx.$style.follow),
            user: __props.user,
            mini: ""
          }, null, 10, ["user"])) : _createCommentVNode("v-if", true)
        ],
        2
        /* CLASS */
      );
    };
  }
};
