import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "flex-auto": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:delete-bin-line": "true", "w-5": "true", "h-5": "true" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusScheduledCard',
  props: {
    item: { type: null, required: true }
  },
  emits: ["deleted"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const scheduledAt = useFormattedDateTime(__props.item.scheduledAt)
const timeAgoOptions = useTimeAgoOptions(true)
const timeago = useTimeAgo(() => __props.item.scheduledAt, timeAgoOptions)
// mastodon.v1.ScheduledStatus does not have account information so we use the current login user
const account = currentUser.value!.account
async function handleDelete() {
  const confirmDelete = await openConfirmDialog({
    title: $t('confirm.delete_posts.title'),
    description: $t('confirm.delete_posts.description'),
    confirm: $t('confirm.delete_posts.confirm'),
    cancel: $t('confirm.delete_posts.cancel'),
  })
  if (confirmDelete.choice !== 'confirm')
    return
  try {
    await useMastoClient().v1.scheduledStatuses.$select(__props.item.id).remove()
    emit('deleted', __props.item.id)
  }
  catch (e) {
    console.error(e)
  }
}

return (_ctx: any,_cache: any) => {
  const _component_AccountBigAvatar = _resolveComponent("AccountBigAvatar")
  const _component_AccountHoverWrapper = _resolveComponent("AccountHoverWrapper")
  const _component_StatusAccountDetails = _resolveComponent("StatusAccountDetails")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createElementBlock("article", {
      relative: "",
      flex: "",
      "gap-3": "",
      "p-4": "",
      "border-b": "",
      "border-base": ""
    }, [ _createElementVNode("div", null, [ _createVNode(_component_AccountHoverWrapper, { account: _unref(account) }, {
          default: _withCtx(() => [
            _createVNode(_component_AccountBigAvatar, { account: _unref(account) }, null, 8 /* PROPS */, ["account"])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["account"]) ]), _createElementVNode("div", {
        flex: "~ col 1",
        "min-w-0": ""
      }, [ _createElementVNode("div", {
          flex: "",
          "items-center": "",
          "space-x-1": ""
        }, [ _createVNode(_component_AccountHoverWrapper, { account: _unref(account) }, {
            default: _withCtx(() => [
              _createVNode(_component_StatusAccountDetails, { account: _unref(account) }, null, 8 /* PROPS */, ["account"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["account"]), _hoisted_1, _createElementVNode("div", {
            "text-sm": "",
            "text-secondary": "",
            flex: "~ row nowrap",
            "whitespace-nowrap": ""
          }, [ _createVNode(_component_CommonTooltip, { content: _unref(scheduledAt) }, {
              default: _withCtx(() => [
                _createElementVNode("time", {
                  datetime: __props.item.scheduledAt,
                  title: __props.item.scheduledAt
                }, _toDisplayString(_unref(timeago)), 9 /* TEXT, PROPS */, ["datetime", "title"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["content"]) ]), _createElementVNode("button", {
            "p-1": "",
            "rounded-full": "",
            "text-secondary": "",
            "hover:text-red": "",
            title: _ctx.$t('menu.delete'),
            onClick: handleDelete
          }, [ _hoisted_2 ], 8 /* PROPS */, ["title"]) ]), (__props.item.params.text) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            mt2: ""
          }, [ _createElementVNode("p", { innerHTML: __props.item.params.text }, null, 8 /* PROPS */, ["innerHTML"]) ])) : _createCommentVNode("v-if", true), (__props.item.mediaAttachments?.length) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            mt2: "",
            flex: "",
            "flex-wrap": "",
            "gap-2": ""
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.item.mediaAttachments, (media) => {
              return (_openBlock(), _createElementBlock("img", {
                key: media.id,
                src: (media.previewUrl || media.url) ?? undefined,
                "max-w-xs": "",
                rounded: "",
                alt: media.description ?? undefined
              }, 8 /* PROPS */, ["src", "alt"]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ]) ]))
}
}

})
