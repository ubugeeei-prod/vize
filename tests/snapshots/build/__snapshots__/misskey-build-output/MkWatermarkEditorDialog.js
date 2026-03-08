import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-copyright" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-crop-landscape" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-crop-portrait" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-up" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-down" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { ref, useTemplateRef, watch, onMounted, onUnmounted, reactive, nextTick } from 'vue'
import type { WatermarkLayers, WatermarkPreset } from '@/utility/watermark/WatermarkRenderer.js'
import { WatermarkRenderer } from '@/utility/watermark/WatermarkRenderer.js'
import { i18n } from '@/i18n.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkPreviewWithControls from '@/components/MkPreviewWithControls.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import XLayer from '@/components/MkWatermarkEditorDialog.Layer.vue'
import * as os from '@/os.js'
import { deepClone } from '@/utility/clone.js'
import { ensureSignin } from '@/i.js'
import { genId } from '@/utility/id.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkWatermarkEditorDialog',
  props: {
    presetEditMode: { type: Boolean, required: false },
    preset: { type: null, required: false },
    layers: { type: null, required: false },
    image: { type: File, required: false }
  },
  emits: ["ok", "presetOk", "cancel", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
function createTextLayer(): WatermarkPreset['layers'][number] {
	return {
		id: genId(),
		type: 'text',
		text: `(c) @${$i.username}`,
		align: { x: 'right', y: 'bottom', margin: 0 },
		scale: 0.3,
		angle: 0,
		opacity: 0.75,
		repeat: false,
		noBoundingBoxExpansion: false,
	};
}
function createImageLayer(): WatermarkPreset['layers'][number] {
	return {
		id: genId(),
		type: 'image',
		imageId: null,
		imageUrl: null,
		align: { x: 'right', y: 'bottom', margin: 0 },
		scale: 0.3,
		angle: 0,
		opacity: 0.75,
		repeat: false,
		noBoundingBoxExpansion: false,
		cover: false,
	};
}
function createQrLayer(): WatermarkPreset['layers'][number] {
	return {
		id: genId(),
		type: 'qr',
		data: '',
		align: { x: 'right', y: 'bottom', margin: 0 },
		scale: 0.3,
		opacity: 1,
	};
}
function createStripeLayer(): WatermarkPreset['layers'][number] {
	return {
		id: genId(),
		type: 'stripe',
		angle: 0.5,
		frequency: 10,
		threshold: 0.1,
		color: [1, 1, 1],
		opacity: 0.75,
	};
}
function createPolkadotLayer(): WatermarkPreset['layers'][number] {
	return {
		id: genId(),
		type: 'polkadot',
		angle: 0.5,
		scale: 3,
		majorRadius: 0.1,
		minorRadius: 0.25,
		majorOpacity: 0.75,
		minorOpacity: 0.5,
		minorDivisions: 4,
		color: [1, 1, 1],
		opacity: 0.75,
	};
}
function createCheckerLayer(): WatermarkPreset['layers'][number] {
	return {
		id: genId(),
		type: 'checker',
		angle: 0.5,
		scale: 3,
		color: [1, 1, 1],
		opacity: 0.75,
	};
}
const preset = deepClone(props.preset) ?? {
	id: genId(),
	name: '',
};
const layers = reactive<WatermarkLayers>(props.layers ?? []);
const dialog = useTemplateRef('dialog');
async function cancel() {
	if (props.presetEditMode) {
		const { canceled } = await os.confirm({
			type: 'question',
			text: i18n.ts._watermarkEditor.quitWithoutSaveConfirm,
		});
		if (canceled) return;
	}
	emit('cancel');
	dialog.value?.close();
}
watch(layers, async (newValue, oldValue) => {
	if (renderer != null) {
		renderer.render(layers);
	}
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
let renderer: WatermarkRenderer | null = null;
let imageBitmap: ImageBitmap | null = null;
async function initRenderer() {
	if (canvasEl.value == null) return;
	if (sampleImageType.value === '3_2') {
		renderer = new WatermarkRenderer({
			canvas: canvasEl.value,
			renderWidth: 1500,
			renderHeight: 1000,
			image: sampleImage_3_2,
		});
	} else if (sampleImageType.value === '2_3') {
		renderer = new WatermarkRenderer({
			canvas: canvasEl.value,
			renderWidth: 1000,
			renderHeight: 1500,
			image: sampleImage_2_3,
		});
	} else if (imageFile != null) {
		imageBitmap = await window.createImageBitmap(imageFile);
		const MAX_W = 1000;
		const MAX_H = 1000;
		let w = imageBitmap.width;
		let h = imageBitmap.height;
		if (w > MAX_W || h > MAX_H) {
			const scale = Math.min(MAX_W / w, MAX_H / h);
			w = Math.floor(w * scale);
			h = Math.floor(h * scale);
		}
		renderer = new WatermarkRenderer({
			canvas: canvasEl.value,
			renderWidth: w,
			renderHeight: h,
			image: imageBitmap,
		});
	}
	await renderer!.render(layers);
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
			text: i18n.ts._watermarkEditor.failedToLoadImage,
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
			layers: deepClone(layers),
		});
	} else {
		dialog.value?.close();
		if (renderer != null) {
			renderer.destroy();
			renderer = null;
		}
		emit('ok', layers);
	}
}
function addLayer(ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts._watermarkEditor.text,
		action: () => {
			layers.push(createTextLayer());
		},
	}, {
		text: i18n.ts._watermarkEditor.image,
		action: () => {
			layers.push(createImageLayer());
		},
	}, {
		text: i18n.ts._watermarkEditor.qr,
		action: () => {
			layers.push(createQrLayer());
		},
	}, {
		text: i18n.ts._watermarkEditor.stripe,
		action: () => {
			layers.push(createStripeLayer());
		},
	}, {
		text: i18n.ts._watermarkEditor.polkadot,
		action: () => {
			layers.push(createPolkadotLayer());
		},
	}, {
		text: i18n.ts._watermarkEditor.checker,
		action: () => {
			layers.push(createCheckerLayer());
		},
	}], ev.currentTarget ?? ev.target);
}
function swapUpLayer(layer: WatermarkPreset['layers'][number]) {
	const index = layers.findIndex(l => l.id === layer.id);
	if (index > 0) {
		const tmp = layers[index - 1];
		layers[index - 1] = layers[index];
		layers[index] = tmp;
	}
}
function swapDownLayer(layer: WatermarkPreset['layers'][number]) {
	const index = layers.findIndex(l => l.id === layer.id);
	if (index < layers.length - 1) {
		const tmp = layers[index + 1];
		layers[index + 1] = layers[index];
		layers[index] = tmp;
	}
}
function removeLayer(layer: WatermarkPreset['layers'][number]) {
	const index = layers.findIndex(l => l.id === layer.id);
	if (index !== -1) {
		layers.splice(index, 1);
	}
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
        _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.title), 1 /* TEXT */)
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
              _createElementVNode("div", { class: "_gaps_s" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(layers, (layer, i) => {
                  return (_openBlock(), _createBlock(MkFolder, {
                    key: layer.id,
                    defaultOpen: false,
                    canPage: false
                  }, {
                    label: _withCtx(() => [
                      (layer.type === 'text')
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts._watermarkEditor.text), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true),
                      (layer.type === 'image')
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts._watermarkEditor.image), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true),
                      (layer.type === 'qr')
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts._watermarkEditor.qr), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true),
                      (layer.type === 'stripe')
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts._watermarkEditor.stripe), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true),
                      (layer.type === 'polkadot')
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts._watermarkEditor.polkadot), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true),
                      (layer.type === 'checker')
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(i18n).ts._watermarkEditor.checker), 1 /* TEXT */))
                        : _createCommentVNode("v-if", true)
                    ]),
                    footer: _withCtx(() => [
                      _createElementVNode("div", { class: "_buttons" }, [
                        _createVNode(MkButton, {
                          iconOnly: "",
                          onClick: _cache[5] || (_cache[5] = ($event: any) => (removeLayer(layer)))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_5
                          ]),
                          _: 2 /* DYNAMIC */
                        }),
                        _createVNode(MkButton, {
                          iconOnly: "",
                          onClick: _cache[6] || (_cache[6] = ($event: any) => (swapUpLayer(layer)))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_6
                          ]),
                          _: 2 /* DYNAMIC */
                        }),
                        _createVNode(MkButton, {
                          iconOnly: "",
                          onClick: _cache[7] || (_cache[7] = ($event: any) => (swapDownLayer(layer)))
                        }, {
                          default: _withCtx(() => [
                            _hoisted_7
                          ]),
                          _: 2 /* DYNAMIC */
                        })
                      ])
                    ]),
                    default: _withCtx(() => [
                      _createVNode(XLayer, {
                        layer: layers[i],
                        "onUpdate:layer": _cache[8] || (_cache[8] = ($event: any) => ((layers[i]) = $event))
                      }, null, 8 /* PROPS */, ["layer"])
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["defaultOpen", "canPage"]))
                }), 128 /* KEYED_FRAGMENT */)),
                _createVNode(MkButton, {
                  rounded: "",
                  primary: "",
                  style: "margin: 0 auto;",
                  onClick: addLayer
                }, {
                  default: _withCtx(() => [
                    _hoisted_8
                  ]),
                  _: 1 /* STABLE */
                })
              ])
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
