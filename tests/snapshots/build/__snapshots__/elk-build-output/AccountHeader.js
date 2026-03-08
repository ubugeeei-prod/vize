import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = { "text-primary": "true", "font-bold": "true" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "i-ri:play-list-add-fill": "true", block: "true", "text-current": "true" })
const _hoisted_3 = { "sr-only": "true" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { "i-ri-edit-2-line": "true" })
const _hoisted_5 = { "font-medium": "true" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { "text-secondary": "true", "text-xs": "true", "font-bold": "true" }, "|")
import type { mastodon } from 'masto'
const personalNoteMaxLength = 2000

export default /*@__PURE__*/_defineComponent({
  __name: 'AccountHeader',
  props: {
    account: { type: null, required: true },
    command: { type: Boolean, required: false }
  },
  setup(__props: any) {

const { client } = useMasto()
const { t } = useI18n()
const createdAt = useFormattedDateTime(() => __props.account.createdAt, {
  month: 'long',
  day: 'numeric',
  year: 'numeric',
})
const relationship = useRelationship(__props.account)
const namedFields = ref<mastodon.v1.AccountField[]>([])
const iconFields = ref<mastodon.v1.AccountField[]>([])
const isEditingPersonalNote = ref<boolean>(false)
const hasHeader = computed(() => !__props.account.header.endsWith('/original/missing.png'))
const isCopied = ref<boolean>(false)
function getFieldIconTitle(fieldName: string) {
  return fieldName === 'Joined' ? t('account.joined') : fieldName
}
function getNotificationIconTitle() {
  return relationship.value?.notifying ? t('account.notifications_on_post_disable', { username: `@${__props.account.username}` }) : t('account.notifications_on_post_enable', { username: `@${__props.account.username}` })
}
function previewHeader() {
  openMediaPreview([{
    id: `${__props.account.acct}:header`,
    type: 'image',
    previewUrl: __props.account.header,
    description: t('account.profile_description', [__props.account.username]),
  }])
}
function previewAvatar() {
  openMediaPreview([{
    id: `${__props.account.acct}:avatar`,
    type: 'image',
    previewUrl: __props.account.avatar,
    description: t('account.avatar_description', [__props.account.username]),
  }])
}
async function toggleNotifications() {
  relationship.value!.notifying = !relationship.value?.notifying
  try {
    const newRel = await client.value.v1.accounts.$select(__props.account.id).follow({ notify: relationship.value?.notifying })
    Object.assign(relationship!, newRel)
  }
  catch {
    // TODO error handling
    relationship.value!.notifying = !relationship.value?.notifying
  }
}
watchEffect(() => {
  const named: mastodon.v1.AccountField[] = []
  const icons: mastodon.v1.AccountField[] = []
  __props.account.fields?.forEach((field) => {
    const icon = getAccountFieldIcon(field.name)
    if (icon)
      icons.push(field)
    else
      named.push(field)
  })
  icons.push({
    name: 'Joined',
    value: createdAt.value,
  })
  namedFields.value = named
  iconFields.value = icons
})
const personalNoteDraft = ref(relationship.value?.note ?? '')
watch(relationship, (relationship, oldValue) => {
  if (!oldValue && relationship)
    personalNoteDraft.value = relationship.note ?? ''
})
async function editNote(event: Event) {
  if (!event.target || !('value' in event.target) || !relationship.value)
    return
  const newNote = event.target?.value as string
  if (relationship.value.note?.trim() === newNote.trim())
    return
  const newNoteApiResult = await client.value.v1.accounts.$select(__props.account.id).note.create({ comment: newNote })
  relationship.value.note = newNoteApiResult.note
  personalNoteDraft.value = relationship.value.note ?? ''
}
const isSelf = useSelfAccount(() => __props.account)
const isNotifiedOnPost = computed(() => !!relationship.value?.notifying)
async function copyAccountName() {
  try {
    const shortHandle = getShortHandle(__props.account)
    const serverName = getServerName(__props.account)
    const accountName = `${shortHandle}@${serverName}`
    await navigator.clipboard.writeText(accountName)
  }
  catch (err) {
    console.error('Failed to copy account name:', err)
  }
  isCopied.value = true
  setTimeout(() => {
    isCopied.value = false
  }, 2000)
}

return (_ctx: any,_cache: any) => {
  const _component_AccountFollowRequestButton = _resolveComponent("AccountFollowRequestButton")
  const _component_AccountAvatar = _resolveComponent("AccountAvatar")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_AccountFollowButton = _resolveComponent("AccountFollowButton")
  const _component_AccountMoreButton = _resolveComponent("AccountMoreButton")
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_ListLists = _resolveComponent("ListLists")
  const _component_VDropdown = _resolveComponent("VDropdown")
  const _component_AccountDisplayName = _resolveComponent("AccountDisplayName")
  const _component_AccountLockIndicator = _resolveComponent("AccountLockIndicator")
  const _component_AccountBotIndicator = _resolveComponent("AccountBotIndicator")
  const _component_AccountHandle = _resolveComponent("AccountHandle")
  const _component_AccountRolesIndicator = _resolveComponent("AccountRolesIndicator")
  const _component_ContentRich = _resolveComponent("ContentRich")
  const _component_AccountPostsFollowers = _resolveComponent("AccountPostsFollowers")

  return (_openBlock(), _createElementBlock("div", {
      flex: "",
      "flex-col": ""
    }, [ (_unref(relationship)?.requestedBy) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          "p-4": "",
          flex: "",
          "justify-between": "",
          "items-center": "",
          "bg-card": ""
        }, [ _createElementVNode("span", _hoisted_1, _toDisplayString(_ctx.$t('account.requested', [__props.account.displayName])), 1 /* TEXT */), _createVNode(_component_AccountFollowRequestButton, {
            account: __props.account,
            relationship: _unref(relationship)
          }, null, 8 /* PROPS */, ["account", "relationship"]) ])) : _createCommentVNode("v-if", true), _createVNode(_resolveDynamicComponent(hasHeader.value ? 'button' : 'div'), {
        border: "b base",
        "z-1": "",
        onClick: _cache[0] || (_cache[0] = ($event: any) => (hasHeader.value ? previewHeader() : undefined))
      }, {
        default: _withCtx(() => [
          _createElementVNode("img", {
            "h-50": "",
            height: "200",
            "w-full": "",
            "object-cover": "",
            src: __props.account.header,
            alt: _unref(t)('account.profile_description', [__props.account.username])
          }, null, 8 /* PROPS */, ["src", "alt"])
        ]),
        _: 1 /* STABLE */
      }), _createElementVNode("div", {
        p4: "",
        "mt--18": "",
        flex: "",
        "flex-col": "",
        "gap-4": ""
      }, [ _createElementVNode("div", { relative: "" }, [ _createElementVNode("div", {
            flex: "",
            "justify-between": ""
          }, [ _createElementVNode("button", {
              "shrink-0": "",
              "h-full": "",
              class: _normalizeClass({ 'rounded-full': !_unref(isSelf), 'squircle': _unref(isSelf) }),
              p1: "",
              "bg-base": "",
              "border-bg-base": "",
              "z-2": "",
              onClick: previewAvatar
            }, [ _createVNode(_component_AccountAvatar, {
                square: _unref(isSelf),
                account: __props.account,
                "hover:opacity-90": "",
                "transition-opacity": "",
                "w-28": "",
                "h-28": ""
              }, null, 8 /* PROPS */, ["square", "account"]) ], 2 /* CLASS */), _createElementVNode("div", {
              "inset-ie-0": "",
              flex: "~ wrap row-reverse",
              "gap-2": "",
              "items-center": "",
              pt18: "",
              "justify-start": ""
            }, [ (_unref(isSelf)) ? (_openBlock(), _createBlock(_component_NuxtLink, {
                  key: 0,
                  to: "/settings/profile/appearance",
                  "gap-1": "",
                  "items-center": "",
                  border: "1",
                  "rounded-full": "",
                  flex: "~ gap2 center",
                  "font-500": "",
                  "min-w-30": "",
                  "h-fit": "",
                  px3: "",
                  py1: "",
                  hover: "border-primary text-primary bg-active"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_ctx.$t('settings.profile.appearance.title')), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })) : _createCommentVNode("v-if", true), _createVNode(_component_AccountFollowButton, {
                account: __props.account,
                command: __props.command
              }, null, 8 /* PROPS */, ["account", "command"]), _createElementVNode("span", {
                "inset-ie-0": "",
                flex: "",
                "gap-2": "",
                "items-center": ""
              }, [ _createVNode(_component_AccountMoreButton, {
                  account: __props.account,
                  command: __props.command,
                  onAddNote: _cache[1] || (_cache[1] = ($event: any) => (isEditingPersonalNote.value = true)),
                  onRemoveNote: _cache[2] || (_cache[2] = () => { isEditingPersonalNote.value = false; personalNoteDraft.value = '' })
                }, null, 8 /* PROPS */, ["account", "command"]), (!_unref(isSelf) && _unref(relationship)?.following) ? (_openBlock(), _createBlock(_component_CommonTooltip, {
                    key: 0,
                    content: getNotificationIconTitle()
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("button", {
                        "aria-pressed": isNotifiedOnPost.value,
                        "aria-label": _unref(t)('account.notifications_on_post_enable', { username: `@${__props.account.username}` }),
                        "rounded-full": "",
                        "text-sm": "",
                        p2: "",
                        "border-1": "",
                        "transition-colors": "",
                        class: _normalizeClass(isNotifiedOnPost.value ? 'text-primary border-primary hover:bg-red/20 hover:text-red hover:border-red' : 'border-base hover:text-primary'),
                        onClick: toggleNotifications
                      }, [
                        (isNotifiedOnPost.value)
                          ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            "i-ri:notification-4-fill": "",
                            block: "",
                            "text-current": ""
                          }))
                          : (_openBlock(), _createElementBlock("span", {
                            key: 1,
                            "i-ri-notification-4-line": "",
                            block: "",
                            "text-current": ""
                          }))
                      ], 10 /* CLASS, PROPS */, ["aria-pressed", "aria-label"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["content"])) : _createCommentVNode("v-if", true), _createVNode(_component_CommonTooltip, { content: _ctx.$t('list.modify_account') }, {
                  default: _withCtx(() => [
                    (!_unref(isSelf) && _unref(relationship)?.following)
                      ? (_openBlock(), _createBlock(_component_VDropdown, { key: 0 }, {
                        popper: _withCtx(() => [
                          _createVNode(_component_ListLists, { "user-id": __props.account.id }, null, 8 /* PROPS */, ["user-id"])
                        ]),
                        default: _withCtx(() => [
                          _createElementVNode("button", {
                            "aria-label": _ctx.$t('list.modify_account'),
                            "rounded-full": "",
                            "text-sm": "",
                            p2: "",
                            "border-1": "",
                            "transition-colors": "",
                            "border-base": "",
                            "hover:text-primary": ""
                          }, [
                            _hoisted_2
                          ], 8 /* PROPS */, ["aria-label"])
                        ]),
                        _: 1 /* STABLE */
                      }))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["content"]) ]) ]) ]), _createElementVNode("div", {
            flex: "~ col gap1",
            pt2: ""
          }, [ _createElementVNode("div", {
              flex: "",
              gap2: "",
              "items-center": "",
              "flex-wrap": ""
            }, [ _createVNode(_component_AccountDisplayName, {
                account: __props.account,
                "font-bold": "",
                "sm:text-2xl": "",
                "text-xl": ""
              }, null, 8 /* PROPS */, ["account"]), (__props.account.locked) ? (_openBlock(), _createBlock(_component_AccountLockIndicator, {
                  key: 0,
                  "show-label": ""
                })) : _createCommentVNode("v-if", true), (__props.account.bot) ? (_openBlock(), _createBlock(_component_AccountBotIndicator, {
                  key: 0,
                  "show-label": ""
                })) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
              flex: "",
              "items-center": "",
              "gap-1": ""
            }, [ _createVNode(_component_AccountHandle, {
                account: __props.account,
                "overflow-unset": "",
                "line-clamp-unset": ""
              }, null, 8 /* PROPS */, ["account"]), _createVNode(_component_CommonTooltip, {
                placement: "bottom",
                content: _ctx.$t('account.copy_account_name'),
                flex: ""
              }, {
                default: _withCtx(() => [
                  _createElementVNode("button", {
                    "text-secondary-light": "",
                    "text-sm": "",
                    class: _normalizeClass(isCopied.value ? 'i-ri:check-fill text-green' : 'i-ri:file-copy-line'),
                    onClick: copyAccountName
                  }, [
                    _createElementVNode("span", _hoisted_3, _toDisplayString(_ctx.$t('account.copy_account_name')), 1 /* TEXT */)
                  ], 2 /* CLASS */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["content"]) ]), _createElementVNode("div", {
              "self-start": "",
              "mt-1": ""
            }, [ (__props.account.roles?.length) ? (_openBlock(), _createBlock(_component_AccountRolesIndicator, {
                  key: 0,
                  account: __props.account
                }, null, 8 /* PROPS */, ["account"])) : _createCommentVNode("v-if", true) ]) ]) ]), (isEditingPersonalNote.value || (_unref(relationship)?.note && _unref(relationship).note.length > 0)) ? (_openBlock(), _createElementBlock("label", {
            key: 0,
            "space-y-2": "",
            "pb-4": "",
            block: "",
            border: "b base"
          }, [ _createElementVNode("div", {
              flex: "",
              "flex-row": "",
              "space-x-2": "",
              "flex-v-center": ""
            }, [ _hoisted_4, _createElementVNode("p", _hoisted_5, _toDisplayString(_ctx.$t('account.profile_personal_note')), 1 /* TEXT */), _createElementVNode("p", {
                "text-secondary": "",
                "text-sm": "",
                class: _normalizeClass({ 'text-orange': personalNoteDraft.value.length > (personalNoteMaxLength - 100) })
              }, _toDisplayString(personalNoteDraft.value.length) + " / " + _toDisplayString(personalNoteMaxLength), 3 /* TEXT, CLASS */) ]), _createElementVNode("div", { "position-relative": "" }, [ _createElementVNode("div", {
                "input-base": "",
                "min-h-10ex": "",
                "whitespace-pre-wrap": "",
                "opacity-0": "",
                class: _normalizeClass({ 'trailing-newline': personalNoteDraft.value.endsWith('\n') })
              }, _toDisplayString(personalNoteDraft.value), 3 /* TEXT, CLASS */), _withDirectives(_createElementVNode("textarea", {
                "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((personalNoteDraft).value = $event)),
                "input-base": "",
                "position-absolute": "",
                style: "height: 100%",
                "top-0": "",
                "resize-none": "",
                maxlength: personalNoteMaxLength,
                onChange: editNote
              }, null, 40 /* PROPS, NEED_HYDRATION */, ["maxlength"]), [ [_vModelText, personalNoteDraft.value] ]) ]) ])) : _createCommentVNode("v-if", true), (__props.account.note) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            "max-h-100": "",
            "overflow-y-auto": ""
          }, [ _createVNode(_component_ContentRich, {
              "text-4": "",
              "text-base": "",
              content: __props.account.note,
              emojis: __props.account.emojis
            }, null, 8 /* PROPS */, ["content", "emojis"]) ])) : _createCommentVNode("v-if", true), (namedFields.value.length) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            flex: "~ col wrap gap1"
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(namedFields.value, (field) => {
              return (_openBlock(), _createElementBlock("div", {
                key: field.name,
                flex: "~ gap-1",
                "items-center": ""
              }, [
                _createElementVNode("div", {
                  mt: "0.5",
                  "text-secondary": "",
                  uppercase: "",
                  "text-xs": "",
                  "font-bold": ""
                }, [
                  _createVNode(_component_ContentRich, {
                    content: field.name,
                    emojis: __props.account.emojis
                  }, null, 8 /* PROPS */, ["content", "emojis"])
                ]),
                _hoisted_6,
                _createVNode(_component_ContentRich, {
                  content: field.value,
                  emojis: __props.account.emojis
                }, null, 8 /* PROPS */, ["content", "emojis"])
              ]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), (iconFields.value.length) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            flex: "~ wrap gap-2"
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(iconFields.value, (field) => {
              return (_openBlock(), _createElementBlock("div", {
                key: field.name,
                flex: "~ gap-1",
                px1: "",
                "items-center": "",
                class: _normalizeClass(`${field.verifiedAt ? 'border-1 rounded-full border-dark' : ''}`)
              }, [
                _createVNode(_component_CommonTooltip, { content: getFieldIconTitle(field.name) }, {
                  default: _withCtx(() => [
                    _createElementVNode("div", {
                      "text-secondary": "",
                      class: _normalizeClass(_ctx.getAccountFieldIcon(field.name)),
                      title: getFieldIconTitle(field.name)
                    }, null, 10 /* CLASS, PROPS */, ["title"])
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["content"]),
                _createVNode(_component_ContentRich, {
                  "text-sm": "",
                  content: field.value,
                  emojis: __props.account.emojis
                }, null, 8 /* PROPS */, ["content", "emojis"])
              ], 2 /* CLASS */))
            }), 128 /* KEYED_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), _createVNode(_component_AccountPostsFollowers, { account: __props.account }, null, 8 /* PROPS */, ["account"]) ]) ]))
}
}

})
