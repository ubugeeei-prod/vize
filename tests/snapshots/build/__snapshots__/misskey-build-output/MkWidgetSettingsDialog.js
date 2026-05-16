import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-icons" });
import { useTemplateRef, ref, computed, onBeforeUnmount, onMounted } from "vue";
import MkPreviewWithControls from "./MkPreviewWithControls.vue";
import { deepClone } from "@/utility/clone.js";
import { i18n } from "@/i18n.js";
import MkModalWindow from "@/components/MkModalWindow.vue";
import MkForm from "@/components/MkForm.vue";
export default {
  __name: "MkWidgetSettingsDialog",
  props: {
    widgetName: {
      type: null,
      required: true
    },
    form: {
      type: null,
      required: true
    },
    currentSettings: {
      type: null,
      required: true
    }
  },
  emits: [
    "saved",
    "canceled",
    "closed"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const dialog = useTemplateRef("dialog");
    const settings = ref(deepClone(props.currentSettings));
    const canSave = ref(true);
    function onCanSaveStateChanged(newCanSave) {
      canSave.value = newCanSave;
    }
    function save() {
      if (!canSave.value) return;
      emit("saved", deepClone(settings.value));
      dialog.value?.close();
    }
    function cancel() {
      emit("canceled");
      dialog.value?.close();
    }
    //#region プレビューのリサイズ
    const resizerRootEl = useTemplateRef("resizerRootEl");
    const resizerEl = useTemplateRef("resizerEl");
    const widgetHeight = ref(0);
    const widgetScale = ref(1);
    const widgetStyle = computed(() => {
      return `translate(-50%, -50%) scale(${widgetScale.value})`;
    });
    const ro1 = new ResizeObserver(() => {
      widgetHeight.value = resizerEl.value.clientHeight;
      calcScale();
    });
    const ro2 = new ResizeObserver(() => {
      calcScale();
    });
    function calcScale() {
      if (!resizerRootEl.value) return;
      const previewWidth = resizerRootEl.value.clientWidth - 40;
      const previewHeight = resizerRootEl.value.clientHeight - 40;
      const widgetWidth = 280;
      const scale = Math.min(previewWidth / widgetWidth, previewHeight / widgetHeight.value, 1);
      widgetScale.value = scale;
    }
    onMounted(() => {
      if (resizerEl.value) {
        ro1.observe(resizerEl.value);
      }
      if (resizerRootEl.value) {
        ro2.observe(resizerRootEl.value);
      }
      calcScale();
    });
    onBeforeUnmount(() => {
      ro1.disconnect();
      ro2.disconnect();
    });
    //#endregion
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialog",
        ref: dialog,
        width: 1e3,
        height: 600,
        scroll: false,
        withOkButton: true,
        okButtonDisabled: !canSave.value,
        onClose: _cache[0] || (_cache[0] = ($event) => cancel()),
        onOk: _cache[1] || (_cache[1] = ($event) => save()),
        onClosed: _cache[2] || (_cache[2] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [
          _hoisted_1,
          _createTextVNode(" "),
          _createTextVNode(
            _toDisplayString(_unref(i18n).ts._widgets[__props.widgetName] ?? __props.widgetName),
            1
            /* TEXT */
          )
        ]),
        default: _withCtx(() => [_createVNode(MkPreviewWithControls, null, {
          preview: _withCtx(() => [_createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.previewWrapper) },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(["_acrylic", _ctx.$style.previewTitle]) },
              _toDisplayString(_unref(i18n).ts.preview),
              3
              /* TEXT, CLASS */
            ), _createElementVNode(
              "div",
              {
                ref_key: "resizerRootEl",
                ref: resizerRootEl,
                class: _normalizeClass(_ctx.$style.previewResizerRoot),
                inert: ""
              },
              [_createElementVNode(
                "div",
                {
                  ref_key: "resizerEl",
                  ref: resizerEl,
                  class: _normalizeClass(_ctx.$style.previewResizer),
                  style: _normalizeStyle({ transform: widgetStyle.value })
                },
                [_createVNode(_resolveDynamicComponent(`widget-${__props.widgetName}`), { widget: {
                  name: __props.widgetName,
                  id: "__PREVIEW__",
                  data: settings.value
                } }, null, 8, ["widget"])],
                6
                /* CLASS, STYLE */
              )],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          )]),
          controls: _withCtx(() => [_createElementVNode("div", { class: "_spacer" }, [_createVNode(MkForm, {
            form: __props.form,
            onCanSaveStateChange: onCanSaveStateChanged,
            modelValue: settings.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => settings.value = $event)
          }, null, 8, ["form", "modelValue"])])]),
          _: 1
        })]),
        _: 1
      }, 8, [
        "width",
        "height",
        "scroll",
        "withOkButton",
        "okButtonDisabled"
      ]);
    };
  }
};
