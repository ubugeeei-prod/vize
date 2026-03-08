import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { "btn-solid": "true", "mt-1": "true" }
const _hoisted_2 = { "text-primary-active": "true" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { "bg-primary-active": "true", "h-full": "true", "min-w": "1%", class: "w-[var(--bar-width)]" })
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusPoll',
  props: {
    status: { type: null, required: true }
  },
  setup(__props: any) {

const poll = reactive({ ...__props.status.poll! })
function toPercentage(num: number) {
  const percentage = 100 * num
  return `${percentage.toFixed(1).replace(/\.?0+$/, '')}%`
}
const timeAgoOptions = useTimeAgoOptions()
const expiredTimeAgo = useTimeAgo(poll.expiresAt!, timeAgoOptions)
const expiredTimeFormatted = useFormattedDateTime(poll.expiresAt!)
const { formatPercentage } = useHumanReadableNumber()
const loading = ref(false)
const { client } = useMasto()
async function vote(e: Event) {
  const formData = new FormData(e.target as HTMLFormElement)
  const choices = formData.getAll('choices').map(i => +i) as number[]
  // Update the poll optimistically
  for (const [index, option] of poll.options.entries()) {
    if (choices.includes(index))
      option.votesCount = (option.votesCount || 0) + 1
  }
  poll.voted = true
  poll.votesCount++
  if (!poll.votersCount && poll.votesCount)
    poll.votesCount = poll.votesCount + 1
  else
    poll.votersCount = (poll.votersCount || 0) + 1
  cacheStatus({ ...__props.status, poll }, undefined, true)
  await client.value.v1.polls.$select(poll.id).votes.create({ choices })
}
async function refresh() {
  if (loading.value) {
    return
  }
  loading.value = true
  try {
    const newPoll = await client.value.v1.polls.$select(poll.id).fetch()
    Object.assign(poll, newPoll)
    cacheStatus({ ...__props.status, poll: newPoll }, undefined, true)
  }
  catch (e) {
    console.error(e)
  }
  finally {
    loading.value = false
  }
}
const votersCount = computed(() => poll.votersCount ?? poll.votesCount ?? 0)

return (_ctx: any,_cache: any) => {
  const _component_CommonLocalizedNumber = _resolveComponent("CommonLocalizedNumber")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "flex-col": "",
      "w-full": "",
      "items-stretch": "",
      "gap-2": "",
      py3: "",
      dir: "auto",
      class: "poll-wrapper"
    }, [ (!poll.voted && !poll.expired) ? (_openBlock(), _createElementBlock("form", {
          key: 0,
          flex: "~ col gap3",
          "accent-primary": "",
          onClick: _cache[0] || (_cache[0] = _withModifiers((...args) => (_ctx.noop && _ctx.noop(...args)), ["stop"])),
          onSubmit: _withModifiers(vote, ["prevent"])
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(poll.options, (option, index) => {
            return (_openBlock(), _createElementBlock("label", {
              key: index,
              flex: "~ gap2",
              "items-center": ""
            }, [
              _createElementVNode("input", {
                name: "choices",
                value: index,
                type: poll.multiple ? 'checkbox' : 'radio',
                "cursor-pointer": ""
              }, null, 8 /* PROPS */, ["value", "type"]),
              _createTextVNode("\n        " + _toDisplayString(option.title), 1 /* TEXT */)
            ]))
          }), 128 /* KEYED_FRAGMENT */)), _createElementVNode("button", _hoisted_1, _toDisplayString(_ctx.$t('action.vote')), 1 /* TEXT */) ])) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(poll.options, (option, index) => {
            return (_openBlock(), _createElementBlock("div", {
              key: index,
              "py-1": "",
              relative: "",
              style: _normalizeStyle({ '--bar-width': toPercentage(votersCount.value === 0 ? 0 : (option.votesCount ?? 0) / votersCount.value) })
            }, [
              _createElementVNode("div", {
                flex: "",
                "justify-between": "",
                "pb-2": "",
                "w-full": ""
              }, [
                _createElementVNode("span", {
                  "inline-flex": "",
                  "align-items": ""
                }, [
                  _createTextVNode(_toDisplayString(option.title) + "\n            ", 1 /* TEXT */),
                  (poll.voted && poll.ownVotes?.includes(index))
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      "ms-2": "",
                      "mt-1": "",
                      "inline-block": "",
                      "i-ri:checkbox-circle-line": ""
                    }))
                    : _createCommentVNode("v-if", true)
                ]),
                _createElementVNode("span", _hoisted_2, _toDisplayString(_unref(formatPercentage)(votersCount.value > 0 ? (option.votesCount || 0) / votersCount.value : 0)), 1 /* TEXT */)
              ]),
              _createElementVNode("div", {
                class: "bg-gray/40",
                "rounded-l-sm": "",
                "rounded-r-lg": "",
                "h-5px": "",
                "w-full": ""
              }, [
                _hoisted_3
              ])
            ], 4 /* STYLE */))
          }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)), _createElementVNode("div", {
        "text-sm": "",
        "text-secondary": "",
        flex: "",
        "justify-between": "",
        "items-center": "",
        "gap-3": ""
      }, [ _createElementVNode("div", {
          flex: "",
          "gap-x-1": "",
          "flex-wrap": ""
        }, [ _createElementVNode("div", { "inline-block": "" }, [ _createVNode(_component_CommonLocalizedNumber, {
              keypath: "status.poll.count",
              count: poll.votesCount
            }, null, 8 /* PROPS */, ["count"]) ]), _createTextVNode("\n        &middot;\n        "), _createElementVNode("div", { "inline-block": "" }, [ (poll.expiresAt) ? (_openBlock(), _createBlock(_component_CommonTooltip, {
                key: 0,
                content: _unref(expiredTimeFormatted),
                class: "inline-block",
                placement: "right"
              }, {
                default: _withCtx(() => [
                  _createElementVNode("time", { datetime: poll.expiresAt }, _toDisplayString(_ctx.$t(poll.expired ? 'status.poll.finished' : 'status.poll.ends', [_unref(expiredTimeAgo)])), 9 /* TEXT, PROPS */, ["datetime"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["content"])) : _createCommentVNode("v-if", true) ]) ]), (!poll.expired) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("button", {
              "whitespace-nowrap": "",
              flex: "",
              "gap-1": "",
              "items-center": "",
              "hover:text-primary": "",
              onClick: refresh
            }, [ _createElementVNode("div", {
                "text-xs": "",
                class: _normalizeClass(loading.value ? 'animate-spin' : ''),
                "i-ri:loop-right-line": ""
              }, null, 2 /* CLASS */), _createTextVNode("\n          " + _toDisplayString(_ctx.$t('status.poll.update')), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true) ]) ]))
}
}

})
