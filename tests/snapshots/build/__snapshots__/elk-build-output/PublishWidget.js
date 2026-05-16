import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vModelText as _vModelText, withModifiers as _withModifiers, withKeys as _withKeys } from "vue"


const _hoisted_1 = { id: "state-editing", "text-secondary": "true", "self-center": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { "w-1px": "true", border: "x base", "mb-6": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:error-warning-fill": "true" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", w: "1.75em", h: "1.75em", "i-ri:close-line": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:error-warning-fill": "true" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", w: "1.75em", h: "1.75em", "i-ri:close-line": "true" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:error-warning-fill": "true" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:loader-2-fill": "true" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:error-warning-fill": "true" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", w: "1.75em", h: "1.75em", "i-ri:close-line": "true" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:delete-bin-line": "true" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:close-line": "true" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:emotion-line": "true" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:image-add-line": "true" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:chat-poll-line": "true" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:close-line": "true" })
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:list-settings-line": "true" })
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:arrow-down-s-line": "true", "text-sm": "true", "text-secondary": "true", "me--1": "true" })
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:hourglass-line": "true" })
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:arrow-down-s-line": "true", "text-sm": "true", "text-secondary": "true", "me--1": "true" })
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("div", { "flex-auto": "true" })
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:arrow-down-s-line": "true", "text-sm": "true", "text-secondary": "true", "me--1": "true" })
const _hoisted_23 = /*#__PURE__*/ _createElementVNode("div", { block: "true", "i-carbon:face-dizzy-filled": "true" })
const _hoisted_24 = /*#__PURE__*/ _createElementVNode("div", { block: "true", "i-ri:loader-2-fill": "true" })
const _hoisted_25 = /*#__PURE__*/ _createElementVNode("div", { block: "true", "i-carbon:face-dizzy-filled": "true" })
import type { DraftItem, DraftKey } from '#shared/types'
import type { mastodon } from 'masto'
import { EditorContent } from '@tiptap/vue-3'
import { useNow } from '@vueuse/core'
import stringLength from 'string-length'
const expiresInDefaultOptionIndex = 2

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishWidget',
  props: {
    draftKey: { type: null, required: true },
    draftItemIndex: { type: Number, required: true },
    initial: { type: Function, required: false, default: getDefaultDraftItem },
    threadComposer: { type: null, required: false },
    placeholder: { type: String, required: false },
    inReplyToId: { type: String, required: false },
    inReplyToVisibility: { type: null, required: false },
    expanded: { type: Boolean, required: false, default: false },
    dialogLabelledBy: { type: String, required: false }
  },
  emits: ["published"],
  async setup(__props: any, { expose: __expose, emit: __emit }) {

let __temp: any, __restore: any

const emit = __emit
const { t } = useI18n()
const { threadItems, threadIsActive, publishThread, threadIsSending } = __props.threadComposer ?? useThreadComposer(__props.draftKey)
const draft = computed({
  get: () => threadItems.value[draftItemIndex],
  set: (updatedDraft: DraftItem) => {
    threadItems.value[__props.draftItemIndex] = updatedDraft
  },
},
)
const isFinalItemOfThread = computed(() => __props.draftItemIndex === threadItems.value.length - 1)
const {
  isExceedingAttachmentLimit,
  isUploading,
  failedAttachments,
  isOverDropZone,
  uploadAttachments,
  pickAttachments,
  setDescription,
  removeAttachment,
  dropZoneRef,
} = useUploadMediaAttachment(draft)
const {
  shouldExpanded,
  isExpanded,
  isSending,
  isPublishDisabled,
  publishDraft,
  failedMessages,
  preferredLanguage,
  publishSpoilerText,
} = usePublish(
  {
    draftItem: draft,
    ...{ expanded: toRef(() => __props.expanded), isUploading, initialDraft: __props.initial, isPartOfThread: false },
  },
)
const { editor } = useTiptap({
  content: computed({
    get: () => draft.value.params.status,
    set: (newVal) => {
      draft.value.params.status = newVal
      draft.value.lastUpdated = Date.now()
    },
  }),
  placeholder: computed(() => __props.placeholder ?? draft.value.params.inReplyToId ? t('placeholder.replying') : t('placeholder.default_1')),
  autofocus: shouldExpanded.value,
  onSubmit: publish,
  onFocus() {
    if (!isExpanded && draft.value.initialText) {
      editor.value?.chain().insertContent(`${draft.value.initialText} `).focus('end').run()
      draft.value.initialText = ''
    }
    isExpanded.value = true
  },
  onPaste: handlePaste,
})
function trimPollOptions() {
  const indexLastNonEmpty = draft.value.params.poll!.options.findLastIndex(option => option.trim().length > 0)
  const trimmedOptions = draft.value.params.poll!.options.slice(0, indexLastNonEmpty + 1)
  if (currentInstance.value?.configuration
    && trimmedOptions.length >= currentInstance.value?.configuration?.polls.maxOptions) {
    draft.value.params.poll!.options = trimmedOptions
  }
  else {
    draft.value.params.poll!.options = [...trimmedOptions, '']
  }
}
function editPollOptionDraft(event: Event, index: number) {
  draft.value.params.poll!.options = Object.assign(draft.value.params.poll!.options.slice(), { [index]: (event.target as HTMLInputElement).value })
  trimPollOptions()
}
function deletePollOption(index: number) {
  const newPollOptions = draft.value.params.poll!.options.slice()
  newPollOptions.splice(index, 1)
  draft.value.params.poll!.options = newPollOptions
  trimPollOptions()
}
const expiresInOptions = computed(() => [
  {
    seconds: 1 * 60 * 60,
    label: t('time_ago_options.hour_future', 1),
  },
  {
    seconds: 2 * 60 * 60,
    label: t('time_ago_options.hour_future', 2),
  },
  {
    seconds: 1 * 24 * 60 * 60,
    label: t('time_ago_options.day_future', 1),
  },
  {
    seconds: 2 * 24 * 60 * 60,
    label: t('time_ago_options.day_future', 2),
  },
  {
    seconds: 7 * 24 * 60 * 60,
    label: t('time_ago_options.day_future', 7),
  },
])
const scheduledTime = ref('')
const now = useNow({ interval: 1000 })
const minimumScheduledTime = computed(() => getMinimumScheduledTime(now.value))
const isValidScheduledTime = computed(() => {
  if (scheduledTime.value === '')
    return true

  const scheduledTimeDate = new Date(scheduledTime.value)
  return minimumScheduledTime.value.getTime() <= scheduledTimeDate.getTime()
})
const initialDateTime = computed(() => {
  const t = new Date(minimumScheduledTime.value.getTime())
  t.setHours(t.getHours() + 1)
  t.setMinutes(0)
  t.setSeconds(0)
  t.setMilliseconds(0)
  return t
})
watchEffect(() => {
  // Convert the local datetime string from the input to a UTC ISO string for the API
  if (scheduledTime.value) {
    const localDate = new Date(scheduledTime.value)
    draft.value.params.scheduledAt = localDate.toISOString()
  }
  else {
    draft.value.params.scheduledAt = ''
  }
})
function setInitialScheduledTime() {
  if (scheduledTime.value === '') {
    scheduledTime.value = getDatetimeInputFormat(initialDateTime.value)
  }
}
watchEffect(() => {
  draft.value.params.scheduledAt = scheduledTime.value
})
// Calculate the minimum scheduled time.
// Mastodon API allows to set the scheduled time to 5 minutes in the future
// but if the specified scheduled time is less than 5 minutes, Mastodon will
// send the post immediately.
// To prevent this, we add a buffer and round up the minutes.
function getMinimumScheduledTime(now: Date): Date {
  const bufferInSec = 5 + 5 * 60 // + 5 minutes and 5 seconds
  const nowInSec = Math.floor(now.getTime() / 1000)
  const bufferedTimeInSec
    = Math.ceil((nowInSec + bufferInSec) / 60) * 60
  return new Date(bufferedTimeInSec * 1000)
}
function getDatetimeInputFormat(time: Date) {
  // Returns string in 'YYYY-MM-DDTHH:MM' format using local time components
  // This is the format expected by the <input type="datetime-local"> element.
  const year = time.getFullYear()
  const month = (time.getMonth() + 1).toString().padStart(2, '0')
  const day = time.getDate().toString().padStart(2, '0')
  const hours = time.getHours().toString().padStart(2, '0')
  const minutes = time.getMinutes().toString().padStart(2, '0')
  return `${year}-${month}-${day}T${hours}:${minutes}`
}
const characterCount = computed(() => {
  const text = htmlToText(editor.value?.getHTML() || '')

  let length = stringLength(text)

  // taken from https://github.com/mastodon/mastodon/blob/07f8b4d1b19f734d04e69daeb4c3421ef9767aac/app/lib/text_formatter.rb
  const linkRegex = /(https?:\/\/|xmpp:)\S+/g

  // taken from https://github.com/mastodon/mastodon/blob/af578e/app/javascript/mastodon/features/compose/util/counter.js
  const countableMentionRegex = /(^|[^/\w])@((\w+)@[a-z0-9.-]+[a-z0-9])/gi

  // maximum of 23 chars per link
  // https://github.com/elk-zone/elk/issues/1651
  const maxLength = 23

  for (const [fullMatch] of text.matchAll(linkRegex))
    length -= fullMatch.length - Math.min(maxLength, fullMatch.length)

  for (const [fullMatch, before, _handle, username] of text.matchAll(countableMentionRegex))
    length -= fullMatch.length - (before + username).length - 1 // - 1 for the @

  if (draft.value.mentions) {
    // + 1 is needed as mentions always need a space separator at the end
    length += draft.value.mentions.map((mention) => {
      const [handle] = mention.split('@')
      return `@${handle}`
    }).join(' ').length + 1
  }

  length += stringLength(publishSpoilerText.value)

  return length
})
const isExceedingCharacterLimit = computed(() => {
  return characterCount.value > characterLimit.value
})
const postLanguageDisplay = computed(() => languagesNameList.find(i => i.code === (draft.value.params.language || preferredLanguage.value))?.nativeName)
const isDM = computed(() => draft.value.params.visibility === 'direct')
const hasQuote = computed(() => !!draft.value.params.quotedStatusId)
const quotedStatus = ref<mastodon.v1.Status | null>(null)
const quoteFetchError = ref<string | null>(null)
watchEffect(async () => {
  if (hasQuote.value) {
    try {
      quotedStatus.value = await fetchStatus(draft.value.params.quotedStatusId!)
    }
    catch (err) {
      console.error(err)
      quoteFetchError.value = (err as Error).message
    }
  }
})
function removeQuote() {
  draft.value.params.quotedStatusId = undefined
  draft.value.params.quoteApprovalPolicy = undefined
  quotedStatus.value = null
  quoteFetchError.value = null
}
async function handlePaste(evt: ClipboardEvent) {
  const files = evt.clipboardData?.files
  if (!files || files.length === 0)
    return
  evt.preventDefault()
;(
  ([__temp,__restore] = _withAsyncContext(() => uploadAttachments(Array.from(files)))),
  await __temp,
  __restore()
)
}
function insertEmoji(name: string) {
  editor.value?.chain().focus().insertEmoji(name).run()
}
function insertCustomEmoji(image: any) {
  editor.value?.chain().focus().insertCustomEmoji(image).run()
}
async function toggleSensitive() {
  draft.value.params.sensitive = !draft.value.params.sensitive
}
async function publish() {
  if (isPublishDisabled.value || isExceedingCharacterLimit.value)
    return
  const publishResult = await (threadIsActive.value ? publishThread() : publishDraft())
  if (publishResult) {
    if (Array.isArray(publishResult))
      failedMessages.value = publishResult
    else
      emit('published', publishResult)
  }
}
useWebShareTarget(async ({ data: { data, action } }: any) => {
  if (action !== 'compose-with-shared-data')
    return
  editor.value?.commands.focus('end')
  for (const text of data.textParts) {
    for (const line of text.split('\n')) {
      editor.value?.commands.insertContent({
        type: 'paragraph',
        content: [{ type: 'text', text: line }],
      })
    }
  }
  if (data.files.length !== 0)
;(
  ([__temp,__restore] = _withAsyncContext(() => uploadAttachments(data.files))),
  await __temp,
  __restore()
)
})
function stopQuestionMarkPropagation(e: KeyboardEvent) {
  if (e.key === '?')
    e.stopImmediatePropagation()
}
const userSettings = useUserSettings()
const optimizeForLowPerformanceDevice = computed(() => getPreferences(userSettings.value, 'optimizeForLowPerformanceDevice'))
const languageDetectorInGlobalThis = 'LanguageDetector' in globalThis
let supportsLanguageDetector = !optimizeForLowPerformanceDevice.value && languageDetectorInGlobalThis && await (globalThis as any).LanguageDetector.availability() === 'available'
let languageDetector: { detect: (arg0: string, option: { signal: AbortSignal }) => any }
// If the API is supported, but the model not loaded yet…
if (languageDetectorInGlobalThis && !supportsLanguageDetector) {
  // …trigger the model download
  (globalThis as any).LanguageDetector.create().then((_languageDetector: { detect: (arg0: string) => any }) => {
    supportsLanguageDetector = true
    languageDetector = _languageDetector
  })
}
function countLetters(text: string) {
  const segmenter = new Intl.Segmenter('und', { granularity: 'grapheme' })
  const letters = [...segmenter.segment(text)]
  return letters.length
}
let detectLanguageAbortController = new AbortController()
const detectLanguage = useDebounceFn(async () => {
  if (!supportsLanguageDetector) {
    return
  }
  if (!languageDetector) {
    // maybe we dont want to mess with this with abort....
    languageDetector = await (globalThis as any).LanguageDetector.create()
  }
  // we stop previously running language detection process
  detectLanguageAbortController.abort()
  detectLanguageAbortController = new AbortController()
  const text = htmlToText(editor.value?.getHTML() || '')
  if (!text || countLetters(text) <= 5) {
    draft.value.params.language = preferredLanguage.value
    return
  }
  try {
    const detectedLanguage = (await languageDetector.detect(text, { signal: detectLanguageAbortController.signal }))[0].detectedLanguage
    draft.value.params.language = detectedLanguage === 'und' ? preferredLanguage.value : detectedLanguage.substring(0, 2)
  }
  catch (e) {
    // if error or abort we end up there
    if ((e as Error).name !== 'AbortError') {
      console.error(e)
    }
    draft.value.params.language = preferredLanguage.value
  }
}, 500)
__expose({
  focusEditor: () => {
    editor.value?.commands?.focus?.()
  },
})

return (_ctx: any,_cache: any) => {
  const _component_AccountBigAvatar = _resolveComponent("AccountBigAvatar")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_ContentMentionGroup = _resolveComponent("ContentMentionGroup")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_CommonErrorMessage = _resolveComponent("CommonErrorMessage")
  const _component_PublishAttachment = _resolveComponent("PublishAttachment")
  const _component_StatusCard = _resolveComponent("StatusCard")
  const _component_StatusCardSkeleton = _resolveComponent("StatusCardSkeleton")
  const _component_PublishEmojiPicker = _resolveComponent("PublishEmojiPicker")
  const _component_CommonCheckbox = _resolveComponent("CommonCheckbox")
  const _component_CommonDropdown = _resolveComponent("CommonDropdown")
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")
  const _component_PublishEditorTools = _resolveComponent("PublishEditorTools")
  const _component_PublishCharacterCounter = _resolveComponent("PublishCharacterCounter")
  const _component_PublishLanguagePicker = _resolveComponent("PublishLanguagePicker")
  const _component_PublishVisibilityPicker = _resolveComponent("PublishVisibilityPicker")
  const _component_PublishQuoteApprovalPicker = _resolveComponent("PublishQuoteApprovalPicker")
  const _component_PublishThreadTools = _resolveComponent("PublishThreadTools")

  return (_ctx.isHydrated && _ctx.currentUser)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        flex: "~ col gap-4",
        py3: "",
        px2: "",
        "sm:px4": "",
        "aria-roledescription": "publish-widget"
      }, [ (draft.value.editingStatus) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            id: "state-editing",
            "text-secondary": "",
            "self-center": ""
          }, _toDisplayString(_ctx.$t('state.editing')), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          flex: "",
          "gap-3": "",
          "flex-1": ""
        }, [ _createElementVNode("div", null, [ _createVNode(_component_NuxtLink, {
              "self-start": "",
              to: _ctx.getAccountRoute(_ctx.currentUser.account)
            }, {
              default: _withCtx(() => [
                _createVNode(_component_AccountBigAvatar, {
                  account: _ctx.currentUser.account,
                  square: ""
                }, null, 8 /* PROPS */, ["account"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["to"]), (!isFinalItemOfThread.value) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                "w-full": "",
                "h-full": "",
                flex: "",
                "mt--3px": "",
                "justify-center": ""
              }, [ _hoisted_2 ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", { "w-full": "" }, [ _createElementVNode("div", {
              flex: "",
              "gap-3": "",
              "flex-1": ""
            }, [ _createElementVNode("div", {
                ref_key: "dropZoneRef", ref: dropZoneRef,
                flex: "",
                "w-0": "",
                "flex-col": "",
                "gap-3": "",
                "flex-1": "",
                border: "2 dashed transparent",
                class: _normalizeClass([_unref(isSending) ? 'pointer-events-none' : '', _unref(isOverDropZone) ? '!border-primary' : ''])
              }, [ (draft.value.mentions?.length && _unref(shouldExpanded)) ? (_openBlock(), _createBlock(_component_ContentMentionGroup, {
                    key: 0,
                    replying: ""
                  }, {
                    default: _withCtx(() => [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(draft.value.mentions, (m, i) => {
                        return (_openBlock(), _createElementBlock("button", {
                          key: _ctx.m,
                          "text-primary": "",
                          "hover:color-red": "",
                          onClick: _cache[0] || (_cache[0] = ($event: any) => (draft.value.mentions?.splice(_ctx.i, 1)))
                        }, _toDisplayString(_ctx.accountToShortHandle(_ctx.m)), 1 /* TEXT */))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]),
                    _: 1 /* STABLE */
                  })) : _createCommentVNode("v-if", true), (draft.value.params.sensitive) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _withDirectives(_createElementVNode("input", {
                      "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((publishSpoilerText).value = $event)),
                      type: "text",
                      placeholder: _ctx.$t('placeholder.content_warning'),
                      p2: "",
                      "border-rounded": "",
                      "w-full": "",
                      "bg-transparent": "",
                      "outline-none": "",
                      border: "~ base"
                    }, null, 8 /* PROPS */, ["placeholder"]), [ [_vModelText, _unref(publishSpoilerText)] ]) ])) : _createCommentVNode("v-if", true), (_unref(failedMessages).length > 0) ? (_openBlock(), _createBlock(_component_CommonErrorMessage, {
                    key: 0,
                    "described-by": "publish-failed"
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("header", {
                        id: "publish-failed",
                        flex: "",
                        "justify-between": ""
                      }, [
                        _createElementVNode("div", {
                          flex: "",
                          "items-center": "",
                          "gap-x-2": "",
                          "font-bold": ""
                        }, [
                          _hoisted_3,
                          _createElementVNode("p", null, _toDisplayString(_ctx.$t('state.publish_failed')), 1 /* TEXT */)
                        ]),
                        _createVNode(_component_CommonTooltip, {
                          placement: "bottom",
                          content: _ctx.$t('action.clear_publish_failed')
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("button", {
                              flex: "",
                              "rounded-4": "",
                              p1: "",
                              "hover:bg-active": "",
                              "cursor-pointer": "",
                              "transition-100": "",
                              "aria-label": _ctx.$t('action.clear_publish_failed'),
                              onClick: _cache[2] || (_cache[2] = ($event: any) => (failedMessages.value = []))
                            }, [
                              _hoisted_4
                            ], 8 /* PROPS */, ["aria-label"])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["content"])
                      ]),
                      _createElementVNode("ol", {
                        "ps-2": "",
                        "sm:ps-1": ""
                      }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(failedMessages), (error, i) => {
                          return (_openBlock(), _createElementBlock("li", {
                            key: i,
                            flex: "~ col sm:row",
                            "gap-y-1": "",
                            "sm:gap-x-2": ""
                          }, [
                            _createElementVNode("strong", null, _toDisplayString(i + 1) + ".", 1 /* TEXT */),
                            _createElementVNode("span", null, _toDisplayString(error), 1 /* TEXT */)
                          ]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ])
                    ]),
                    _: 1 /* STABLE */
                  })) : _createCommentVNode("v-if", true), (_unref(failedMessages).length > 0) ? (_openBlock(), _createBlock(_component_CommonErrorMessage, {
                    key: 0,
                    "described-by": "publish-failed"
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("header", {
                        id: "publish-failed",
                        flex: "",
                        "justify-between": ""
                      }, [
                        _createElementVNode("div", {
                          flex: "",
                          "items-center": "",
                          "gap-x-2": "",
                          "font-bold": ""
                        }, [
                          _hoisted_5,
                          _createElementVNode("p", null, _toDisplayString(scheduledTime.value ? _ctx.$t('state.schedule_failed') : _ctx.$t('state.publish_failed')), 1 /* TEXT */)
                        ]),
                        _createVNode(_component_CommonTooltip, {
                          placement: "bottom",
                          content: scheduledTime.value ? _ctx.$t('action.clear_schedule_failed') : _ctx.$t('action.clear_publish_failed')
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("button", {
                              flex: "",
                              "rounded-4": "",
                              p1: "",
                              "hover:bg-active": "",
                              "cursor-pointer": "",
                              "transition-100": "",
                              "aria-label": scheduledTime.value ? _ctx.$t('action.clear_schedule_failed') : _ctx.$t('action.clear_publish_failed'),
                              onClick: _cache[3] || (_cache[3] = ($event: any) => (failedMessages.value = []))
                            }, [
                              _hoisted_6
                            ], 8 /* PROPS */, ["aria-label"])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["content"])
                      ]),
                      _createElementVNode("ol", {
                        "ps-2": "",
                        "sm:ps-1": ""
                      }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(failedMessages), (error, i) => {
                          return (_openBlock(), _createElementBlock("li", {
                            key: i,
                            flex: "~ col sm:row",
                            "gap-y-1": "",
                            "sm:gap-x-2": ""
                          }, [
                            _createElementVNode("strong", null, _toDisplayString(i + 1) + ".", 1 /* TEXT */),
                            _createElementVNode("span", null, _toDisplayString(error), 1 /* TEXT */)
                          ]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ])
                    ]),
                    _: 1 /* STABLE */
                  })) : _createCommentVNode("v-if", true), (!isValidScheduledTime.value) ? (_openBlock(), _createBlock(_component_CommonErrorMessage, {
                    key: 0,
                    "described-by": "scheduled-time-invalid",
                    "pt-2": ""
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("header", {
                        id: "scheduled-time-invalid",
                        flex: "",
                        "justify-between": ""
                      }, [
                        _createElementVNode("div", {
                          flex: "",
                          "items-center": "",
                          "gap-x-2": "",
                          "font-bold": ""
                        }, [
                          _hoisted_7,
                          _createElementVNode("p", null, _toDisplayString(_ctx.$t('state.schedule_time_invalid', [minimumScheduledTime.value.toLocaleString()])), 1 /* TEXT */)
                        ])
                      ])
                    ]),
                    _: 1 /* STABLE */
                  })) : _createCommentVNode("v-if", true), _createElementVNode("div", {
                  relative: "",
                  "flex-1": "",
                  flex: "",
                  "flex-col": "",
                  class: _normalizeClass(_unref(shouldExpanded) ? 'min-h-30' : '')
                }, [ _createVNode(EditorContent, {
                    editor: _unref(editor),
                    flex: "",
                    "max-w-full": "",
                    class: _normalizeClass({
                    'md:max-h-[calc(100vh-200px)] sm:max-h-[calc(100vh-400px)] max-h-35 of-y-auto overscroll-contain': _unref(shouldExpanded),
                    'py2 px3.5 bg-dm rounded-4 me--1 ms--1 mt--1': isDM.value,
                  }),
                    onKeydown: [stopQuestionMarkPropagation, _withKeys(_withModifiers(($event: any) => (_unref(editor)?.commands.blur()), ["prevent"]), ["esc"])],
                    onKeyup: _cache[4] || (_cache[4] = (...args) => (detectLanguage && detectLanguage(...args)))
                  }, null, 10 /* CLASS, PROPS */, ["editor"]) ], 2 /* CLASS */), (_unref(isUploading)) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    flex: "",
                    "gap-1": "",
                    "items-center": "",
                    "text-sm": "",
                    p1: "",
                    "text-primary": ""
                  }, [ _createElementVNode("div", {
                      "animate-spin": "",
                      "preserve-3d": ""
                    }, [ _hoisted_8 ]), _createTextVNode("\n              "), _toDisplayString(_ctx.$t('state.uploading')) ])) : (_unref(failedAttachments).length > 0) ? (_openBlock(), _createBlock(_component_CommonErrorMessage, {
                      key: 1,
                      "described-by": _unref(isExceedingAttachmentLimit) ? 'upload-failed uploads-per-post' : 'upload-failed'
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("header", {
                          id: "upload-failed",
                          flex: "",
                          "justify-between": ""
                        }, [
                          _createElementVNode("div", {
                            flex: "",
                            "items-center": "",
                            "gap-x-2": "",
                            "font-bold": ""
                          }, [
                            _hoisted_9,
                            _createElementVNode("p", null, _toDisplayString(_ctx.$t('state.upload_failed')), 1 /* TEXT */)
                          ]),
                          _createVNode(_component_CommonTooltip, {
                            placement: "bottom",
                            content: _ctx.$t('action.clear_upload_failed')
                          }, {
                            default: _withCtx(() => [
                              _createElementVNode("button", {
                                flex: "",
                                "rounded-4": "",
                                p1: "",
                                "hover:bg-active": "",
                                "cursor-pointer": "",
                                "transition-100": "",
                                "aria-label": _ctx.$t('action.clear_upload_failed'),
                                onClick: _cache[5] || (_cache[5] = ($event: any) => (failedAttachments.value = []))
                              }, [
                                _hoisted_10
                              ], 8 /* PROPS */, ["aria-label"])
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["content"])
                        ]),
                        (_unref(isExceedingAttachmentLimit))
                          ? (_openBlock(), _createElementBlock("div", {
                            key: 0,
                            id: "uploads-per-post",
                            "ps-2": "",
                            "sm:ps-1": "",
                            "text-small": ""
                          }, _toDisplayString(_ctx.$t('state.attachments_exceed_server_limit')), 1 /* TEXT */))
                          : _createCommentVNode("v-if", true),
                        _createElementVNode("ol", {
                          "ps-2": "",
                          "sm:ps-1": ""
                        }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(failedAttachments), (error) => {
                            return (_openBlock(), _createElementBlock("li", {
                              key: error[0],
                              flex: "~ col sm:row",
                              "gap-y-1": "",
                              "sm:gap-x-2": ""
                            }, [
                              _createElementVNode("strong", null, _toDisplayString(error[1]) + ":", 1 /* TEXT */),
                              _createElementVNode("span", null, _toDisplayString(error[0]), 1 /* TEXT */)
                            ]))
                          }), 128 /* KEYED_FRAGMENT */))
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["described-by"])) : _createCommentVNode("v-if", true), (draft.value.attachments.length) ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    flex: "~ col gap-2",
                    "overflow-auto": ""
                  }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(draft.value.attachments, (att, idx) => {
                      return (_openBlock(), _createBlock(_component_PublishAttachment, {
                        key: att.id,
                        attachment: att,
                        "dialog-labelled-by": __props.dialogLabelledBy ?? (draft.value.editingStatus ? 'state-editing' : undefined),
                        onRemove: _cache[6] || (_cache[6] = ($event: any) => (_unref(removeAttachment)(idx))),
                        onSetDescription: _cache[7] || (_cache[7] = ($event: any) => (_unref(setDescription)(att, $event)))
                      }, null, 8 /* PROPS */, ["attachment", "dialog-labelled-by"]))
                    }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true) ], 2 /* CLASS */) ]), _createElementVNode("div", {
              flex: "~ col 1",
              "max-w-full": ""
            }, [ (_unref(isExpanded) && draft.value.params.poll) ? (_openBlock(), _createElementBlock("form", {
                  key: 0,
                  "my-4": "",
                  flex: "~ 1 col",
                  "gap-3": "",
                  m: "s--1"
                }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(draft.value.params.poll.options, (option, index) => {
                    return (_openBlock(), _createElementBlock("div", {
                      key: index,
                      flex: "~ row",
                      "gap-3": ""
                    }, [
                      _createElementVNode("input", {
                        value: option,
                        "bg-base": "",
                        border: "~ base",
                        "flex-1": "",
                        h10: "",
                        "pe-4": "",
                        "rounded-2": "",
                        "w-full": "",
                        flex: "~ row",
                        "items-center": "",
                        relative: "",
                        "focus-within:box-shadow-outline": "",
                        "gap-3": "",
                        "px-4": "",
                        "py-2": "",
                        placeholder: _ctx.$t('polls.option_placeholder', { current: index + 1, max: _ctx.currentInstance?.configuration?.polls.maxOptions }),
                        class: "option-input",
                        onInput: _cache[8] || (_cache[8] = ($event: any) => (editPollOptionDraft($event, index)))
                      }, null, 40 /* PROPS, NEED_HYDRATION */, ["value", "placeholder"]),
                      _createVNode(_component_CommonTooltip, {
                        placement: "top",
                        content: _ctx.$t('polls.remove_option'),
                        class: "delete-button"
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("button", {
                            "btn-action-icon": "",
                            class: "hover:bg-red/75",
                            disabled: index === draft.value.params.poll.options.length - 1 && (index + 1 !== _ctx.currentInstance?.configuration?.polls.maxOptions || draft.value.params.poll.options[index].length === 0),
                            onClick: _cache[9] || (_cache[9] = _withModifiers(($event: any) => (deletePollOption(index)), ["prevent"]))
                          }, [
                            _hoisted_11
                          ], 8 /* PROPS */, ["disabled"])
                        ]),
                        _: 2 /* DYNAMIC */
                      }, 8 /* PROPS */, ["content"]),
                      (_ctx.currentInstance?.configuration?.polls.maxCharactersPerOption)
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          class: "char-limit-radial",
                          "aspect-ratio-1": "",
                          "h-10": "",
                          style: _normalizeStyle({ background: `radial-gradient(closest-side, rgba(var(--rgb-bg-base)) 79%, transparent 80% 100%), conic-gradient(${draft.value.params.poll.options[index].length / _ctx.currentInstance?.configuration?.polls.maxCharactersPerOption > 1 ? "var(--c-danger)" : "var(--c-primary)"} ${draft.value.params.poll.options[index].length / _ctx.currentInstance?.configuration?.polls.maxCharactersPerOption * 100}%, var(--c-primary-fade) 0)` })
                        }, _toDisplayString(draft.value.params.poll.options[index].length), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ]))
                  }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (hasQuote.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("div", {
                    flex: "",
                    "justify-end": "",
                    "mt-2": ""
                  }, [ _createElementVNode("button", {
                      "text-sm": "",
                      "px-2": "",
                      "py-1": "",
                      "rounded-3": "",
                      "hover:bg-gray-300": "",
                      flex: "~ gap1",
                      "items-center": "",
                      "aria-label": _ctx.$t('action.remove_quote'),
                      onClick: removeQuote
                    }, [ _hoisted_12, _createTextVNode("\n                " + _toDisplayString(_ctx.$t('action.remove_quote')), 1 /* TEXT */) ], 8 /* PROPS */, ["aria-label"]) ]), (quotedStatus.value) ? (_openBlock(), _createElementBlock("blockquote", {
                      key: 0,
                      border: "~ base 1",
                      "rounded-lg": "",
                      "overflow-hidden": "",
                      "my-3": ""
                    }, [ _createVNode(_component_StatusCard, {
                        status: quotedStatus.value,
                        actions: false,
                        "is-nested": true
                      }, null, 8 /* PROPS */, ["status", "actions", "is-nested"]) ])) : (quoteFetchError.value) ? (_openBlock(), _createElementBlock("div", {
                        key: 1,
                        "text-danger": "",
                        border: "base 1",
                        "rounded-lg": "",
                        "hover:bg-active": "",
                        "my-3": "",
                        "p-3": ""
                      }, _toDisplayString(_ctx.$t('error.quote_fetch_error')) + " (" + _toDisplayString(quoteFetchError.value) + ")\n            ", 1 /* TEXT */)) : (_openBlock(), _createBlock(_component_StatusCardSkeleton, {
                      key: 2,
                      border: "base 1",
                      "rounded-lg": "",
                      "hover:bg-active": "",
                      "my-3": ""
                    })) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n          " + "\n          "), (_unref(shouldExpanded)) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  flex: "~ gap-1 1 wrap",
                  m: "s--1",
                  "pt-2": "",
                  justify: "end",
                  "max-w-full": "",
                  border: "t base"
                }, [ _createVNode(_component_PublishEmojiPicker, {
                    onSelect: insertEmoji,
                    onSelectCustom: insertCustomEmoji
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("button", {
                        "btn-action-icon": "",
                        title: _ctx.$t('tooltip.emojis'),
                        "aria-label": _ctx.$t('tooltip.add_emojis')
                      }, [
                        _hoisted_13
                      ], 8 /* PROPS */, ["title", "aria-label"])
                    ]),
                    _: 1 /* STABLE */
                  }), (draft.value.params.poll === undefined) ? (_openBlock(), _createBlock(_component_CommonTooltip, {
                      key: 0,
                      placement: "top",
                      content: _ctx.$t('tooltip.add_media')
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("button", {
                          "btn-action-icon": "",
                          "aria-label": _ctx.$t('tooltip.add_media'),
                          onClick: _cache[10] || (_cache[10] = (...args) => (pickAttachments && pickAttachments(...args)))
                        }, [
                          _hoisted_14
                        ], 8 /* PROPS */, ["aria-label"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["content"])) : _createCommentVNode("v-if", true), (draft.value.attachments.length === 0) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (!draft.value.params.poll) ? (_openBlock(), _createBlock(_component_CommonTooltip, {
                          key: 0,
                          placement: "top",
                          content: _ctx.$t('polls.create')
                        }, {
                          default: _withCtx(() => [
                            _createElementVNode("button", {
                              "btn-action-icon": "",
                              "aria-label": _ctx.$t('polls.create'),
                              onClick: _cache[11] || (_cache[11] = ($event: any) => (draft.value.params.poll = { options: [''], expiresIn: expiresInOptions.value[expiresInDefaultOptionIndex].seconds }))
                            }, [
                              _hoisted_15
                            ], 8 /* PROPS */, ["aria-label"])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["content"])) : (_openBlock(), _createElementBlock("div", {
                          key: 1,
                          "rounded-full": "",
                          "b-1": "",
                          "border-dark": "",
                          flex: "~ row",
                          "gap-1": ""
                        }, [ _createVNode(_component_CommonTooltip, {
                            placement: "top",
                            content: _ctx.$t('polls.cancel')
                          }, {
                            default: _withCtx(() => [
                              _createElementVNode("button", {
                                "btn-action-icon": "",
                                "b-r": "",
                                "border-dark": "",
                                "aria-label": _ctx.$t('polls.cancel'),
                                onClick: _cache[12] || (_cache[12] = ($event: any) => (draft.value.params.poll = undefined))
                              }, [
                                _hoisted_16
                              ], 8 /* PROPS */, ["aria-label"])
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["content"]), _createVNode(_component_CommonDropdown, { placement: "top" }, {
                            popper: _withCtx(() => [
                              _createElementVNode("div", {
                                flex: "~ col",
                                "gap-1": "",
                                "p-2": ""
                              }, [
                                _createVNode(_component_CommonCheckbox, {
                                  label: draft.value.params.poll.multiple ? _ctx.$t('polls.disallow_multiple') : _ctx.$t('polls.allow_multiple'),
                                  "px-2": "",
                                  "gap-3": "",
                                  "h-9": "",
                                  flex: "",
                                  "justify-center": "",
                                  "hover:bg-active": "",
                                  "rounded-full": "",
                                  "icon-checked": "i-ri:checkbox-multiple-blank-line",
                                  "icon-unchecked": "i-ri:checkbox-blank-circle-line",
                                  modelValue: draft.value.params.poll.multiple,
                                  "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event: any) => ((draft.value.params.poll.multiple) = $event))
                                }, null, 8 /* PROPS */, ["label", "modelValue"]),
                                _createVNode(_component_CommonCheckbox, {
                                  label: draft.value.params.poll.hideTotals ? _ctx.$t('polls.show_votes') : _ctx.$t('polls.hide_votes'),
                                  "px-2": "",
                                  "gap-3": "",
                                  "h-9": "",
                                  flex: "",
                                  "justify-center": "",
                                  "hover:bg-active": "",
                                  "rounded-full": "",
                                  "icon-checked": "i-ri:eye-close-line",
                                  "icon-unchecked": "i-ri:eye-line",
                                  modelValue: draft.value.params.poll.hideTotals,
                                  "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event: any) => ((draft.value.params.poll.hideTotals) = $event))
                                }, null, 8 /* PROPS */, ["label", "modelValue"])
                              ])
                            ]),
                            default: _withCtx(() => [
                              _createVNode(_component_CommonTooltip, {
                                placement: "top",
                                content: _ctx.$t('polls.settings')
                              }, {
                                default: _withCtx(() => [
                                  _createElementVNode("button", {
                                    "aria-label": _ctx.$t('polls.settings'),
                                    "btn-action-icon": "",
                                    "w-12": ""
                                  }, [
                                    _hoisted_17,
                                    _hoisted_18
                                  ], 8 /* PROPS */, ["aria-label"])
                                ]),
                                _: 1 /* STABLE */
                              }, 8 /* PROPS */, ["content"])
                            ]),
                            _: 1 /* STABLE */
                          }), _createVNode(_component_CommonDropdown, { placement: "bottom" }, {
                            popper: _withCtx(() => [
                              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(expiresInOptions.value, (expiresInOption) => {
                                return (_openBlock(), _createBlock(_component_CommonDropdownItem, {
                                  key: expiresInOption.seconds,
                                  text: expiresInOption.label,
                                  checked: draft.value.params.poll.expiresIn === expiresInOption.seconds,
                                  onClick: _cache[15] || (_cache[15] = ($event: any) => (draft.value.params.poll.expiresIn = expiresInOption.seconds))
                                }, null, 8 /* PROPS */, ["text", "checked"]))
                              }), 128 /* KEYED_FRAGMENT */))
                            ]),
                            default: _withCtx(() => [
                              _createVNode(_component_CommonTooltip, {
                                placement: "top",
                                content: _ctx.$t('polls.expiration')
                              }, {
                                default: _withCtx(() => [
                                  _createElementVNode("button", {
                                    "aria-label": _ctx.$t('polls.expiration'),
                                    "btn-action-icon": "",
                                    "w-12": ""
                                  }, [
                                    _hoisted_19,
                                    _hoisted_20
                                  ], 8 /* PROPS */, ["aria-label"])
                                ]),
                                _: 1 /* STABLE */
                              }, 8 /* PROPS */, ["content"])
                            ]),
                            _: 1 /* STABLE */
                          }) ])) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), (_unref(editor)) ? (_openBlock(), _createBlock(_component_PublishEditorTools, {
                      key: 0,
                      editor: _unref(editor)
                    }, null, 8 /* PROPS */, ["editor"])) : _createCommentVNode("v-if", true), _createVNode(_component_CommonDropdown, {
                    placement: "bottom",
                    onClick: setInitialScheduledTime
                  }, {
                    popper: _withCtx(() => [
                      _withDirectives(_createElementVNode("input", {
                        "onUpdate:modelValue": [($event: any) => ((scheduledTime).value = $event), ($event: any) => ((scheduledTime).value = $event)],
                        p2: "",
                        type: "datetime-local",
                        name: "schedule-datetime",
                        min: getDatetimeInputFormat(minimumScheduledTime.value)
                      }, null, 8 /* PROPS */, ["min"]), [
                        [_vModelText, scheduledTime.value]
                      ])
                    ]),
                    default: _withCtx(() => [
                      _createVNode(_component_CommonTooltip, {
                        placement: "top",
                        content: _ctx.$t('tooltip.schedule_post'),
                        "no-auto-focus": ""
                      }, {
                        default: _withCtx(() => [
                          _createElementVNode("button", {
                            "btn-action-icon": "",
                            "aria-label": _ctx.$t('tooltip.schedule_post')
                          }, [
                            _createElementVNode("div", {
                              "i-ri:calendar-schedule-line": "",
                              class: _normalizeClass(scheduledTime.value !== '' ? 'text-primary' : '')
                            }, null, 2 /* CLASS */)
                          ], 8 /* PROPS */, ["aria-label"])
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["content"])
                    ]),
                    _: 1 /* STABLE */
                  }), _hoisted_21, _createVNode(_component_PublishCharacterCounter, {
                    max: _ctx.characterLimit,
                    length: characterCount.value
                  }, null, 8 /* PROPS */, ["max", "length"]), _createVNode(_component_CommonTooltip, {
                    placement: "top",
                    content: _ctx.$t('tooltip.change_language')
                  }, {
                    default: _withCtx(() => [
                      _createVNode(_component_CommonDropdown, {
                        placement: "bottom",
                        "auto-boundary-max-size": ""
                      }, {
                        popper: _withCtx(() => [
                          _createVNode(_component_PublishLanguagePicker, {
                            "min-w-80": "",
                            modelValue: draft.value.params.language,
                            "onUpdate:modelValue": _cache[16] || (_cache[16] = ($event: any) => ((draft.value.params.language) = $event))
                          }, null, 8 /* PROPS */, ["modelValue"])
                        ]),
                        default: _withCtx(() => [
                          _createElementVNode("button", {
                            "btn-action-icon": "",
                            "aria-label": _ctx.$t('tooltip.change_language'),
                            "w-max": "",
                            mr1: ""
                          }, [
                            (postLanguageDisplay.value)
                              ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                "text-secondary": "",
                                "text-sm": "",
                                ml1: ""
                              }, _toDisplayString(postLanguageDisplay.value), 1 /* TEXT */))
                              : (_openBlock(), _createElementBlock("div", {
                                key: 1,
                                "i-ri:translate-2": ""
                              })),
                            _hoisted_22
                          ], 8 /* PROPS */, ["aria-label"])
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["content"]), _createVNode(_component_CommonTooltip, {
                    placement: "top",
                    content: _ctx.$t('tooltip.add_content_warning')
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("button", {
                        "btn-action-icon": "",
                        "aria-label": _ctx.$t('tooltip.add_content_warning'),
                        onClick: toggleSensitive
                      }, [
                        (draft.value.params.sensitive)
                          ? (_openBlock(), _createElementBlock("div", {
                            key: 0,
                            "i-ri:alarm-warning-fill": "",
                            "text-orange": ""
                          }))
                          : (_openBlock(), _createElementBlock("div", {
                            key: 1,
                            "i-ri:alarm-warning-line": ""
                          }))
                      ], 8 /* PROPS */, ["aria-label"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["content"]), _createVNode(_component_PublishVisibilityPicker, {
                    editing: !!draft.value.editingStatus,
                    modelValue: draft.value.params.visibility,
                    "onUpdate:modelValue": _cache[17] || (_cache[17] = ($event: any) => ((draft.value.params.visibility) = $event))
                  }, {
                    default: _withCtx(({ visibility }) => [
                      _createElementVNode("button", {
                        disabled: !!draft.value.editingStatus,
                        "aria-label": _ctx.$t('tooltip.change_content_visibility'),
                        "btn-action-icon": "",
                        class: _normalizeClass({ 'w-12': !draft.value.editingStatus })
                      }, [
                        _createElementVNode("div", {
                          class: _normalizeClass(visibility.icon)
                        }),
                        (!draft.value.editingStatus)
                          ? (_openBlock(), _createElementBlock("div", {
                            key: 0,
                            "i-ri:arrow-down-s-line": "",
                            "text-sm": "",
                            "text-secondary": "",
                            "me--1": ""
                          }))
                          : _createCommentVNode("v-if", true)
                      ], 10 /* CLASS, PROPS */, ["disabled", "aria-label"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["editing", "modelValue"]), (hasQuote.value) ? (_openBlock(), _createBlock(_component_PublishQuoteApprovalPicker, {
                      key: 0,
                      editing: !!draft.value.editingStatus,
                      modelValue: draft.value.params.quoteApprovalPolicy,
                      "onUpdate:modelValue": _cache[18] || (_cache[18] = ($event: any) => ((draft.value.params.quoteApprovalPolicy) = $event))
                    }, {
                      default: _withCtx(({ quoteApprovalPolicy }) => [
                        _createElementVNode("button", {
                          disabled: !!draft.value.editingStatus,
                          "aria-label": _ctx.$t('tooltip.change_content_visibility'),
                          "btn-action-icon": "",
                          class: _normalizeClass({ 'w-12': !draft.value.editingStatus })
                        }, [
                          _createElementVNode("div", {
                            class: _normalizeClass(quoteApprovalPolicy.icon)
                          }),
                          (!draft.value.editingStatus)
                            ? (_openBlock(), _createElementBlock("div", {
                              key: 0,
                              "i-ri:arrow-down-s-line": "",
                              "text-sm": "",
                              "text-secondary": "",
                              "me--1": ""
                            }))
                            : _createCommentVNode("v-if", true)
                        ], 10 /* CLASS, PROPS */, ["disabled", "aria-label"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["editing", "modelValue"])) : _createCommentVNode("v-if", true), _createVNode(_component_PublishThreadTools, {
                    "draft-item-index": __props.draftItemIndex,
                    "draft-key": __props.draftKey
                  }, null, 8 /* PROPS */, ["draft-item-index", "draft-key"]), (_unref(failedMessages).length > 0) ? (_openBlock(), _createBlock(_component_CommonTooltip, {
                      key: 0,
                      id: "publish-failed-tooltip",
                      placement: "top",
                      content: scheduledTime.value ? _ctx.$t('state.schedule_failed') : _ctx.$t('tooltip.publish_failed')
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("button", {
                          "btn-danger": "",
                          "rounded-3": "",
                          "text-sm": "",
                          "w-full": "",
                          flex: "~ gap1",
                          "items-center": "",
                          "md:w-fit": "",
                          "aria-describedby": "publish-failed-tooltip"
                        }, [
                          _createElementVNode("span", { block: "" }, [
                            _hoisted_23
                          ]),
                          _createElementVNode("span", null, _toDisplayString(scheduledTime.value ? _ctx.$t('state.schedule_failed') : _ctx.$t('state.publish_failed')), 1 /* TEXT */)
                        ])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["content"])) : (_openBlock(), _createBlock(_component_CommonTooltip, {
                      key: 1,
                      id: "publish-tooltip",
                      placement: "top",
                      content: _ctx.$t('tooltip.add_publishable_content'),
                      disabled: !(_unref(isPublishDisabled) || isExceedingCharacterLimit.value)
                    }, {
                      default: _withCtx(() => [
                        (!_unref(threadIsActive) || isFinalItemOfThread.value)
                          ? (_openBlock(), _createElementBlock("button", {
                            key: 0,
                            "btn-solid": "",
                            "rounded-3": "",
                            "text-sm": "",
                            "w-full": "",
                            flex: "~ gap1",
                            "items-center": "",
                            "md:w-fit": "",
                            class: "publish-button",
                            "aria-disabled": _unref(isPublishDisabled) || isExceedingCharacterLimit.value || _unref(threadIsSending) || !isValidScheduledTime.value,
                            "aria-describedby": "publish-tooltip",
                            disabled: _unref(isPublishDisabled) || isExceedingCharacterLimit.value || _unref(threadIsSending) || !isValidScheduledTime.value,
                            onClick: publish
                          }, [
                            (_unref(isSending) || _unref(threadIsSending))
                              ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                block: "",
                                "animate-spin": "",
                                "preserve-3d": ""
                              }, [
                                _hoisted_24
                              ]))
                              : _createCommentVNode("v-if", true),
                            (_unref(failedMessages).length)
                              ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                block: ""
                              }, [
                                _hoisted_25
                              ]))
                              : _createCommentVNode("v-if", true),
                            (_unref(threadIsActive))
                              ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(!_unref(threadIsSending) ? _ctx.$t('action.publish_thread') : _ctx.$t('state.publishing')), 1 /* TEXT */))
                              : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                                (draft.value.editingStatus)
                                  ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(_ctx.$t('action.save_changes')), 1 /* TEXT */))
                                  : (scheduledTime.value)
                                    ? (_openBlock(), _createElementBlock("span", { key: 1 }, _toDisplayString(!_unref(isSending) ? _ctx.$t('action.schedule') : _ctx.$t('state.scheduling')), 1 /* TEXT */))
                                  : (draft.value.params.inReplyToId)
                                    ? (_openBlock(), _createElementBlock("span", { key: 2 }, _toDisplayString(_ctx.$t('action.reply')), 1 /* TEXT */))
                                  : (_openBlock(), _createElementBlock("span", { key: 3 }, _toDisplayString(!_unref(isSending) ? _ctx.$t('action.publish') : _ctx.$t('state.publishing')), 1 /* TEXT */))
                              ], 64 /* STABLE_FRAGMENT */))
                          ]))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["content", "disabled"])) ])) : _createCommentVNode("v-if", true) ]) ]) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
