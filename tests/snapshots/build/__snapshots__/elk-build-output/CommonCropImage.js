import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"

import type { Boundaries } from 'vue-advanced-cropper'
import { Cropper } from 'vue-advanced-cropper'
import 'vue-advanced-cropper/dist/style.css'

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonCropImage',
  props: {
    stencilAspectRatio: { type: Number, required: false, default: 1 / 1 },
    stencilSizePercentage: { type: Number, required: false, default: 0.9 },
    "modelValue": {}
  },
  emits: ["update:modelValue"],
  setup(__props: any) {

const file = _useModel(__props, "modelValue")
const cropperDialog = ref(false)
const cropper = ref<InstanceType<typeof Cropper>>()
const cropperFlag = ref(false)
const cropperImage = reactive({
  src: '',
  type: 'image/jpg',
})
function stencilSize({ boundaries }: { boundaries: Boundaries }) {
  return {
    width: boundaries.width * __props.stencilSizePercentage,
    height: boundaries.height * __props.stencilSizePercentage,
  }
}
watch(file, (file, _, onCleanup) => {
  let expired = false
  onCleanup(() => expired = true)
  if (file && !cropperFlag.value) {
    cropperDialog.value = true
    const reader = new FileReader()
    reader.readAsDataURL(file)
    reader.onload = (e) => {
      if (expired)
        return
      cropperImage.src = e.target?.result as string
      cropperImage.type = file.type
    }
  }
  cropperFlag.value = false
})
function cropImage() {
  if (cropper.value && file.value) {
    cropperFlag.value = true
    cropperDialog.value = false
    const { canvas } = cropper.value.getResult()
    canvas?.toBlob((blob) => {
      file.value = new File([blob as any], `cropped${file.value?.name}` as string, { type: blob?.type })
    }, cropperImage.type)
  }
}

return (_ctx: any,_cache: any) => {
  const _component_ModalDialog = _resolveComponent("ModalDialog")

  return (_openBlock(), _createBlock(_component_ModalDialog, {
      "use-v-if": false,
      "max-w-500px": "",
      flex: "",
      modelValue: cropperDialog.value,
      "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((cropperDialog).value = $event))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          "flex-1": "",
          "w-0": ""
        }, [
          _createElementVNode("div", {
            "text-lg": "",
            "text-center": "",
            my2: "",
            px3: ""
          }, [
            _createElementVNode("h1", null, _toDisplayString(_ctx.$t('action.edit')), 1 /* TEXT */)
          ]),
          _createElementVNode("div", { "aspect-ratio-1": "" }, [
            _createVNode(Cropper, {
              ref_key: "cropper", ref: cropper,
              class: "overflow-hidden w-full h-full",
              src: cropperImage.src,
              "resize-image": {
              adjustStencil: false,
            },
              "stencil-size": stencilSize,
              "stencil-props": {
              aspectRatio: __props.stencilAspectRatio,
              movable: false,
              resizable: false,
              handlers: {},
            },
              "image-restriction": "stencil"
            }, null, 8 /* PROPS */, ["src", "resize-image", "stencil-size", "stencil-props"])
          ]),
          _createElementVNode("div", { "m-4": "" }, [
            _createElementVNode("button", {
              "btn-solid": "",
              "w-full": "",
              rounded: "",
              "text-sm": "",
              onClick: _cache[1] || (_cache[1] = ($event: any) => (cropImage()))
            }, _toDisplayString(_ctx.$t('action.confirm')), 1 /* TEXT */)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["use-v-if", "modelValue"]))
}
}

})
