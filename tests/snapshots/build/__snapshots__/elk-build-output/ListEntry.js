import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText, withModifiers as _withModifiers, withKeys as _withKeys } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "text-current": "true", "i-ri:close-fill": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true", "aria-hidden": "true" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "text-current": "true", "i-ri:edit-2-line": "true", class: "rtl-flip" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-2-fill": "true", "aria-hidden": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", { "aria-hidden": "true", "i-ri:error-warning-fill": "true" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", w: "1.75em", h: "1.75em", "i-ri:close-line": "true" })
const _hoisted_7 = { "sr-only": "true" }
import type { mastodon } from 'masto'
import { useForm } from 'slimeform'

export default /*@__PURE__*/_defineComponent({
  __name: 'ListEntry',
  props: {
    "modelValue": { required: true },
    "modelModifiers": {},
  },
  emits: ["listUpdated", "listRemoved", "update:modelValue"],
  setup(__props, { emit: __emit }) {

const emit = __emit
const list = _useModel(__props, "modelValue")
const { t } = useI18n()
const client = useMastoClient()
const { form, isDirty, submitter, reset } = useForm({
  form: () => ({ ...list.value }),
})
const isEditing = ref<boolean>(false)
const deleting = ref<boolean>(false)
const actionError = ref<string | undefined>(undefined)
const input = ref<HTMLInputElement>()
const editBtn = ref<HTMLButtonElement>()
const deleteBtn = ref<HTMLButtonElement>()
async function prepareEdit() {
  isEditing.value = true
  actionError.value = undefined
  await nextTick()
  input.value?.focus()
}
async function cancelEdit() {
  isEditing.value = false
  actionError.value = undefined
  await nextTick()
  reset()
  editBtn.value?.focus()
}
const { submit, submitting } = submitter(async () => {
  try {
    list.value = await client.v1.lists.$select(form.id).update({
      title: form.title,
    })
    cancelEdit()
  }
  catch (err) {
    console.error(err)
    actionError.value = (err as Error).message
    await nextTick()
    input.value?.focus()
  }
})
async function removeList() {
  if (deleting.value)
    return
  const confirmDelete = await openConfirmDialog({
    title: t('confirm.delete_list.title'),
    description: t('confirm.delete_list.description', [list.value.title]),
    confirm: t('confirm.delete_list.confirm'),
    cancel: t('confirm.delete_list.cancel'),
  })
  deleting.value = true
  actionError.value = undefined
  await nextTick()
  if (confirmDelete.choice === 'confirm') {
    await nextTick()
    try {
      await client.v1.lists.$select(list.value.id).remove()
      emit('listRemoved', list.value.id)
    }
    catch (err) {
      console.error(err)
      actionError.value = (err as Error).message
      await nextTick()
      deleteBtn.value?.focus()
    }
    finally {
      deleting.value = false
    }
  }
  else {
    deleting.value = false
  }
}
async function clearError() {
  actionError.value = undefined
  await nextTick()
  if (isEditing.value)
    input.value?.focus()
  else
    deleteBtn.value?.focus()
}
onDeactivated(cancelEdit)

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_CommonErrorMessage = _resolveComponent("CommonErrorMessage")

  return (_openBlock(), _createElementBlock(_Fragment, null, [ _createElementVNode("form", {
        "hover:bg-active": "",
        flex: "",
        "justify-between": "",
        "items-center": "",
        "gap-x-2": "",
        "aria-describedby": actionError.value ? `action-list-error-${list.value.id}` : undefined,
        class: _normalizeClass(actionError.value ? 'border border-base border-rounded rounded-be-is-0 rounded-be-ie-0 border-b-unset border-$c-danger-active' : null),
        onSubmit: _cache[0] || (_cache[0] = _withModifiers((...args) => (submit && submit(...args)), ["prevent"]))
      }, [ (isEditing.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            "bg-base": "",
            border: "~ base",
            h10: "",
            m2: "",
            "ps-1": "",
            "pe-4": "",
            "rounded-3": "",
            "w-full": "",
            flex: "~ row",
            "items-center": "",
            relative: "",
            "focus-within:box-shadow-outline": "",
            "gap-3": ""
          }, [ (isEditing.value) ? (_openBlock(), _createBlock(_component_CommonTooltip, {
                key: 0,
                content: _ctx.$t('list.cancel_edit')
              }, {
                default: _withCtx(() => [
                  _createElementVNode("button", {
                    type: "button",
                    "rounded-full": "",
                    "text-sm": "",
                    p2: "",
                    "transition-colors": "",
                    "hover:text-primary": "",
                    onClick: _cache[1] || (_cache[1] = ($event: any) => (cancelEdit()))
                  }, [
                    _hoisted_1
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["content"])) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode("input", {
              ref_key: "input", ref: input,
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((_unref(form).title) = $event)),
              "rounded-3": "",
              "w-full": "",
              "bg-transparent": "",
              outline: "focus:none",
              "pe-4": "",
              pb: "1px",
              "flex-1": "",
              "placeholder-text-secondary": "",
              onKeydown: _cache[3] || (_cache[3] = _withKeys(($event: any) => (cancelEdit()), ["esc"]))
            }, null, 32 /* NEED_HYDRATION */), [ [_vModelText, _unref(form).title] ]) ])) : (_openBlock(), _createBlock(_component_NuxtLink, {
            key: 1,
            to: `list/${list.value.id}`,
            block: "",
            grow: "",
            p4: ""
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(form).title), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"])), _createElementVNode("div", {
          mr4: "",
          flex: "",
          gap2: ""
        }, [ (isEditing.value) ? (_openBlock(), _createBlock(_component_CommonTooltip, {
              key: 0,
              content: _ctx.$t('list.save')
            }, {
              default: _withCtx(() => [
                _createElementVNode("button", {
                  type: "submit",
                  "text-sm": "",
                  p2: "",
                  "border-1": "",
                  "transition-colors": "",
                  "border-dark": "",
                  "hover:text-primary": "",
                  "btn-action-icon": "",
                  disabled: deleting.value || !_unref(isDirty) || _unref(submitting)
                }, [
                  (isEditing.value)
                    ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                      (_unref(submitting))
                        ? (_openBlock(), _createElementBlock("span", {
                          key: 0,
                          "aria-hidden": "true",
                          block: "",
                          animate: "",
                          "animate-spin": "",
                          "preserve-3d": "",
                          class: "rtl-flip"
                        }, [
                          _hoisted_2
                        ]))
                        : (_openBlock(), _createElementBlock("span", {
                          key: 1,
                          block: "",
                          "text-current": "",
                          "i-ri:save-2-fill": "",
                          class: "rtl-flip"
                        }))
                    ], 64 /* STABLE_FRAGMENT */))
                    : _createCommentVNode("v-if", true)
                ], 8 /* PROPS */, ["disabled"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["content"])) : (_openBlock(), _createBlock(_component_CommonTooltip, {
              key: 1,
              content: _ctx.$t('list.edit')
            }, {
              default: _withCtx(() => [
                _createElementVNode("button", {
                  ref_key: "editBtn", ref: editBtn,
                  type: "button",
                  "text-sm": "",
                  p2: "",
                  "border-1": "",
                  "transition-colors": "",
                  "border-dark": "",
                  "hover:text-primary": "",
                  "btn-action-icon": "",
                  onClick: _withModifiers(prepareEdit, ["prevent"])
                }, [
                  _hoisted_3
                ], 512 /* NEED_PATCH */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["content"])), _createVNode(_component_CommonTooltip, { content: _ctx.$t('list.delete') }, {
            default: _withCtx(() => [
              _createElementVNode("button", {
                type: "button",
                "text-sm": "",
                p2: "",
                "border-1": "",
                "transition-colors": "",
                "border-dark": "",
                "hover:text-primary": "",
                "btn-action-icon": "",
                disabled: isEditing.value,
                onClick: _withModifiers(removeList, ["prevent"])
              }, [
                (deleting.value)
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    "aria-hidden": "true",
                    block: "",
                    animate: "",
                    "animate-spin": "",
                    "preserve-3d": "",
                    class: "rtl-flip"
                  }, [
                    _hoisted_4
                  ]))
                  : (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    block: "",
                    "text-current": "",
                    "i-ri:delete-bin-2-line": "",
                    class: "rtl-flip"
                  }))
              ], 8 /* PROPS */, ["disabled"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["content"]) ]) ], 42 /* CLASS, PROPS, NEED_HYDRATION */, ["aria-describedby"]), (actionError.value) ? (_openBlock(), _createBlock(_component_CommonErrorMessage, {
          key: 0,
          id: `action-list-error-${list.value.id}`,
          "described-by": `action-list-failed-${list.value.id}`,
          class: "rounded-bs-is-0 rounded-bs-ie-0 border-t-dashed m-b-2"
        }, {
          default: _withCtx(() => [
            _createElementVNode("header", {
              id: `action-list-failed-${list.value.id}`,
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
                _createElementVNode("p", null, _toDisplayString(_ctx.$t(`list.${isEditing.value ? 'edit_error' : 'delete_error'}`)), 1 /* TEXT */)
              ]),
              _createVNode(_component_CommonTooltip, {
                placement: "bottom",
                content: _ctx.$t('list.clear_error')
              }, {
                default: _withCtx(() => [
                  _createElementVNode("button", {
                    flex: "",
                    "rounded-4": "",
                    p1: "",
                    "hover:bg-active": "",
                    "cursor-pointer": "",
                    "transition-100": "",
                    "aria-label": _ctx.$t('list.clear_error'),
                    onClick: clearError
                  }, [
                    _hoisted_6
                  ], 8 /* PROPS */, ["aria-label"])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["content"])
            ], 8 /* PROPS */, ["id"]),
            _createElementVNode("ol", {
              "ps-2": "",
              "sm:ps-1": ""
            }, [
              _createElementVNode("li", {
                flex: "~ col sm:row",
                "gap-y-1": "",
                "sm:gap-x-2": ""
              }, [
                _createElementVNode("strong", _hoisted_7, _toDisplayString(_ctx.$t('list.error_prefix')), 1 /* TEXT */),
                _createElementVNode("span", null, _toDisplayString(actionError.value), 1 /* TEXT */)
              ])
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["id", "described-by"])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */))
}
}

})
