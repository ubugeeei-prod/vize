import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, createSlots as _createSlots, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-lock" });
import { ref, useTemplateRef, computed } from "vue";
import MkModal from "@/components/MkModal.vue";
import MkButton from "@/components/MkButton.vue";
import MkInput from "@/components/MkInput.vue";
import MkSelect from "@/components/MkSelect.vue";
import { useMkSelect } from "@/composables/use-mkselect.js";
import { i18n } from "@/i18n.js";
export {};
export default {
  __name: "MkDialog",
  props: {
    type: {
      type: String,
      required: false,
      default: "info"
    },
    title: {
      type: String,
      required: false
    },
    text: {
      type: String,
      required: false
    },
    input: {
      type: Object,
      required: false
    },
    select: {
      type: Object,
      required: false
    },
    icon: {
      type: String,
      required: false
    },
    actions: {
      type: Array,
      required: false
    },
    showOkButton: {
      type: Boolean,
      required: false,
      default: true
    },
    showCancelButton: {
      type: Boolean,
      required: false,
      default: false
    },
    cancelableByBgClick: {
      type: Boolean,
      required: false,
      default: true
    },
    okText: {
      type: String,
      required: false
    },
    cancelText: {
      type: String,
      required: false
    }
  },
  emits: ["done", "closed"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const modal = useTemplateRef("modal");
    const inputValue = ref(props.input?.default ?? null);
    const okButtonDisabledReason = computed(() => {
      if (props.input) {
        if (props.input.minLength) {
          if (inputValue.value == null || inputValue.value.length < props.input.minLength) {
            return "charactersBelow";
          }
        }
        if (props.input.maxLength) {
          if (inputValue.value && inputValue.value.length > props.input.maxLength) {
            return "charactersExceeded";
          }
        }
      }
      return null;
    });
    const { def: selectDef, model: selectedValue } = useMkSelect({
      items: computed(() => props.select?.items ?? []),
      initialValue: props.select?.default ?? null
    });
    // eslint-disable-line no-redeclare
    function done(canceled, result) {
      emit("done", {
        canceled,
        result
      });
      modal.value?.close();
    }
    async function ok() {
      if (!props.showOkButton) return;
      const result = props.input ? inputValue.value : props.select ? selectedValue.value : true;
      done(false, result);
    }
    function cancel() {
      done(true);
    }
    /*
    function onBgClick() {
    if (props.cancelableByBgClick) cancel();
    }
    */
    function onInputKeydown(evt) {
      if (evt.key === "Enter" && okButtonDisabledReason.value === null) {
        evt.preventDefault();
        evt.stopPropagation();
        ok();
      }
    }
    return (_ctx, _cache) => {
      const _component_MkSystemIcon = _resolveComponent("MkSystemIcon");
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _component_Mfm = _resolveComponent("Mfm");
      return _openBlock(), _createBlock(MkModal, {
        ref_key: "modal",
        ref: modal,
        preferType: "dialog",
        zPriority: "high",
        onClick: _cache[0] || (_cache[0] = ($event) => done(true)),
        onClosed: _cache[1] || (_cache[1] = ($event) => emit("closed")),
        onEsc: _cache[2] || (_cache[2] = ($event) => cancel())
      }, {
        default: _withCtx(() => [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.root) },
          [
            __props.icon ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.icon)
              },
              [_createElementVNode(
                "i",
                { class: _normalizeClass(__props.icon) },
                null,
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            )) : !__props.input && !__props.select ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 1,
                class: _normalizeClass([_ctx.$style.icon])
              },
              [__props.type === "success" ? (_openBlock(), _createBlock(
                _component_MkSystemIcon,
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.iconInner),
                  style: "width: 45px;",
                  type: "success"
                },
                null,
                2
                /* CLASS */
              )) : __props.type === "error" ? (_openBlock(), _createBlock(
                _component_MkSystemIcon,
                {
                  key: 1,
                  class: _normalizeClass(_ctx.$style.iconInner),
                  style: "width: 45px;",
                  type: "error"
                },
                null,
                2
                /* CLASS */
              )) : __props.type === "warning" ? (_openBlock(), _createBlock(
                _component_MkSystemIcon,
                {
                  key: 2,
                  class: _normalizeClass(_ctx.$style.iconInner),
                  style: "width: 45px;",
                  type: "warn"
                },
                null,
                2
                /* CLASS */
              )) : __props.type === "info" ? (_openBlock(), _createBlock(
                _component_MkSystemIcon,
                {
                  key: 3,
                  class: _normalizeClass(_ctx.$style.iconInner),
                  style: "width: 45px;",
                  type: "info"
                },
                null,
                2
                /* CLASS */
              )) : __props.type === "question" ? (_openBlock(), _createBlock(
                _component_MkSystemIcon,
                {
                  key: 4,
                  class: _normalizeClass(_ctx.$style.iconInner),
                  style: "width: 45px;",
                  type: "question"
                },
                null,
                2
                /* CLASS */
              )) : __props.type === "waiting" ? (_openBlock(), _createBlock(_component_MkLoading, {
                key: 5,
                class: _normalizeClass(_ctx.$style.iconInner),
                em: true
              }, null, 10, ["em"])) : _createCommentVNode("v-if", true)],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true),
            __props.title ? (_openBlock(), _createElementBlock(
              "header",
              {
                key: 0,
                class: _normalizeClass(["_selectable", _ctx.$style.title])
              },
              [_createVNode(_component_Mfm, { text: __props.title }, null, 8, ["text"])],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true),
            __props.text ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(["_selectable", _ctx.$style.text])
              },
              [_createVNode(_component_Mfm, { text: __props.text }, null, 8, ["text"])],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true),
            __props.input ? (_openBlock(), _createBlock(MkInput, {
              key: 0,
              autofocus: "",
              type: __props.input.type || "text",
              placeholder: __props.input.placeholder || undefined,
              autocomplete: __props.input.autocomplete,
              onKeydown: onInputKeydown,
              modelValue: inputValue.value,
              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => inputValue.value = $event)
            }, _createSlots({ _: 2 }, [__props.input.type === "password" ? {
              name: "prefix",
              fn: _withCtx(() => [_hoisted_1]),
              key: "0"
            } : undefined, {
              name: "caption",
              fn: _withCtx(() => [okButtonDisabledReason.value === "charactersExceeded" ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                textContent: _toDisplayString(_unref(i18n).tsx._dialog.charactersExceeded({
                  current: inputValue.value?.length ?? 0,
                  max: __props.input.maxLength ?? "NaN"
                }))
              }, null, 8, ["textContent"])) : okButtonDisabledReason.value === "charactersBelow" ? (_openBlock(), _createElementBlock("span", {
                key: 1,
                textContent: _toDisplayString(_unref(i18n).tsx._dialog.charactersBelow({
                  current: inputValue.value?.length ?? 0,
                  min: __props.input.minLength ?? "NaN"
                }))
              }, null, 8, ["textContent"])) : _createCommentVNode("v-if", true)])
            }]), 1032, [
              "type",
              "placeholder",
              "autocomplete",
              "modelValue"
            ])) : _createCommentVNode("v-if", true),
            __props.select ? (_openBlock(), _createBlock(MkSelect, {
              key: 0,
              items: _unref(selectDef),
              autofocus: "",
              modelValue: _unref(selectedValue),
              "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => selectedValue.value = $event)
            }, null, 8, ["items", "modelValue"])) : _createCommentVNode("v-if", true),
            (__props.showOkButton || __props.showCancelButton) && !__props.actions ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.buttons)
              },
              [__props.showOkButton ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                "data-cy-modal-dialog-ok": "",
                inline: "",
                primary: "",
                rounded: "",
                autofocus: !__props.input && !__props.select,
                disabled: okButtonDisabledReason.value != null,
                onClick: ok
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(__props.okText ?? (__props.showCancelButton || __props.input || __props.select ? _unref(i18n).ts.ok : _unref(i18n).ts.gotIt)),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["autofocus", "disabled"])) : _createCommentVNode("v-if", true), __props.showCancelButton || __props.input || __props.select ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                "data-cy-modal-dialog-cancel": "",
                inline: "",
                rounded: "",
                onClick: cancel
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(__props.cancelText ?? _unref(i18n).ts.cancel),
                  1
                  /* TEXT */
                )]),
                _: 1
              })) : _createCommentVNode("v-if", true)],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true),
            __props.actions ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.buttons)
              },
              [(_openBlock(true), _createElementBlock(
                _Fragment,
                null,
                _renderList(__props.actions, (action) => {
                  return _openBlock(), _createBlock(MkButton, {
                    key: action.text,
                    inline: "",
                    rounded: "",
                    primary: action.primary,
                    danger: action.danger,
                    onClick: () => {
                      action.callback();
                      _unref(modal)?.close();
                    }
                  }, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(action.text),
                      1
                      /* TEXT */
                    )]),
                    _: 2
                  }, 1032, [
                    "primary",
                    "danger",
                    "onClick"
                  ]);
                }),
                128
                /* KEYED_FRAGMENT */
              ))],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)
          ],
          2
          /* CLASS */
        )]),
        _: 1
      }, 8, ["preferType", "zPriority"]);
    };
  }
};
