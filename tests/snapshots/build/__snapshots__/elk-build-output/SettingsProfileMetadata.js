import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, vModelText as _vModelText } from "vue"


const _hoisted_1 = { "font-medium": "true" }
const _hoisted_2 = { "text-sm": "true", "text-secondary": "true" }
import type { mastodon } from 'masto'

export default /*@__PURE__*/_defineComponent({
  __name: 'SettingsProfileMetadata',
  props: {
    "modelValue": { required: true },
    "modelModifiers": {},
  },
  emits: ["update:modelValue"],
  setup(__props) {

const form = _useModel(__props, "modelValue")
const dropdown = ref<any>()
const fieldIcons = computed(() =>
  Array.from({ length: maxAccountFieldCount.value }, (_, i) =>
    getAccountFieldIcon(form.value.fieldsAttributes[i].name)),
)
const fieldCount = computed(() => {
  // find last non-empty field
  const idx = [...form.value.fieldsAttributes].reverse().findIndex(f => f.name || f.value)
  if (idx === -1)
    return 1
  return Math.min(
    form.value.fieldsAttributes.length - idx + 1,
    maxAccountFieldCount.value,
  )
})
function chooseIcon(i: number, text: string) {
  form.value.fieldsAttributes[i].name = text
  dropdown.value[i]?.hide()
}

return (_ctx: any,_cache: any) => {
  const _component_CommonTooltip = _resolveComponent("CommonTooltip")
  const _component_CommonDropdown = _resolveComponent("CommonDropdown")

  return (_openBlock(), _createElementBlock("div", { "space-y-2": "" }, [ _createElementVNode("div", _hoisted_1, _toDisplayString(_ctx.$t('settings.profile.appearance.profile_metadata')), 1 /* TEXT */), _createElementVNode("div", _hoisted_2, _toDisplayString(_ctx.$t('settings.profile.appearance.profile_metadata_desc', [_ctx.maxAccountFieldCount])), 1 /* TEXT */), _createElementVNode("div", { flex: "~ col gap4" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(fieldCount.value, (i) => {
          return (_openBlock(), _createElementBlock("div", {
            key: i,
            flex: "~ gap3",
            "items-center": ""
          }, [
            _createVNode(_component_CommonDropdown, {
              ref_key: "dropdown", ref: dropdown,
              placement: "left"
            }, {
              popper: _withCtx(() => [
                _createElementVNode("div", {
                  flex: "~ wrap gap-1",
                  "max-w-60": "",
                  m2: "",
                  me1: ""
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.accountFieldIcons, (icon, text) => {
                    return (_openBlock(), _createBlock(_component_CommonTooltip, {
                      key: icon,
                      content: text
                    }, {
                      default: _withCtx(() => [
                        (text !== 'Joined')
                          ? (_openBlock(), _createElementBlock("button", {
                            key: 0,
                            type: "button",
                            "btn-action-icon": "",
                            onClick: _cache[0] || (_cache[0] = ($event: any) => (chooseIcon(i - 1, text)))
                          }, [
                            _createElementVNode("div", {
                              "text-xl": "",
                              class: _normalizeClass(icon)
                            }, null, 2 /* CLASS */)
                          ]))
                          : _createCommentVNode("v-if", true)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["content"]))
                  }), 128 /* KEYED_FRAGMENT */))
                ])
              ]),
              default: _withCtx(() => [
                _createVNode(_component_CommonTooltip, { content: _ctx.$t('tooltip.pick_an_icon') }, {
                  default: _withCtx(() => [
                    _createElementVNode("button", {
                      type: "button",
                      "btn-action-icon": ""
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(fieldIcons.value[i - 1] || 'i-ri:question-mark')
                      }, null, 2 /* CLASS */)
                    ])
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["content"])
              ]),
              _: 2 /* DYNAMIC */
            }, 512 /* NEED_PATCH */),
            _withDirectives(_createElementVNode("input", {
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((form.value.fieldsAttributes[i - 1].name) = $event)),
              type: "text",
              "placeholder-text-secondary": "",
              placeholder: _ctx.$t('settings.profile.appearance.profile_metadata_label'),
              "input-base": ""
            }, null, 8 /* PROPS */, ["placeholder"]), [
              [_vModelText, form.value.fieldsAttributes[i - 1].name]
            ]),
            _withDirectives(_createElementVNode("input", {
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((form.value.fieldsAttributes[i - 1].value) = $event)),
              type: "text",
              "placeholder-text-secondary": "",
              placeholder: _ctx.$t('settings.profile.appearance.profile_metadata_value'),
              "input-base": ""
            }, null, 8 /* PROPS */, ["placeholder"]), [
              [_vModelText, form.value.fieldsAttributes[i - 1].value]
            ])
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]) ]))
}
}

})
