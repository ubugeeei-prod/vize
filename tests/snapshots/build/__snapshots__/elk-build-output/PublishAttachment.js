import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, withCtx as _withCtx, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { "i-ri:close-line": "true", "text-3": "true", "text-6": "true", "md:text-3": "true" })
const _hoisted_2 = { id: "edit-attachment", "font-bold": "true" }
import type { mastodon } from 'masto'
const maxDescriptionLength = 1500

export default /*@__PURE__*/_defineComponent({
  __name: 'PublishAttachment',
  props: {
    attachment: { type: null, required: true },
    alt: { type: String, required: false },
    removable: { type: Boolean, required: false, default: true },
    dialogLabelledBy: { type: String, required: false }
  },
  emits: ["remove", "setDescription"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
// from https://github.com/mastodon/mastodon/blob/dfa984/app/models/media_attachment.rb#L40
const isEditDialogOpen = ref(false)
const description = ref(__props.attachment.description ?? '')
function toggleApply() {
  isEditDialogOpen.value = false
  emit('setDescription', description.value)
}

return (_ctx: any,_cache: any) => {
  const _component_StatusAttachment = _resolveComponent("StatusAttachment")
  const _component_PublishCharacterCounter = _resolveComponent("PublishCharacterCounter")
  const _component_ModalDialog = _resolveComponent("ModalDialog")

  return (_openBlock(), _createElementBlock("div", {
      relative: "",
      group: ""
    }, [ _createVNode(_component_StatusAttachment, {
        attachment: __props.attachment,
        "w-full": "",
        "is-preview": ""
      }, null, 8 /* PROPS */, ["attachment"]), _createElementVNode("div", {
        absolute: "",
        "right-2": "",
        "top-2": ""
      }, [ (__props.removable) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            "aria-label": _ctx.$t('attachment.remove_label'),
            class: "bg-black/75 hover:bg-red/75",
            "text-white": "",
            px2: "",
            py2: "",
            "rounded-full": "",
            "cursor-pointer": "",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (_ctx.$emit('remove')))
          }, [ _hoisted_1 ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
        absolute: "",
        "right-2": "",
        "bottom-2": ""
      }, [ _createElementVNode("button", {
          class: "bg-black/75",
          "text-white": "",
          px2: "",
          py1: "",
          "rounded-2": "",
          onClick: _cache[1] || (_cache[1] = ($event: any) => (isEditDialogOpen.value = true))
        }, _toDisplayString(_ctx.$t('action.edit')), 1 /* TEXT */) ]), _createVNode(_component_ModalDialog, {
        "dialog-labelled-by": __props.dialogLabelledBy,
        "py-6": "",
        "px-6": "",
        "max-w-300": "",
        modelValue: isEditDialogOpen.value,
        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((isEditDialogOpen).value = $event))
      }, {
        default: _withCtx(() => [
          _createElementVNode("div", {
            flex: "",
            "flex-col-reverse": "",
            "gap-5": "",
            "md:flex-row": ""
          }, [
            _createElementVNode("div", {
              flex: "",
              "flex-col": "",
              "gap-2": "",
              "justify-between": ""
            }, [
              _createElementVNode("h1", _hoisted_2, _toDisplayString(_ctx.$t('attachment.edit_title')), 1 /* TEXT */),
              _createElementVNode("div", {
                flex: "",
                "flex-col": "",
                "gap-2": ""
              }, [
                _withDirectives(_createElementVNode("textarea", {
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((description).value = $event)),
                  "p-3": "",
                  "h-50": "",
                  "bg-base": "",
                  "rounded-2": "",
                  "border-strong": "",
                  "border-1": "",
                  "md:w-100": ""
                }, null, 512 /* NEED_PATCH */), [
                  [_vModelText, description.value]
                ]),
                _createElementVNode("div", {
                  flex: "",
                  "flex-row-reverse": ""
                }, [
                  _createVNode(_component_PublishCharacterCounter, {
                    length: description.value.length,
                    max: maxDescriptionLength
                  }, null, 8 /* PROPS */, ["length", "max"])
                ]),
                _createElementVNode("button", {
                  "btn-outline": "",
                  disabled: description.value.length > maxDescriptionLength,
                  onClick: toggleApply
                }, _toDisplayString(_ctx.$t('action.apply')), 9 /* TEXT, PROPS */, ["disabled"])
              ]),
              _createElementVNode("button", {
                "btn-outline": "",
                onClick: _cache[4] || (_cache[4] = ($event: any) => (isEditDialogOpen.value = false))
              }, _toDisplayString(_ctx.$t('action.close')), 1 /* TEXT */)
            ]),
            _createVNode(_component_StatusAttachment, {
              attachment: __props.attachment,
              "w-full": "",
              "is-preview": ""
            }, null, 8 /* PROPS */, ["attachment"])
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["dialog-labelled-by", "modelValue"]) ]))
}
}

})
