import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, vModelText as _vModelText } from "vue"


const _hoisted_1 = { "font-bold": "true", "text-lg": "true" }
const _hoisted_2 = { "text-sm": "true", "text-secondary": "true" }
import type { ConfirmDialogChoice, ConfirmDialogOptions } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'ModalConfirm',
  emits: ["choice"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const hasDuration = ref(false)
const isValidDuration = ref(true)
const duration = ref(60 * 60) // default to 1 hour
const shouldMuteNotifications = ref(true)
const isMute = computed(() => __props.extraOptionType === 'mute')
const domainInput = ref('')
const isBlockDomain = computed(() => __props.extraOptionType === 'block_domain')
const isDomainConfirmed = computed(() => domainInput.value === __props.domainToBlock)
function handleChoice(choice: ConfirmDialogChoice['choice']) {
  const dialogChoice: ConfirmDialogChoice = {
    choice,
  }
  if (isMute.value) {
    dialogChoice.extraOptions = {
      mute: {
        duration: hasDuration.value ? duration.value : 0,
        notifications: shouldMuteNotifications.value,
      },
    }
  }
  if (isBlockDomain.value) {
    dialogChoice.extraOptions = {
      ...dialogChoice.extraOptions,
      block_domain: {
        confirmed: isDomainConfirmed.value,
      },
    }
  }
  emit('choice', dialogChoice)
}

return (_ctx: any,_cache: any) => {
  const _component_CommonCheckbox = _resolveComponent("CommonCheckbox")
  const _component_ModalDurationPicker = _resolveComponent("ModalDurationPicker")

  return (_openBlock(), _createElementBlock("div", {
      flex: "~ col",
      "gap-6": ""
    }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_ctx.title), 1 /* TEXT */), (_ctx.description) ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_ctx.description), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (isMute.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          "flex-col": "",
          flex: "",
          "gap-4": ""
        }, [ _createVNode(_component_CommonCheckbox, {
            label: _ctx.$t('confirm.mute_account.specify_duration'),
            "prepend-checkbox": "",
            "checked-icon-color": "text-primary",
            modelValue: hasDuration.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((hasDuration).value = $event))
          }, null, 8 /* PROPS */, ["label", "modelValue"]), (hasDuration.value) ? (_openBlock(), _createBlock(_component_ModalDurationPicker, {
              key: 0,
              modelValue: duration.value,
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((duration).value = $event)),
              "is-valid": isValidDuration.value,
              "onUpdate:is-valid": _cache[2] || (_cache[2] = ($event: any) => ((isValidDuration).value = $event))
            }, null, 8 /* PROPS */, ["modelValue", "is-valid"])) : _createCommentVNode("v-if", true), _createVNode(_component_CommonCheckbox, {
            label: _ctx.$t('confirm.mute_account.notifications'),
            "prepend-checkbox": "",
            "checked-icon-color": "text-primary",
            modelValue: shouldMuteNotifications.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((shouldMuteNotifications).value = $event))
          }, null, 8 /* PROPS */, ["label", "modelValue"]) ])) : _createCommentVNode("v-if", true), (isBlockDomain.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          "flex-col": "",
          flex: "",
          "gap-2": ""
        }, [ _createElementVNode("label", _hoisted_2, _toDisplayString(_ctx.$t('confirm.block_domain.type_to_confirm', [__props.domainToBlock])), 1 /* TEXT */), _withDirectives(_createElementVNode("input", {
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((domainInput).value = $event)),
            type: "text",
            placeholder: __props.domainToBlock,
            class: "px-3 py-2 border border-base rounded",
            autocomplete: "off"
          }, null, 8 /* PROPS */, ["placeholder"]), [ [_vModelText, domainInput.value] ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
        flex: "",
        "justify-end": "",
        "gap-2": ""
      }, [ _createElementVNode("button", {
          "btn-text": "",
          onClick: _cache[5] || (_cache[5] = ($event: any) => (handleChoice('cancel')))
        }, _toDisplayString(_ctx.cancel || _ctx.$t('confirm.common.cancel')), 1 /* TEXT */), _createElementVNode("button", {
          "btn-solid": "",
          disabled: !isValidDuration.value || (isBlockDomain.value && !isDomainConfirmed.value),
          onClick: _cache[6] || (_cache[6] = ($event: any) => (handleChoice('confirm')))
        }, _toDisplayString(_ctx.confirm || _ctx.$t('confirm.common.confirm')), 9 /* TEXT, PROPS */, ["disabled"]) ]) ]))
}
}

})
