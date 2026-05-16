import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
import { onMounted, useTemplateRef, ref, onUnmounted } from "vue";
import Cropper from "cropperjs";
import tinycolor from "tinycolor2";
import MkModalWindow from "@/components/MkModalWindow.vue";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkCropperDialog",
  props: {
    imageFile: {
      type: null,
      required: true
    },
    aspectRatio: {
      type: [Number, null],
      required: true
    },
    uploadFolder: {
      type: [String, null],
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
    const imgUrl = URL.createObjectURL(props.imageFile);
    const dialogEl = useTemplateRef("dialogEl");
    const imgEl = useTemplateRef("imgEl");
    let cropper = null;
    const loading = ref(true);
    async function ok() {
      const promise = new Promise(async (res) => {
        if (cropper == null) throw new Error("Cropper is not initialized");
        const croppedImage = await cropper.getCropperImage();
        const croppedSection = await cropper.getCropperSelection();
        // 拡大率を計算し、(ほぼ)元の大きさに戻す
        const zoomedRate = croppedImage.getBoundingClientRect().width / croppedImage.clientWidth;
        const widthToRender = croppedSection.getBoundingClientRect().width / zoomedRate;
        const croppedCanvas = await croppedSection.$toCanvas({ width: widthToRender });
        croppedCanvas.toBlob((blob) => {
          if (!blob) return;
          res(blob);
        });
      });
      const f = await promise;
      let finalFile;
      if (props.imageFile instanceof File) {
        finalFile = new File([f], props.imageFile.name, { type: f.type });
      } else {
        finalFile = f;
      }
      emit("ok", finalFile);
      if (dialogEl.value != null) dialogEl.value.close();
    }
    function cancel() {
      emit("cancel");
      if (dialogEl.value != null) dialogEl.value.close();
    }
    function onImageLoad() {
      loading.value = false;
      if (cropper) {
        cropper.getCropperImage().$center("contain");
        cropper.getCropperSelection().$center();
      }
    }
    onMounted(() => {
      if (imgEl.value == null) return;
      cropper = new Cropper(imgEl.value, {});
      const computedStyle = getComputedStyle(window.document.documentElement);
      const selection = cropper.getCropperSelection();
      selection.themeColor = tinycolor(computedStyle.getPropertyValue("--MI_THEME-accent")).toHexString();
      if (props.aspectRatio != null) selection.aspectRatio = props.aspectRatio;
      selection.initialAspectRatio = props.aspectRatio ?? 1;
      selection.outlined = true;
      window.setTimeout(() => {
        if (cropper == null) return;
        cropper.getCropperImage().$center("contain");
        selection.$center();
      }, 100);
      // モーダルオープンアニメーションが終わったあとで再度調整
      window.setTimeout(() => {
        if (cropper == null) return;
        cropper.getCropperImage().$center("contain");
        selection.$center();
      }, 500);
    });
    onUnmounted(() => {
      URL.revokeObjectURL(imgUrl);
    });
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialogEl",
        ref: dialogEl,
        width: 800,
        height: 500,
        scroll: false,
        withOkButton: true,
        onClose: _cache[0] || (_cache[0] = ($event) => cancel()),
        onOk: _cache[1] || (_cache[1] = ($event) => ok()),
        onClosed: _cache[2] || (_cache[2] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(i18n).ts.cropImage),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createElementVNode("div", {
          class: "mk-cropper-dialog",
          style: `--vw: 100%; --vh: 100%;`
        }, [_createVNode(
          _Transition,
          { name: "fade" },
          {
            default: _withCtx(() => [loading.value ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "loading"
            }, [_createVNode(_component_MkLoading)])) : _createCommentVNode("v-if", true)]),
            _: 2
          },
          1024
          /* DYNAMIC_SLOTS */
        ), _createElementVNode("div", { class: "container" }, [_createElementVNode("img", {
          ref_key: "imgEl",
          ref: imgEl,
          src: _unref(imgUrl),
          style: "display: none;",
          onLoad: onImageLoad
        }, null, 40, ["src"])])])]),
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
