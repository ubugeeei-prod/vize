import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-ipad-horizontal" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-crop-landscape" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-crop-portrait" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{filename}")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{filename_without_ext}")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{caption}")
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{year}")
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{month}")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{day}")
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{hour}")
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{minute}")
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{second}")
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{0month}")
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{0day}")
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{0hour}")
const _hoisted_17 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{0minute}")
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{0second}")
const _hoisted_19 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{camera_model}")
const _hoisted_20 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{camera_lens_model}")
const _hoisted_21 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{camera_mm}")
const _hoisted_22 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{camera_mm_35}")
const _hoisted_23 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{camera_f}")
const _hoisted_24 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{camera_s}")
const _hoisted_25 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{camera_iso}")
const _hoisted_26 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{gps_lat}")
const _hoisted_27 = /*#__PURE__*/ _createElementVNode("code", { class: "_selectableAtomic" }, "{gps_long}")
import { ref, useTemplateRef, watch, onMounted, onUnmounted, reactive, nextTick } from 'vue'
import ExifReader from 'exifreader'
import { throttle } from 'throttle-debounce'
import MkPreviewWithControls from './MkPreviewWithControls.vue'
import type { ImageFrameParams, ImageFramePreset } from '@/utility/image-frame-renderer/ImageFrameRenderer.js'
import { ImageFrameRenderer } from '@/utility/image-frame-renderer/ImageFrameRenderer.js'
import { i18n } from '@/i18n.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkRange from '@/components/MkRange.vue'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkInfo from '@/components/MkInfo.vue'
import * as os from '@/os.js'
import { deepClone } from '@/utility/clone.js'
import { ensureSignin } from '@/i.js'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkImageFrameEditorDialog',
  props: {
    presetEditMode: { type: Boolean, required: false },
    preset: { type: null, required: false },
    params: { type: null, required: false },
    image: { type: File, required: false },
    imageCaption: { type: String, required: false },
    imageFilename: { type: String, required: false }
  },
  emits: ["ok", "presetOk", "cancel", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
const preset = deepClone(props.preset) ?? {
	id: genId(),
	name: '',
};
const params = reactive<ImageFrameParams>(deepClone(props.params) ?? {
	borderThickness: 0.05,
	borderRadius: 0,
	labelTop: {
		enabled: false,
		scale: 1.0,
		padding: 0.2,
		textBig: '',
		textSmall: '',
		centered: false,
		withQrCode: false,
	},
	labelBottom: {
		enabled: true,
		scale: 1.0,
		padding: 0.2,
		textBig: '{year}/{0month}/{0day}',
		textSmall: '{camera_mm}mm   f/{camera_f}   {camera_s}s   ISO{camera_iso}',
		centered: false,
		withQrCode: true,
	},
	bgColor: [1, 1, 1],
	fgColor: [0, 0, 0],
	font: 'sans-serif',
});
const dialog = useTemplateRef('dialog');
async function cancel() {
	if (props.presetEditMode) {
		const { canceled } = await os.confirm({
			type: 'question',
			text: i18n.ts._imageFrameEditor.quitWithoutSaveConfirm,
		});
		if (canceled) return;
	}
	dialog.value?.close();
}
const updateThrottled = throttle(50, () => {
	if (renderer != null) {
		renderer.render(params);
	}
});
watch(params, async (newValue, oldValue) => {
	updateThrottled();
}, { deep: true });
const canvasEl = useTemplateRef('canvasEl');
const sampleImage_3_2 = new Image();
sampleImage_3_2.src = '/client-assets/sample/3-2.jpg';
const sampleImage_3_2_loading = new Promise<void>(resolve => {
	sampleImage_3_2.onload = () => resolve();
});
const sampleImage_2_3 = new Image();
sampleImage_2_3.src = '/client-assets/sample/2-3.jpg';
const sampleImage_2_3_loading = new Promise<void>(resolve => {
	sampleImage_2_3.onload = () => resolve();
});
const sampleImageType = ref(props.image != null ? 'provided' : '3_2');
watch(sampleImageType, async () => {
	if (sampleImageType.value === 'provided') return;
	if (renderer != null) {
		renderer.destroy(false);
		renderer = null;
		initRenderer();
	}
});
let imageFile = props.image;
async function choiceImage() {
	const files = await os.chooseFileFromPc({ multiple: false });
	if (files.length === 0) return;
	imageFile = files[0];
	sampleImageType.value = 'provided';
	if (renderer != null) {
		renderer.destroy(false);
		renderer = null;
		initRenderer();
	}
}
let renderer: ImageFrameRenderer | null = null;
let imageBitmap: ImageBitmap | null = null;
async function initRenderer() {
	if (canvasEl.value == null) return;
	if (sampleImageType.value === '3_2') {
		renderer = new ImageFrameRenderer({
			canvas: canvasEl.value,
			image: sampleImage_3_2,
			exif: null,
			caption: 'Example caption',
			filename: 'example_file_name.jpg',
			renderAsPreview: true,
		});
	} else if (sampleImageType.value === '2_3') {
		renderer = new ImageFrameRenderer({
			canvas: canvasEl.value,
			image: sampleImage_2_3,
			exif: null,
			caption: 'Example caption',
			filename: 'example_file_name.jpg',
			renderAsPreview: true,
		});
	} else if (imageFile != null) {
		imageBitmap = await window.createImageBitmap(imageFile);
		const exif = ExifReader.load(await imageFile.arrayBuffer());
		renderer = new ImageFrameRenderer({
			canvas: canvasEl.value,
			image: imageBitmap,
			exif: exif,
			caption: props.imageCaption ?? null,
			filename: props.imageFilename ?? null,
			renderAsPreview: true,
		});
	}
	await renderer!.render(params);
}
onMounted(async () => {
	const closeWaiting = os.waiting();
	await nextTick(); // waitingがレンダリングされるまで待つ
	await sampleImage_3_2_loading;
	await sampleImage_2_3_loading;
	try {
		await initRenderer();
	} catch (err) {
		console.error(err);
		os.alert({
			type: 'error',
			text: i18n.ts._imageFrameEditor.failedToLoadImage,
		});
	}
	closeWaiting();
});
onUnmounted(() => {
	if (renderer != null) {
		renderer.destroy();
		renderer = null;
	}
	if (imageBitmap != null) {
		imageBitmap.close();
		imageBitmap = null;
	}
});
async function save() {
	if (props.presetEditMode) {
		const { canceled, result: name } = await os.inputText({
			title: i18n.ts.name,
			default: preset.name,
		});
		if (canceled) return;
		preset.name = name || '';
		dialog.value?.close();
		if (renderer != null) {
			renderer.destroy();
			renderer = null;
		}
		emit('presetOk', {
			...preset,
			params: deepClone(params),
		});
	} else {
		dialog.value?.close();
		if (renderer != null) {
			renderer.destroy();
			renderer = null;
		}
		emit('ok', params);
	}
}
function getHex(c: [number, number, number]) {
	return `#${c.map(x => Math.round(x * 255).toString(16).padStart(2, '0')).join('')}`;
}
function getRgb(hex: string | number): [number, number, number] | null {
	if (
		typeof hex === 'number' ||
		typeof hex !== 'string' ||
		!/^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.test(hex)
	) {
		return null;
	}
	const m = hex.slice(1).match(/[0-9a-fA-F]{2}/g);
	if (m == null) return [0, 0, 0];
	return m.map(x => parseInt(x, 16) / 255) as [number, number, number];
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 1000,
      height: 600,
      scroll: false,
      withOkButton: true,
      onClose: _cache[0] || (_cache[0] = ($event: any) => (cancel())),
      onOk: _cache[1] || (_cache[1] = ($event: any) => (save())),
      onClosed: _cache[2] || (_cache[2] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createTextVNode(" "),
        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.title), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createVNode(MkPreviewWithControls, null, {
          preview: _withCtx(() => [
            _createElementVNode("canvas", {
              ref_key: "canvasEl", ref: canvasEl,
              class: _normalizeClass(_ctx.$style.previewCanvas)
            }, null, 512 /* NEED_PATCH */),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.previewContainer)
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(["_acrylic", _ctx.$style.previewTitle])
              }, _toDisplayString(_unref(i18n).ts.preview), 1 /* TEXT */),
              (props.image == null)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(["_acrylic", _ctx.$style.previewControls])
                }, [
                  _createElementVNode("button", {
                    class: _normalizeClass(["_button", [_ctx.$style.previewControlsButton, sampleImageType.value === '3_2' ? _ctx.$style.active : null]]),
                    onClick: _cache[3] || (_cache[3] = ($event: any) => (sampleImageType.value = '3_2'))
                  }, [
                    _hoisted_2
                  ], 2 /* CLASS */),
                  _createElementVNode("button", {
                    class: _normalizeClass(["_button", [_ctx.$style.previewControlsButton, sampleImageType.value === '2_3' ? _ctx.$style.active : null]]),
                    onClick: _cache[4] || (_cache[4] = ($event: any) => (sampleImageType.value = '2_3'))
                  }, [
                    _hoisted_3
                  ], 2 /* CLASS */),
                  _createElementVNode("button", {
                    class: _normalizeClass(["_button", [_ctx.$style.previewControlsButton]]),
                    onClick: choiceImage
                  }, [
                    _hoisted_4
                  ])
                ]))
                : _createCommentVNode("v-if", true)
            ])
          ]),
          controls: _withCtx(() => [
            _createElementVNode("div", { class: "_spacer _gaps" }, [
              _createVNode(MkRange, {
                min: 0,
                max: 0.2,
                step: 0.01,
                continuousUpdate: true,
                modelValue: params.borderThickness,
                "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((params.borderThickness) = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.borderThickness), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["min", "max", "step", "continuousUpdate", "modelValue"]),
              _createVNode(MkInput, {
                modelValue: getHex(params.bgColor),
                type: "color",
                "onUpdate:modelValue": _cache[6] || (_cache[6] = v => { const c = getRgb(v); if (_ctx.c != null) params.bgColor = _ctx.c; })
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.backgroundColor), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"]),
              _createVNode(MkInput, {
                modelValue: getHex(params.fgColor),
                type: "color",
                "onUpdate:modelValue": _cache[7] || (_cache[7] = v => { const c = getRgb(v); if (_ctx.c != null) params.fgColor = _ctx.c; })
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.textColor), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"]),
              _createVNode(MkSelect, {
                items: [
  						{ label: _unref(i18n).ts._imageFrameEditor.fontSansSerif, value: 'sans-serif' },
  						{ label: _unref(i18n).ts._imageFrameEditor.fontSerif, value: 'serif' },
  					],
                modelValue: params.font,
                "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((params.font) = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.font), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"]),
              _createVNode(MkFolder, { defaultOpen: params.labelTop.enabled }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.header), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      modelValue: params.labelTop.enabled,
                      "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((params.labelTop.enabled) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.show), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkRange, {
                      min: 0.01,
                      max: 0.5,
                      step: 0.01,
                      continuousUpdate: true,
                      modelValue: params.labelTop.padding,
                      "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((params.labelTop.padding) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.labelThickness), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "continuousUpdate", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0.5,
                      max: 2.0,
                      step: 0.01,
                      continuousUpdate: true,
                      modelValue: params.labelTop.scale,
                      "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event: any) => ((params.labelTop.scale) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.labelScale), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "continuousUpdate", "modelValue"]),
                    _createVNode(MkSwitch, {
                      modelValue: params.labelTop.centered,
                      "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event: any) => ((params.labelTop.centered) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.centered), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkInput, {
                      modelValue: params.labelTop.textBig,
                      "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event: any) => ((params.labelTop.textBig) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.captionMain), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkTextarea, {
                      modelValue: params.labelTop.textSmall,
                      "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event: any) => ((params.labelTop.textSmall) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.captionSub), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkSwitch, {
                      modelValue: params.labelTop.withQrCode,
                      "onUpdate:modelValue": _cache[15] || (_cache[15] = ($event: any) => ((params.labelTop.withQrCode) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.withQrCode), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["defaultOpen"]),
              _createVNode(MkFolder, { defaultOpen: params.labelBottom.enabled }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.footer), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps" }, [
                    _createVNode(MkSwitch, {
                      modelValue: params.labelBottom.enabled,
                      "onUpdate:modelValue": _cache[16] || (_cache[16] = ($event: any) => ((params.labelBottom.enabled) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts.show), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkRange, {
                      min: 0.01,
                      max: 0.5,
                      step: 0.01,
                      continuousUpdate: true,
                      modelValue: params.labelBottom.padding,
                      "onUpdate:modelValue": _cache[17] || (_cache[17] = ($event: any) => ((params.labelBottom.padding) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.labelThickness), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "continuousUpdate", "modelValue"]),
                    _createVNode(MkRange, {
                      min: 0.5,
                      max: 2.0,
                      step: 0.01,
                      continuousUpdate: true,
                      modelValue: params.labelBottom.scale,
                      "onUpdate:modelValue": _cache[18] || (_cache[18] = ($event: any) => ((params.labelBottom.scale) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.labelScale), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["min", "max", "step", "continuousUpdate", "modelValue"]),
                    _createVNode(MkSwitch, {
                      modelValue: params.labelBottom.centered,
                      "onUpdate:modelValue": _cache[19] || (_cache[19] = ($event: any) => ((params.labelBottom.centered) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.centered), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkInput, {
                      modelValue: params.labelBottom.textBig,
                      "onUpdate:modelValue": _cache[20] || (_cache[20] = ($event: any) => ((params.labelBottom.textBig) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.captionMain), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkTextarea, {
                      modelValue: params.labelBottom.textSmall,
                      "onUpdate:modelValue": _cache[21] || (_cache[21] = ($event: any) => ((params.labelBottom.textSmall) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.captionSub), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"]),
                    _createVNode(MkSwitch, {
                      modelValue: params.labelBottom.withQrCode,
                      "onUpdate:modelValue": _cache[22] || (_cache[22] = ($event: any) => ((params.labelBottom.withQrCode) = $event))
                    }, {
                      label: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageFrameEditor.withQrCode), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["defaultOpen"]),
              _createVNode(MkInfo, null, {
                default: _withCtx(() => [
                  _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._imageFrameEditor.availableVariables) + ":", 1 /* TEXT */),
                  _createElementVNode("div", null, [
                    _hoisted_5,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.filename), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_6,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.filename_without_ext), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_7,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.caption), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_8,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.year), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_9,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.month), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_10,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.day), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_11,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.hour), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_12,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.minute), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_13,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.second), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_14,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.month) + " (" + _toDisplayString(_unref(i18n).ts.zeroPadding) + ")", 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_15,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.day) + " (" + _toDisplayString(_unref(i18n).ts.zeroPadding) + ")", 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_16,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.hour) + " (" + _toDisplayString(_unref(i18n).ts.zeroPadding) + ")", 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_17,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.minute) + " (" + _toDisplayString(_unref(i18n).ts.zeroPadding) + ")", 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_18,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.second) + " (" + _toDisplayString(_unref(i18n).ts.zeroPadding) + ")", 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_19,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.camera_model), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_20,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.camera_lens_model), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_21,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.camera_mm), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_22,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.camera_mm_35), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_23,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.camera_f), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_24,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.camera_s), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_25,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.camera_iso), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_26,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.gps_lat), 1 /* TEXT */)
                  ]),
                  _createElementVNode("div", null, [
                    _hoisted_27,
                    _createTextVNode(" - " + _toDisplayString(_unref(i18n).ts._imageEditing._vars.gps_long), 1 /* TEXT */)
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ])
          ]),
          _: 1 /* STABLE */
        })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "height", "scroll", "withOkButton"]))
}
}

})
