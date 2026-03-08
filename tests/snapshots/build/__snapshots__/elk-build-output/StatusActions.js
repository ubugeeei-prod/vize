import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, withCtx as _withCtx, unref as _unref } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusActions',
  props: {
    status: { type: null, required: true },
    details: { type: Boolean, required: false },
    command: { type: Boolean, required: false }
  },
  setup(__props: any) {

const focusEditor = inject<typeof noop>('focus-editor', noop)
const userSettings = useUserSettings()
const useStarFavoriteIcon = usePreferences('useStarFavoriteIcon')
const {
  status,
  isLoading,
  canReblog,
  canQuote,
  toggleBookmark,
  toggleFavourite,
  toggleReblog,
  composeWithQuote,
} = useStatusActions({ status: __props.status })
function reply() {
  if (!checkLogin())
    return
  if (__props.details)
    focusEditor()
  else
    navigateToStatus({ status: status.value, focusReply: true })
}

return (_ctx: any,_cache: any) => {
  const _component_CommonLocalizedNumber = _resolveComponent("CommonLocalizedNumber")
  const _component_StatusActionButton = _resolveComponent("StatusActionButton")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "justify-between": "",
      "items-center": "",
      class: "status-actions"
    }, [ _createElementVNode("div", { "flex-1": "" }, [ _createVNode(_component_StatusActionButton, {
          content: _ctx.$t('action.reply'),
          text: !_ctx.getPreferences(_unref(userSettings), 'hideReplyCount') && _unref(status).repliesCount || '',
          color: "text-blue",
          hover: "text-blue",
          "elk-group-hover": "bg-blue/10",
          icon: "i-ri:chat-1-line",
          command: __props.command,
          onClick: reply
        }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(status).repliesCount && !_ctx.getPreferences(_unref(userSettings), 'hideReplyCount')) ? {
              name: "text",
              fn: _withCtx(() => [
                _createVNode(_component_CommonLocalizedNumber, {
                  keypath: "action.reply_count",
                  count: _unref(status).repliesCount
                }, null, 8 /* PROPS */, ["count"])
              ]),
              key: "0"
            } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["content", "text", "command"]) ]), _createElementVNode("div", { "flex-1": "" }, [ _createVNode(_component_StatusActionButton, {
          content: _ctx.$t(_unref(status).reblogged ? 'action.boosted' : 'action.boost'),
          text: !_ctx.getPreferences(_unref(userSettings), 'hideBoostCount') && _unref(status).reblogsCount ? _unref(status).reblogsCount : '',
          color: "text-green",
          hover: "text-green",
          "elk-group-hover": "bg-green/10",
          icon: "i-ri:repeat-line",
          "active-icon": "i-ri:repeat-fill",
          "inactive-icon": "i-tabler:repeat-off",
          active: !!_unref(status).reblogged,
          disabled: _unref(isLoading).reblogged || !_unref(canReblog),
          command: __props.command,
          onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(toggleReblog)()))
        }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(status).reblogsCount && !_ctx.getPreferences(_unref(userSettings), 'hideBoostCount')) ? {
              name: "text",
              fn: _withCtx(() => [
                _createVNode(_component_CommonLocalizedNumber, {
                  keypath: "action.boost_count",
                  count: _unref(status).reblogsCount
                }, null, 8 /* PROPS */, ["count"])
              ]),
              key: "0"
            } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["content", "text", "active", "disabled", "command"]) ]), _createElementVNode("div", { "flex-1": "" }, [ _createVNode(_component_StatusActionButton, {
          content: _ctx.$t('action.quote'),
          text: !_ctx.getPreferences(_unref(userSettings), 'hideQuoteCount') && _unref(status).quotesCount ? _unref(status).quotesCount : '',
          color: "text-purple",
          hover: "text-purple",
          "elk-group-hover": "bg-purple/10",
          icon: "i-ri:double-quotes-l",
          "active-icon": "i-ri:double-quotes-l",
          "inactive-icon": "i-ri:double-quotes-l",
          disabled: !_unref(canQuote),
          command: __props.command,
          onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(composeWithQuote)()))
        }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(status).quotesCount && !_ctx.getPreferences(_unref(userSettings), 'hideQuoteCount')) ? {
              name: "text",
              fn: _withCtx(() => [
                _createVNode(_component_CommonLocalizedNumber, {
                  keypath: "action.quote_count",
                  count: _unref(status).quotesCount
                }, null, 8 /* PROPS */, ["count"])
              ]),
              key: "0"
            } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["content", "text", "disabled", "command"]) ]), _createElementVNode("div", { "flex-1": "" }, [ _createVNode(_component_StatusActionButton, {
          content: _ctx.$t(_unref(status).favourited ? 'action.favourited' : 'action.favourite'),
          text: !_ctx.getPreferences(_unref(userSettings), 'hideFavoriteCount') && _unref(status).favouritesCount ? _unref(status).favouritesCount : '',
          color: _unref(useStarFavoriteIcon) ? 'text-yellow' : 'text-rose',
          hover: _unref(useStarFavoriteIcon) ? 'text-yellow' : 'text-rose',
          "elk-group-hover": _unref(useStarFavoriteIcon) ? 'bg-yellow/10' : 'bg-rose/10',
          icon: _unref(useStarFavoriteIcon) ? 'i-ri:star-line' : 'i-ri:heart-3-line',
          "active-icon": _unref(useStarFavoriteIcon) ? 'i-ri:star-fill' : 'i-ri:heart-3-fill',
          active: !!_unref(status).favourited,
          disabled: _unref(isLoading).favourited,
          command: __props.command,
          onClick: _cache[2] || (_cache[2] = ($event: any) => (_unref(toggleFavourite)()))
        }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (_unref(status).favouritesCount && !_ctx.getPreferences(_unref(userSettings), 'hideFavoriteCount')) ? {
              name: "text",
              fn: _withCtx(() => [
                _createVNode(_component_CommonLocalizedNumber, {
                  keypath: "action.favourite_count",
                  count: _unref(status).favouritesCount
                }, null, 8 /* PROPS */, ["count"])
              ]),
              key: "0"
            } : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["content", "text", "color", "hover", "elk-group-hover", "icon", "active-icon", "active", "disabled", "command"]) ]), _createElementVNode("div", { "flex-none": "" }, [ _createVNode(_component_StatusActionButton, {
          content: _ctx.$t(_unref(status).bookmarked ? 'action.bookmarked' : 'action.bookmark'),
          color: _unref(useStarFavoriteIcon) ? 'text-rose' : 'text-yellow',
          hover: _unref(useStarFavoriteIcon) ? 'text-rose' : 'text-yellow',
          "elk-group-hover": _unref(useStarFavoriteIcon) ? 'bg-rose/10' : 'bg-yellow/10' ,
          icon: "i-ri:bookmark-line",
          "active-icon": "i-ri:bookmark-fill",
          active: !!_unref(status).bookmarked,
          disabled: _unref(isLoading).bookmarked,
          command: __props.command,
          onClick: _cache[3] || (_cache[3] = ($event: any) => (_unref(toggleBookmark)()))
        }, null, 8 /* PROPS */, ["content", "color", "hover", "elk-group-hover", "active", "disabled", "command"]) ]) ]))
}
}

})
