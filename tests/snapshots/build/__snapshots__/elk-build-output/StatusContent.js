import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, createSlots as _createSlots, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"

import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'StatusContent',
  props: {
    status: { type: null, required: true },
    newer: { type: null, required: false },
    context: { type: [null, String], required: false },
    isPreview: { type: Boolean, required: false },
    inNotification: { type: Boolean, required: false },
    isNested: { type: Boolean, required: true }
  },
  setup(__props: any) {

const isDM = computed(() => __props.status.visibility === 'direct')
const isDetails = computed(() => __props.context === 'details')
// Content Filter logic
const filterResult = computed(() => __props.status.filtered?.length ? __props.status.filtered[0] : null)
const filter = computed(() => filterResult.value?.filter)
const filterPhrase = computed(() => filter.value?.title)
const isFiltered = computed(() => __props.status.account.id !== currentUser.value?.account.id && filterPhrase && __props.context && __props.context !== 'details' && !!filter.value?.context.includes(__props.context))
// check spoiler text or media attachment
// needed to handle accounts that mark all their posts as sensitive
const spoilerTextPresent = computed(() => !!__props.status.spoilerText && __props.status.spoilerText.trim().length > 0)
const hasSpoilerOrSensitiveMedia = computed(() => spoilerTextPresent.value || (__props.status.sensitive && !!__props.status.mediaAttachments.length))
const isSensitiveNonSpoiler = computed(() => __props.status.sensitive && !__props.status.spoilerText && !!__props.status.mediaAttachments.length)
const hideAllMedia = computed(
  () => {
    return currentUser.value ? (getHideMediaByDefault(currentUser.value.account) && (!!__props.status.mediaAttachments.length || !!__props.status.card?.html)) : false
  },
)
const embeddedMediaPreference = usePreferences('experimentalEmbeddedMedia')
const allowEmbeddedMedia = computed(() => __props.status.card?.html && embeddedMediaPreference.value)

return (_ctx: any,_cache: any) => {
  const _component_StatusBody = _resolveComponent("StatusBody")
  const _component_ContentRich = _resolveComponent("ContentRich")
  const _component_StatusTranslation = _resolveComponent("StatusTranslation")
  const _component_StatusPoll = _resolveComponent("StatusPoll")
  const _component_StatusMedia = _resolveComponent("StatusMedia")
  const _component_StatusPreviewCard = _resolveComponent("StatusPreviewCard")
  const _component_StatusEmbeddedMedia = _resolveComponent("StatusEmbeddedMedia")
  const _component_StatusCard = _resolveComponent("StatusCard")
  const _component_StatusSpoiler = _resolveComponent("StatusSpoiler")

  return (_openBlock(), _createElementBlock("div", {
      "space-y-3": "",
      class: _normalizeClass({
        'py2 px3.5 bg-dm rounded-4 me--1': isDM.value,
        'ms--3.5 mt--1 ms--1': isDM.value && __props.context !== 'details',
      })
    }, [ ((!isFiltered.value && isSensitiveNonSpoiler.value) || hideAllMedia.value) ? (_openBlock(), _createBlock(_component_StatusBody, {
          key: 0,
          status: __props.status,
          newer: __props.newer,
          "with-action": !isDetails.value,
          "is-nested": __props.isNested,
          class: _normalizeClass(isDetails.value ? 'text-xl' : '')
        }, null, 10 /* CLASS, PROPS */, ["status", "newer", "with-action", "is-nested"])) : _createCommentVNode("v-if", true), _createVNode(_component_StatusSpoiler, {
        enabled: hasSpoilerOrSensitiveMedia.value || isFiltered.value,
        filter: isFiltered.value,
        "sensitive-non-spoiler": isSensitiveNonSpoiler.value || hideAllMedia.value,
        "is-d-m": isDM.value
      }, _createSlots({ _: 2 /* DYNAMIC */ }, [ (spoilerTextPresent.value) ? {
            name: "spoiler",
            fn: _withCtx(() => [
              _createElementVNode("p", null, [
                _createVNode(_component_ContentRich, {
                  content: __props.status.spoilerText,
                  emojis: __props.status.emojis,
                  markdown: false
                }, null, 8 /* PROPS */, ["content", "emojis", "markdown"])
              ])
            ]),
            key: "0"
          } : (filterPhrase.value) ? {
            name: "spoiler",
            fn: _withCtx(() => [
              _createElementVNode("p", null, _toDisplayString(`${_ctx.$t('status.filter_hidden_phrase')}: ${filterPhrase.value}`), 1 /* TEXT */)
            ]),
            key: "1"
          } : undefined, (!(isSensitiveNonSpoiler.value || hideAllMedia.value)) ? undefined : undefined, (__props.status.poll) ? undefined : undefined, (__props.status.mediaAttachments?.length) ? undefined : undefined, (__props.status.card && !allowEmbeddedMedia.value && !__props.isNested) ? undefined : undefined, (allowEmbeddedMedia.value) ? undefined : undefined, (__props.status.reblog) ? undefined : undefined ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["enabled", "filter", "sensitive-non-spoiler", "is-d-m"]) ], 2 /* CLASS */))
}
}

})
