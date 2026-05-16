import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-code" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-arrow-right" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-copy" });
import { useTemplateRef, ref, computed, nextTick, onMounted, onDeactivated, onUnmounted } from "vue";
import { url } from "@@/js/config.js";
import { embedRouteWithScrollbar } from "@@/js/embed-page.js";
import MkModalWindow from "@/components/MkModalWindow.vue";
import MkPreviewWithControls from "@/components/MkPreviewWithControls.vue";
import MkInput from "@/components/MkInput.vue";
import MkSelect from "@/components/MkSelect.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkButton from "@/components/MkButton.vue";
import MkCode from "@/components/MkCode.vue";
import MkInfo from "@/components/MkInfo.vue";
import { i18n } from "@/i18n.js";
import { useMkSelect } from "@/composables/use-mkselect.js";
import { copyToClipboard } from "@/utility/copy-to-clipboard.js";
import { normalizeEmbedParams, getEmbedCode } from "@/utility/get-embed-code.js";
export default {
  __name: "MkEmbedCodeGenDialog",
  props: {
    entity: {
      type: null,
      required: true
    },
    id: {
      type: String,
      required: true
    },
    params: {
      type: null,
      required: false
    }
  },
  emits: [
    "ok",
    "cancel",
    "closed"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    //#region Modalの制御
    const dialogEl = useTemplateRef("dialogEl");
    function cancel() {
      emit("cancel");
      dialogEl.value?.close();
    }
    function close() {
      dialogEl.value?.close();
    }
    const phase = ref("input");
    //#endregion
    //#region 埋め込みURL生成・カスタマイズ
    // 本URL生成用params
    const paramsForUrl = computed(() => ({
      header: header.value,
      maxHeight: typeof maxHeight.value === "number" ? Math.max(0, maxHeight.value) : undefined,
      colorMode: colorMode.value === "auto" ? undefined : colorMode.value,
      rounded: rounded.value,
      border: border.value
    }));
    // プレビュー用params（手動で更新を掛けるのでref）
    const paramsForPreview = ref(props.params ?? {});
    const embedPreviewUrl = computed(() => {
      const paramClass = new URLSearchParams(normalizeEmbedParams(paramsForPreview.value));
      if (paramClass.has("maxHeight")) {
        const maxHeight = parseInt(paramClass.get("maxHeight"));
        paramClass.set("maxHeight", maxHeight === 0 ? "500" : Math.min(maxHeight, 700).toString());
      }
      return `${url}/embed/${props.entity}/${props.id}${paramClass.toString() ? "?" + paramClass.toString() : ""}`;
    });
    const isEmbedWithScrollbar = computed(() => embedRouteWithScrollbar.includes(props.entity));
    const header = ref(props.params?.header ?? true);
    const maxHeight = ref(props.params?.maxHeight !== 0 ? props.params?.maxHeight ?? null : 500);
    const { model: colorMode, def: colorModeDef } = useMkSelect({
      items: [
        {
          value: "auto",
          label: i18n.ts.syncDeviceDarkMode
        },
        {
          value: "light",
          label: i18n.ts.light
        },
        {
          value: "dark",
          label: i18n.ts.dark
        }
      ],
      initialValue: props.params?.colorMode ?? "auto"
    });
    const rounded = ref(props.params?.rounded ?? true);
    const border = ref(props.params?.border ?? true);
    function applyToPreview() {
      const currentPreviewUrl = embedPreviewUrl.value;
      paramsForPreview.value = {
        header: header.value,
        maxHeight: typeof maxHeight.value === "number" ? Math.max(0, maxHeight.value) : undefined,
        colorMode: colorMode.value === "auto" ? undefined : colorMode.value,
        rounded: rounded.value,
        border: border.value
      };
      nextTick(() => {
        if (currentPreviewUrl === embedPreviewUrl.value) {
          // URLが変わらなくてもリロード
          iframeEl.value?.contentWindow?.window.location.reload();
        }
      });
    }
    const result = ref("");
    function generate() {
      result.value = getEmbedCode(`/embed/${props.entity}/${props.id}`, paramsForUrl.value);
      phase.value = "result";
    }
    function doCopy() {
      copyToClipboard(result.value);
    }
    //#endregion
    //#region プレビューのリサイズ
    const resizerRootEl = useTemplateRef("resizerRootEl");
    const iframeLoading = ref(true);
    const iframeEl = useTemplateRef("iframeEl");
    const iframeHeight = ref(0);
    const iframeScale = ref(1);
    const iframeStyle = computed(() => {
      return `translate(-50%, -50%) scale(${iframeScale.value})`;
    });
    const resizeObserver = new ResizeObserver(() => {
      calcScale();
    });
    function iframeOnLoad() {
      iframeEl.value?.contentWindow?.addEventListener("beforeunload", () => {
        iframeLoading.value = true;
        nextTick(() => {
          iframeHeight.value = 0;
          iframeScale.value = 1;
        });
      });
    }
    function windowEventHandler(event) {
      if (event.source !== iframeEl.value?.contentWindow) {
        return;
      }
      if (event.data.type === "misskey:embed:ready") {
        iframeEl.value.contentWindow?.postMessage({
          type: "misskey:embedParent:registerIframeId",
          payload: { iframeId: "embedCodeGen" }
        });
      }
      if (event.data.type === "misskey:embed:changeHeight") {
        iframeHeight.value = event.data.payload.height;
        nextTick(() => {
          calcScale();
          iframeLoading.value = false;
        });
      }
    }
    function calcScale() {
      if (!resizerRootEl.value) return;
      const previewWidth = resizerRootEl.value.clientWidth - 40;
      const previewHeight = resizerRootEl.value.clientHeight - 40;
      const iframeWidth = 500;
      const scale = Math.min(previewWidth / iframeWidth, previewHeight / iframeHeight.value, 1);
      iframeScale.value = scale;
    }
    onMounted(() => {
      window.addEventListener("message", windowEventHandler);
      if (!resizerRootEl.value) return;
      resizeObserver.observe(resizerRootEl.value);
    });
    function reset() {
      window.removeEventListener("message", windowEventHandler);
      resizeObserver.disconnect();
      // プレビューのリセット
      iframeHeight.value = 0;
      iframeScale.value = 1;
      iframeLoading.value = true;
      result.value = "";
      phase.value = "input";
    }
    onDeactivated(() => {
      reset();
    });
    onUnmounted(() => {
      reset();
    });
    //#endregion
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialogEl",
        ref: dialogEl,
        width: 1e3,
        height: 600,
        scroll: false,
        withOkButton: false,
        onClose: _cache[0] || (_cache[0] = ($event) => cancel()),
        onClosed: _cache[1] || (_cache[1] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [
          _hoisted_1,
          _createTextVNode(" "),
          _createTextVNode(
            _toDisplayString(_unref(i18n).ts._embedCodeGen.title),
            1
            /* TEXT */
          )
        ]),
        default: _withCtx(() => [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.embedCodeGenRoot) },
          [_createVNode(_Transition, {
            mode: "out-in",
            enterActiveClass: _ctx.$style.transition_x_enterActive,
            leaveActiveClass: _ctx.$style.transition_x_leaveActive,
            enterFromClass: _ctx.$style.transition_x_enterFrom,
            leaveToClass: _ctx.$style.transition_x_leaveTo
          }, {
            default: _withCtx(() => [phase.value === "input" ? (_openBlock(), _createBlock(MkPreviewWithControls, {
              key: "input",
              previewLoading: iframeLoading.value
            }, {
              preview: _withCtx(() => [_createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.embedCodeGenPreviewWrapper) },
                [_createElementVNode(
                  "div",
                  { class: _normalizeClass(["_acrylic", _ctx.$style.embedCodeGenPreviewTitle]) },
                  _toDisplayString(_unref(i18n).ts.preview),
                  3
                  /* TEXT, CLASS */
                ), _createElementVNode(
                  "div",
                  {
                    ref_key: "resizerRootEl",
                    ref: resizerRootEl,
                    class: _normalizeClass(_ctx.$style.embedCodeGenPreviewResizerRoot),
                    inert: ""
                  },
                  [_createElementVNode(
                    "div",
                    {
                      class: _normalizeClass(_ctx.$style.embedCodeGenPreviewResizer),
                      style: _normalizeStyle({ transform: iframeStyle.value })
                    },
                    [_createElementVNode("iframe", {
                      ref_key: "iframeEl",
                      ref: iframeEl,
                      src: embedPreviewUrl.value,
                      class: _normalizeClass(_ctx.$style.embedCodeGenPreviewIframe),
                      style: _normalizeStyle({ height: `${iframeHeight.value}px` }),
                      onLoad: iframeOnLoad
                    }, null, 46, ["src"])],
                    6
                    /* CLASS, STYLE */
                  )],
                  2
                  /* CLASS */
                )],
                2
                /* CLASS */
              )]),
              controls: _withCtx(() => [_createElementVNode("div", { class: "_spacer _gaps" }, [
                isEmbedWithScrollbar.value ? (_openBlock(), _createBlock(MkInput, {
                  key: 0,
                  type: "number",
                  min: 0,
                  modelValue: maxHeight.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => maxHeight.value = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._embedCodeGen.maxHeight),
                    1
                    /* TEXT */
                  )]),
                  suffix: _withCtx(() => [_createTextVNode("px")]),
                  caption: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._embedCodeGen.maxHeightDescription),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["min", "modelValue"])) : _createCommentVNode("v-if", true),
                _createVNode(MkSelect, {
                  items: _unref(colorModeDef),
                  modelValue: _unref(colorMode),
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => colorMode.value = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.theme),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["items", "modelValue"]),
                isEmbedWithScrollbar.value ? (_openBlock(), _createBlock(MkSwitch, {
                  key: 0,
                  modelValue: header.value,
                  "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => header.value = $event)
                }, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._embedCodeGen.header),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])) : _createCommentVNode("v-if", true),
                _createVNode(MkSwitch, {
                  modelValue: rounded.value,
                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => rounded.value = $event)
                }, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._embedCodeGen.rounded),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"]),
                _createVNode(MkSwitch, {
                  modelValue: border.value,
                  "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event) => border.value = $event)
                }, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._embedCodeGen.border),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"]),
                isEmbedWithScrollbar.value && (!maxHeight.value || maxHeight.value <= 0) ? (_openBlock(), _createBlock(MkInfo, {
                  key: 0,
                  warn: ""
                }, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._embedCodeGen.maxHeightWarn),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })) : _createCommentVNode("v-if", true),
                typeof maxHeight.value === "number" && (maxHeight.value <= 0 || maxHeight.value > 700) ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._embedCodeGen.previewIsNotActual),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })) : _createCommentVNode("v-if", true),
                _createElementVNode("div", { class: "_buttons" }, [_createVNode(MkButton, {
                  disabled: iframeLoading.value,
                  onClick: applyToPreview
                }, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._embedCodeGen.applyToPreview),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["disabled"]), _createVNode(MkButton, {
                  disabled: iframeLoading.value,
                  primary: "",
                  onClick: generate
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(
                      _toDisplayString(_unref(i18n).ts._embedCodeGen.generateCode),
                      1
                      /* TEXT */
                    ),
                    _createTextVNode(" "),
                    _hoisted_2
                  ]),
                  _: 1
                }, 8, ["disabled"])])
              ])]),
              _: 1
            }, 8, ["previewLoading"])) : phase.value === "result" ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: "result",
                class: _normalizeClass(_ctx.$style.embedCodeGenResultRoot)
              },
              [_createElementVNode(
                "div",
                { class: _normalizeClass(["_gaps", _ctx.$style.embedCodeGenResultWrapper]) },
                [
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    _createElementVNode(
                      "div",
                      { class: _normalizeClass(_ctx.$style.embedCodeGenResultHeadingIcon) },
                      [_hoisted_3],
                      2
                      /* CLASS */
                    ),
                    _createElementVNode(
                      "div",
                      { class: _normalizeClass(_ctx.$style.embedCodeGenResultHeading) },
                      _toDisplayString(_unref(i18n).ts._embedCodeGen.codeGenerated),
                      3
                      /* TEXT, CLASS */
                    ),
                    _createElementVNode(
                      "div",
                      { class: _normalizeClass(_ctx.$style.embedCodeGenResultDescription) },
                      _toDisplayString(_unref(i18n).ts._embedCodeGen.codeGeneratedDescription),
                      3
                      /* TEXT, CLASS */
                    )
                  ]),
                  _createElementVNode("div", { class: "_gaps_s" }, [_createVNode(MkCode, {
                    code: result.value,
                    lang: "html",
                    forceShow: true,
                    copyButton: false,
                    class: _normalizeClass(_ctx.$style.embedCodeGenResultCode)
                  }, null, 10, [
                    "code",
                    "forceShow",
                    "copyButton"
                  ]), _createVNode(
                    MkButton,
                    {
                      class: _normalizeClass(_ctx.$style.embedCodeGenResultButtons),
                      rounded: "",
                      primary: "",
                      onClick: doCopy
                    },
                    {
                      default: _withCtx(() => [
                        _hoisted_4,
                        _createTextVNode(" "),
                        _createTextVNode(
                          _toDisplayString(_unref(i18n).ts.copy),
                          1
                          /* TEXT */
                        )
                      ]),
                      _: 1
                    },
                    2
                    /* CLASS */
                  )]),
                  _createVNode(
                    MkButton,
                    {
                      class: _normalizeClass(_ctx.$style.embedCodeGenResultButtons),
                      rounded: "",
                      transparent: "",
                      onClick: close
                    },
                    {
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts.close),
                        1
                        /* TEXT */
                      )]),
                      _: 1
                    },
                    2
                    /* CLASS */
                  )
                ],
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)]),
            _: 2
          }, 1032, [
            "enterActiveClass",
            "leaveActiveClass",
            "enterFromClass",
            "leaveToClass"
          ])],
          2
          /* CLASS */
        )]),
        _: 1
      }, 8, [
        "width",
        "height",
        "scroll",
        "withOkButton"
      ]);
    };
  }
};
