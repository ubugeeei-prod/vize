import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { "i-svg-spinners-180-ring-with-bg": "true" })
const _hoisted_2 = { "elk-group-hover": "hidden" }
const _hoisted_3 = { hidden: "true", "elk-group-hover": "inline" }
const _hoisted_4 = { "elk-group-hover": "hidden" }
const _hoisted_5 = { hidden: "true", "elk-group-hover": "inline" }
const _hoisted_6 = { "elk-group-hover": "hidden" }
const _hoisted_7 = { hidden: "true", "elk-group-hover": "inline" }
const _hoisted_8 = { "elk-group-hover": "hidden" }
const _hoisted_9 = { hidden: "true", "elk-group-hover": "inline" }
const _hoisted_10 = { "elk-group-hover": "hidden" }
const _hoisted_11 = { hidden: "true", "elk-group-hover": "inline" }
import type { mastodon } from 'masto'
import { toggleFollowAccount, useRelationship } from '~/composables/masto/relationship'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountFollowButton',
  props: {
    account: { type: null, required: true },
    relationship: { type: null, required: false },
    context: { type: String, required: false },
    command: { type: Boolean, required: false }
  },
  setup(__props: any) {

const { t } = useI18n()
const isSelf = useSelfAccount(() => __props.account)
const enable = computed(() => !isSelf.value && currentUser.value)
const relationship = computed(() => __props.relationship || useRelationship(__props.account).value)
const isLoading = computed(() => relationship.value === undefined)
const { client } = useMasto()
async function unblock() {
  relationship.value!.blocking = false
  try {
    const newRel = await client.value.v1.accounts.$select(__props.account.id).unblock()
    Object.assign(relationship!, newRel)
  }
  catch (err) {
    console.error(err)
    // TODO error handling
    relationship.value!.blocking = true
  }
}
async function unmute() {
  relationship.value!.muting = false
  try {
    const newRel = await client.value.v1.accounts.$select(__props.account.id).unmute()
    Object.assign(relationship!, newRel)
  }
  catch (err) {
    console.error(err)
    // TODO error handling
    relationship.value!.muting = true
  }
}
useCommand({
  scope: 'Actions',
  order: -2,
  visible: () => __props.command && enable,
  name: () => `${relationship.value?.following ? t('account.unfollow') : t('account.follow')} ${getShortHandle(__props.account)}`,
  icon: 'i-ri:star-line',
  onActivate: () => toggleFollowAccount(relationship.value!, __props.account),
})
const buttonStyle = computed(() => {
  if (relationship.value?.blocking)
    return 'text-inverted bg-red border-red'

  if (relationship.value?.muting)
    return 'text-base bg-card border-base'

  // If following, use a label style with a strong border for Mutuals
  if (relationship.value ? relationship.value.following : __props.context === 'following')
    return `text-base ${relationship.value?.followedBy ? 'border-strong' : 'border-base'}`

  // If loading, use a plain style
  if (isLoading.value)
    return 'text-base border-base'

  // If not following, use a button style
  return 'text-inverted bg-primary border-primary'
})

return (_ctx: any,_cache: any) => {
  return (enable.value)
      ? (_openBlock(), _createElementBlock("button", {
        key: 0,
        "gap-1": "",
        "items-center": "",
        group: "",
        "border-1": "",
        "rounded-full": "",
        flex: "~ gap2 center",
        "font-500": "",
        "min-w-30": "",
        "h-fit": "",
        px3: "",
        py1: "",
        class: _normalizeClass(buttonStyle.value),
        hover: !relationship.value?.blocking && !relationship.value?.muting && relationship.value?.following ? 'border-red text-red' : 'bg-base border-primary text-primary',
        onClick: _cache[0] || (_cache[0] = ($event: any) => (relationship.value?.blocking ? unblock() : relationship.value?.muting ? unmute() : _unref(toggleFollowAccount)(relationship.value, __props.account)))
      }, [ (isLoading.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_1 ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ (relationship.value?.blocking) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("span", _hoisted_2, _toDisplayString(_ctx.$t('account.blocking')), 1 /* TEXT */), _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('account.unblock')), 1 /* TEXT */) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), (relationship.value?.muting) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("span", _hoisted_4, _toDisplayString(_ctx.$t('account.muting')), 1 /* TEXT */), _createElementVNode("span", _hoisted_5, _toDisplayString(_ctx.$t('account.unmute')), 1 /* TEXT */) ], 64 /* STABLE_FRAGMENT */)) : (relationship.value ? relationship.value.following : __props.context === 'following') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createElementVNode("span", _hoisted_6, _toDisplayString(relationship.value?.followedBy ? _ctx.$t('account.mutuals') : _ctx.$t('account.following')), 1 /* TEXT */), _createElementVNode("span", _hoisted_7, _toDisplayString(_ctx.$t('account.unfollow')), 1 /* TEXT */) ], 64 /* STABLE_FRAGMENT */)) : (relationship.value?.requested) ? (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _createElementVNode("span", _hoisted_8, _toDisplayString(_ctx.$t('account.follow_requested')), 1 /* TEXT */), _createElementVNode("span", _hoisted_9, _toDisplayString(_ctx.$t('account.withdraw_follow_request')), 1 /* TEXT */) ], 64 /* STABLE_FRAGMENT */)) : (relationship.value ? relationship.value.followedBy : __props.context === 'followedBy') ? (_openBlock(), _createElementBlock(_Fragment, { key: 3 }, [ _createElementVNode("span", _hoisted_10, _toDisplayString(_ctx.$t('account.follows_you')), 1 /* TEXT */), _createElementVNode("span", _hoisted_11, _toDisplayString(__props.account.locked ? _ctx.$t('account.request_follow') : _ctx.$t('account.follow_back')), 1 /* TEXT */) ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock("span", { key: 4 }, _toDisplayString(__props.account.locked ? _ctx.$t('account.request_follow') : _ctx.$t('account.follow')), 1 /* TEXT */)) ], 64 /* STABLE_FRAGMENT */)) ]))
      : _createCommentVNode("v-if", true)
}
}

})
