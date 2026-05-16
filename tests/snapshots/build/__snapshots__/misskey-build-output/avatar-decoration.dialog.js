import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-x" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { useTemplateRef, ref, computed } from "vue";
import MkButton from "@/components/MkButton.vue";
import MkModalWindow from "@/components/MkModalWindow.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import { i18n } from "@/i18n.js";
import MkRange from "@/components/MkRange.vue";
import { ensureSignin } from "@/i.js";
export default {
  __name: "avatar-decoration.dialog",
  props: {
    usingIndex: {
      type: [Number, null],
      required: true
    },
    decoration: {
      type: Object,
      required: true
    }
  },
  emits: [
    "closed",
    "attach",
    "update",
    "detach"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const $i = ensureSignin();
    const dialog = useTemplateRef("dialog");
    const exceeded = computed(() => $i.policies.avatarDecorationLimit - $i.avatarDecorations.length <= 0);
    const locked = computed(() => props.decoration.roleIdsThatCanBeUsedThisDecoration.length > 0 && !$i.roles.some((r) => props.decoration.roleIdsThatCanBeUsedThisDecoration.includes(r.id)));
    const angle = ref((props.usingIndex != null ? $i.avatarDecorations[props.usingIndex].angle : null) ?? 0);
    const flipH = ref((props.usingIndex != null ? $i.avatarDecorations[props.usingIndex].flipH : null) ?? false);
    const offsetX = ref((props.usingIndex != null ? $i.avatarDecorations[props.usingIndex].offsetX : null) ?? 0);
    const offsetY = ref((props.usingIndex != null ? $i.avatarDecorations[props.usingIndex].offsetY : null) ?? 0);
    const decorationsForPreview = computed(() => {
      const decoration = {
        id: props.decoration.id,
        url: props.decoration.url,
        angle: angle.value,
        flipH: flipH.value,
        offsetX: offsetX.value,
        offsetY: offsetY.value,
        blink: true
      };
      const decorations = [...$i.avatarDecorations];
      if (props.usingIndex != null) {
        decorations[props.usingIndex] = decoration;
      } else {
        decorations.push(decoration);
      }
      return decorations;
    });
    function cancel() {
      dialog.value?.close();
    }
    async function update() {
      emit("update", {
        angle: angle.value,
        flipH: flipH.value,
        offsetX: offsetX.value,
        offsetY: offsetY.value
      });
      dialog.value?.close();
    }
    async function attach() {
      emit("attach", {
        angle: angle.value,
        flipH: flipH.value,
        offsetX: offsetX.value,
        offsetY: offsetY.value
      });
      dialog.value?.close();
    }
    async function detach() {
      emit("detach");
      dialog.value?.close();
    }
    return (_ctx, _cache) => {
      const _component_MkAvatar = _resolveComponent("MkAvatar");
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialog",
        ref: dialog,
        width: 400,
        height: 450,
        onClose: cancel,
        onClosed: _cache[0] || (_cache[0] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(i18n).ts.avatarDecorations),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createElementVNode("div", null, [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
        }, [_createElementVNode("div", { style: "text-align: center;" }, [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.name) },
          _toDisplayString(__props.decoration.name),
          3
          /* TEXT, CLASS */
        ), _createVNode(_component_MkAvatar, {
          style: "width: 64px; height: 64px; margin-bottom: 20px;",
          user: _unref($i),
          decorations: decorationsForPreview.value,
          forceShowDecoration: ""
        }, null, 8, ["user", "decorations"])]), _createElementVNode("div", { class: "_gaps_s" }, [
          _createVNode(MkRange, {
            continuousUpdate: "",
            min: -.5,
            max: .5,
            step: .025,
            textConverter: (v) => `${Math.floor(v * 360)}°`,
            modelValue: angle.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => angle.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.angle),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, [
            "min",
            "max",
            "step",
            "textConverter",
            "modelValue"
          ]),
          _createVNode(MkRange, {
            continuousUpdate: "",
            min: -.25,
            max: .25,
            step: .025,
            textConverter: (v) => `${Math.floor(v * 100)}%`,
            modelValue: offsetX.value,
            "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => offsetX.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              "X " + _toDisplayString(_unref(i18n).ts.position),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, [
            "min",
            "max",
            "step",
            "textConverter",
            "modelValue"
          ]),
          _createVNode(MkRange, {
            continuousUpdate: "",
            min: -.25,
            max: .25,
            step: .025,
            textConverter: (v) => `${Math.floor(v * 100)}%`,
            modelValue: offsetY.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => offsetY.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              "Y " + _toDisplayString(_unref(i18n).ts.position),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, [
            "min",
            "max",
            "step",
            "textConverter",
            "modelValue"
          ]),
          _createVNode(MkSwitch, {
            modelValue: flipH.value,
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => flipH.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.flip),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["modelValue"])
        ])]), _createElementVNode(
          "div",
          { class: _normalizeClass(["_buttonsCenter", _ctx.$style.footer]) },
          [__props.usingIndex != null ? (_openBlock(), _createBlock(MkButton, {
            key: 0,
            primary: "",
            rounded: "",
            onClick: update
          }, {
            default: _withCtx(() => [
              _hoisted_1,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.update),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })) : _createCommentVNode("v-if", true), __props.usingIndex != null ? (_openBlock(), _createBlock(MkButton, {
            key: 0,
            rounded: "",
            onClick: detach
          }, {
            default: _withCtx(() => [
              _hoisted_2,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.detach),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })) : (_openBlock(), _createBlock(MkButton, {
            key: 1,
            disabled: exceeded.value || locked.value,
            primary: "",
            rounded: "",
            onClick: attach
          }, {
            default: _withCtx(() => [
              _hoisted_3,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.attach),
                1
                /* TEXT */
              )
            ]),
            _: 1
          }, 8, ["disabled"]))],
          2
          /* CLASS */
        )])]),
        _: 1
      }, 8, ["width", "height"]);
    };
  }
};
