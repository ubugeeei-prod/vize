import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:repeat-fill": "true", "text-xl": "true", "me-2": "true", "color-green": "true" })
import type { GroupedLikeNotifications } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'NotificationGroupedLikes',
  props: {
    group: { type: null, required: true }
  },
  setup(__props: any) {

const useStarFavoriteIcon = usePreferences('useStarFavoriteIcon')
const reblogs = computed(() => __props.group.likes.filter(i => i.reblog))
const likes = computed(() => __props.group.likes.filter(i => i.favourite && !i.reblog))
const timeAgoOptions = useTimeAgoOptions(true)
const reblogsTimeAgoCreatedAt = computed(() => reblogs.value[0].reblog?.createdAt)
const reblogsTimeAgo = useTimeAgo(() => reblogsTimeAgoCreatedAt.value ?? '', timeAgoOptions)
const likesTimeAgoCreatedAt = computed(() => likes.value[0].favourite?.createdAt)
const likesTimeAgo = useTimeAgo(() => likesTimeAgoCreatedAt.value ?? '', timeAgoOptions)

return (_ctx: any,_cache: any) => {
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_AccountHoverWrapper = _resolveComponent("AccountHoverWrapper")
  const _component_StatusBody = _resolveComponent("StatusBody")
  const _component_StatusMedia = _resolveComponent("StatusMedia")
  const _component_StatusPoll = _resolveComponent("StatusPoll")
  const _component_StatusLink = _resolveComponent("StatusLink")

  return (_openBlock(), _createElementBlock("article", {
      flex: "",
      "flex-col": "",
      relative: ""
    }, [ _createVNode(_component_StatusLink, {
        status: __props.group.status,
        pb4: "",
        pt5: ""
      }, {
        default: _withCtx(() => [
          _createElementVNode("div", {
            flex: "",
            "flex-col": "",
            "gap-3": ""
          }, [
            (reblogs.value.length)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                flex: "~ gap-1"
              }, [
                _hoisted_1,
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(reblogs.value, (i, idx) => {
                  return (_openBlock(), _createElementBlock(_Fragment, { key: _ctx.idx }, [
                    _createVNode(_component_AccountHoverWrapper, { account: _ctx.i.account }, {
                      default: _withCtx(() => [
                        _createVNode(_component_NuxtLink, { to: _ctx.getAccountRoute(_ctx.i.account) }, {
                          default: _withCtx(() => [
                            _createVNode(_component_AccountAvatar, {
                              "text-primary": "",
                              "font-bold": "",
                              account: _ctx.i.account,
                              class: "h-1.5em w-1.5em"
                            }, null, 8 /* PROPS */, ["account"])
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 8 /* PROPS */, ["to"])
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["account"])
                  ], 64 /* STABLE_FRAGMENT */))
                }), 128 /* KEYED_FRAGMENT */)),
                _createElementVNode("div", { ml1: "" }, [
                  _createTextVNode(_toDisplayString(_ctx.$t('notification.reblogged_post')) + "\n            ", 1 /* TEXT */),
                  _createElementVNode("time", {
                    "text-secondary": "",
                    datetime: reblogsTimeAgoCreatedAt.value
                  }, "\n              ・" + _toDisplayString(_unref(reblogsTimeAgo)), 9 /* TEXT, PROPS */, ["datetime"])
                ])
              ]))
              : _createCommentVNode("v-if", true),
            (likes.value.length)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                flex: "~ gap-1 wrap"
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_unref(useStarFavoriteIcon) ? 'i-ri:star-line color-yellow' : 'i-ri:heart-line color-red'),
                  "text-xl": "",
                  "me-2": ""
                }, null, 2 /* CLASS */),
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(likes.value, (i, idx) => {
                  return (_openBlock(), _createElementBlock(_Fragment, { key: _ctx.idx }, [
                    _createVNode(_component_AccountHoverWrapper, {
                      account: _ctx.i.account,
                      relative: "",
                      "me--4": "",
                      border: "2 bg-base",
                      "rounded-full": "",
                      "hover:z-1": "",
                      "focus-within:z-1": ""
                    }, {
                      default: _withCtx(() => [
                        _createVNode(_component_NuxtLink, { to: _ctx.getAccountRoute(_ctx.i.account) }, {
                          default: _withCtx(() => [
                            _createVNode(_component_AccountAvatar, {
                              "text-primary": "",
                              "font-bold": "",
                              account: _ctx.i.account,
                              class: "h-1.5em w-1.5em"
                            }, null, 8 /* PROPS */, ["account"])
                          ]),
                          _: 2 /* DYNAMIC */
                        }, 8 /* PROPS */, ["to"])
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["account"])
                  ], 64 /* STABLE_FRAGMENT */))
                }), 128 /* KEYED_FRAGMENT */)),
                _createElementVNode("div", { "ms-4": "" }, [
                  _createTextVNode(_toDisplayString(_ctx.$t('notification.favourited_post')) + "\n            ", 1 /* TEXT */),
                  _createElementVNode("time", {
                    "text-secondary": "",
                    datetime: likesTimeAgoCreatedAt.value
                  }, "\n              ・" + _toDisplayString(_unref(likesTimeAgo)), 9 /* TEXT, PROPS */, ["datetime"])
                ])
              ]))
              : _createCommentVNode("v-if", true)
          ]),
          _createElementVNode("div", {
            ps9: "",
            "mt-1": ""
          }, [
            _createVNode(_component_StatusBody, {
              status: __props.group.status,
              "text-secondary": ""
            }, null, 8 /* PROPS */, ["status"]),
            _createTextVNode("\n        " + "\n        "),
            (!__props.group.status.content)
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                (__props.group.status.mediaAttachments?.length)
                  ? (_openBlock(), _createBlock(_component_StatusMedia, {
                    key: 0,
                    status: __props.group.status,
                    "is-preview": false,
                    "pointer-events-none": ""
                  }, null, 8 /* PROPS */, ["status", "is-preview"]))
                  : (__props.group.status.poll)
                    ? (_openBlock(), _createBlock(_component_StatusPoll, {
                      key: 1,
                      status: __props.group.status
                    }, null, 8 /* PROPS */, ["status"]))
                  : _createCommentVNode("v-if", true)
              ], 64 /* STABLE_FRAGMENT */))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["status"]) ]))
}
}

})
