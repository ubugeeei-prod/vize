import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:upload-line": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { block: "true", "i-ri:loader-4-line": "true", "text-4xl": "true" })
import type { FileWithHandle } from 'browser-fs-access'
import { fileOpen } from 'browser-fs-access'

export default /*@__PURE__*/_defineComponent({
  __name: 'CommonInputImage',
  props: {
    original: { type: String, required: false },
    allowedFileSize: { type: Number, required: false, default: 1024 * 1024 * 5 },
    imgClass: { type: String, required: false },
    loading: { type: Boolean, required: false },
    "modelValue": {}
  },
  emits: ["pick", "error", "update:modelValue"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const file = _useModel(__props, "modelValue")
const { t } = useI18n()
const defaultImage = computed(() => __props.original || '')
/** Preview of selected images */
const previewImage = ref('')
/** The current images on display */
const imageSrc = computed<string>(() => previewImage.value || defaultImage.value)
const mimeTypes = currentInstance.value!.configuration?.mediaAttachments.supportedMimeTypes.filter(mime => mime.startsWith('image/'))
  ?? ['image/jpeg', 'image/png']
async function pickImage() {
  if (import.meta.server)
    return
  const image = await fileOpen({
    description: 'Image',
    mimeTypes,
  })
  if (!mimeTypes.includes(image.type)) {
    emit('error', 1, t('error.unsupported_file_format'))
    return
  }
  else if (image.size > __props.allowedFileSize) {
    emit('error', 2, t('error.file_size_cannot_exceed_n_mb', [5]))
    return
  }
  file.value = image
  emit('pick', file.value)
}
watch(file, (image, _, onCleanup) => {
  let expired = false
  onCleanup(() => expired = true)
  if (!image) {
    previewImage.value = ''
    return
  }
  const reader = new FileReader()
  reader.readAsDataURL(image)
  reader.onload = (evt) => {
    if (expired)
      return
    previewImage.value = evt.target?.result as string
  }
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("label", {
      class: "bg-slate-500/10 focus-within:(outline outline-primary)",
      relative: "",
      flex: "",
      "justify-center": "",
      "items-center": "",
      "cursor-pointer": "",
      "of-hidden": "",
      onClick: pickImage
    }, [ (imageSrc.value) ? (_openBlock(), _createElementBlock("img", {
          key: 0,
          src: imageSrc.value,
          class: _normalizeClass(__props.imgClass || ''),
          "object-cover": "",
          "w-full": "",
          "h-full": ""
        })) : _createCommentVNode("v-if", true), _createElementVNode("span", {
        absolute: "",
        bg: "black/50",
        "text-white": "",
        "rounded-full": "",
        "text-xl": "",
        w12: "",
        h12: "",
        flex: "",
        "justify-center": "",
        "items-center": "",
        hover: "bg-black/40 text-primary"
      }, [ _hoisted_1 ]), (__props.loading) ? (_openBlock(), _createElementBlock("span", {
          key: 0,
          absolute: "",
          "inset-0": "",
          bg: "black/30",
          "text-white": "",
          flex: "",
          "justify-center": "",
          "items-center": ""
        }, [ _createElementVNode("span", { class: "animate-spin animate-duration-[2.5s] preserve-3d" }, [ _hoisted_2 ]) ])) : _createCommentVNode("v-if", true) ]))
}
}

})
