import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-sparkles" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { ref, useTemplateRef, watch, onMounted, onUnmounted, reactive, nextTick } from 'vue'
import type { ImageEffectorLayer } from '@/utility/image-effector/ImageEffector.js'
import { i18n } from '@/i18n.js'
import { ImageEffector } from '@/utility/image-effector/ImageEffector.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import MkPreviewWithControls from '@/components/MkPreviewWithControls.vue'
import MkButton from '@/components/MkButton.vue'
import XLayer from '@/components/MkImageEffectorDialog.Layer.vue'
import * as os from '@/os.js'
import { FXS } from '@/utility/image-effector/fxs.js'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkImageEffectorDialog',
  props: {
    image: { type: File, required: true }
  },
  emits: ["ok", "cancel", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const dialog = useTemplateRef('dialog');
async function cancel() {
	if (layers.length > 0) {
		const { canceled } = await os.confirm({
			type: 'warning',
			text: i18n.ts._imageEffector.discardChangesConfirm,
		});
		if (canceled) return;
	}
	emit('cancel');
	dialog.value?.close();
}
const layers = reactive<ImageEffectorLayer[]>([]);
watch(layers, async () => {
	if (renderer != null) {
		renderer.render(layers);
	}
}, { deep: true });
function addEffect(ev: PointerEvent) {
	os.popupMenu(Object.entries(FXS).map(([id, fx]) => ({
		text: fx.uiDefinition.name,
		action: () => {
			layers.push({
				id: genId(),
				fxId: id as keyof typeof FXS,
				params: Object.fromEntries(Object.entries(fx.uiDefinition.params).map(([k, v]) => [k, v.default])),
			} as ImageEffectorLayer);
		},
	})), ev.currentTarget ?? ev.target);
}
function onLayerSwapUp(layer: ImageEffectorLayer) {
	const index = layers.indexOf(layer);
	if (index > 0) {
		layers.splice(index, 1);
		layers.splice(index - 1, 0, layer);
	}
}
function onLayerSwapDown(layer: ImageEffectorLayer) {
	const index = layers.indexOf(layer);
	if (index < layers.length - 1) {
		layers.splice(index, 1);
		layers.splice(index + 1, 0, layer);
	}
}
function onLayerDelete(layer: ImageEffectorLayer) {
	const index = layers.indexOf(layer);
	if (index !== -1) {
		layers.splice(index, 1);
	}
}
const canvasEl = useTemplateRef('canvasEl');
let renderer: ImageEffector | null = null;
let imageBitmap: ImageBitmap | null = null;
onMounted(async () => {
	if (canvasEl.value == null) return;
	const closeWaiting = os.waiting();
	await nextTick(); // waitingがレンダリングされるまで待つ
	try {
		imageBitmap = await window.createImageBitmap(props.image);
		const MAX_W = 1000;
		const MAX_H = 1000;
		let w = imageBitmap.width;
		let h = imageBitmap.height;
		if (w > MAX_W || h > MAX_H) {
			const scale = Math.min(MAX_W / w, MAX_H / h);
			w = Math.floor(w * scale);
			h = Math.floor(h * scale);
		}
		renderer = new ImageEffector({
			canvas: canvasEl.value,
			renderWidth: w,
			renderHeight: h,
			image: imageBitmap,
		});
		await renderer.render(layers);
	} catch (err) {
		console.error(err);
		os.alert({
			type: 'error',
			text: i18n.ts._imageEffector.failedToLoadImage,
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
	if (layers.length === 0 || renderer == null || imageBitmap == null || canvasEl.value == null) {
		cancel();
		return;
	}
	const closeWaiting = os.waiting();
	await nextTick(); // waitingがレンダリングされるまで待つ
	renderer.changeResolution(imageBitmap.width, imageBitmap.height); // 本番レンダリングのためオリジナル画質に戻す
	await renderer.render(layers); // toBlobの直前にレンダリングしないと何故か壊れる
	canvasEl.value.toBlob((blob) => {
		emit('ok', new File([blob!], `image-${Date.now()}.png`, { type: 'image/png' }));
		dialog.value?.close();
		closeWaiting();
	}, 'image/png');
}
const enabled = ref(true);
watch(enabled, () => {
	if (renderer != null) {
		if (enabled.value) {
			renderer.render(layers);
		} else {
			renderer.render([]);
		}
	}
});
const penMode = ref<'fill' | 'blur' | 'pixelate' | null>(null);
function showPenMenu(ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts._imageEffector._fxs.fill,
		action: () => {
			penMode.value = 'fill';
		},
	}, {
		text: i18n.ts._imageEffector._fxs.blur,
		action: () => {
			penMode.value = 'blur';
		},
	}, {
		text: i18n.ts._imageEffector._fxs.pixelate,
		action: () => {
			penMode.value = 'pixelate';
		},
	}], ev.currentTarget ?? ev.target);
}
function onImagePointerdown(ev: PointerEvent) {
	if (canvasEl.value == null || imageBitmap == null || penMode.value == null) return;
	const AW = canvasEl.value.clientWidth;
	const AH = canvasEl.value.clientHeight;
	const BW = imageBitmap.width;
	const BH = imageBitmap.height;
	let xOffset = 0;
	let yOffset = 0;
	if (AW / AH < BW / BH) { // 横長
		yOffset = AH - BH * (AW / BW);
	} else { // 縦長
		xOffset = AW - BW * (AH / BH);
	}
	xOffset /= 2;
	yOffset /= 2;
	let startX = ev.offsetX - xOffset;
	let startY = ev.offsetY - yOffset;
	if (AW / AH < BW / BH) { // 横長
		startX = startX / (Math.max(AW, AH) / Math.max(BH / BW, 1));
		startY = startY / (Math.max(AW, AH) / Math.max(BW / BH, 1));
	} else { // 縦長
		startX = startX / (Math.min(AW, AH) / Math.max(BH / BW, 1));
		startY = startY / (Math.min(AW, AH) / Math.max(BW / BH, 1));
	}
	const id = genId();
	if (penMode.value === 'fill') {
		layers.push({
			id,
			fxId: 'fill',
			params: {
				offsetX: 0,
				offsetY: 0,
				scaleX: 0.1,
				scaleY: 0.1,
				angle: 0,
				opacity: 1,
				color: [1, 1, 1],
				ellipse: false,
			},
		});
	} else if (penMode.value === 'blur') {
		layers.push({
			id,
			fxId: 'blur',
			params: {
				offsetX: 0,
				offsetY: 0,
				scaleX: 0.1,
				scaleY: 0.1,
				angle: 0,
				radius: 10,
				ellipse: false,
			},
		});
	} else if (penMode.value === 'pixelate') {
		layers.push({
			id,
			fxId: 'pixelate',
			params: {
				offsetX: 0,
				offsetY: 0,
				scaleX: 0.1,
				scaleY: 0.1,
				angle: 0,
				strength: 0.2,
				ellipse: false,
			},
		});
	}
	_move(ev.offsetX, ev.offsetY);
	function _move(pointerX: number, pointerY: number) {
		let x = pointerX - xOffset;
		let y = pointerY - yOffset;
		if (AW / AH < BW / BH) { // 横長
			x = x / (Math.max(AW, AH) / Math.max(BH / BW, 1));
			y = y / (Math.max(AW, AH) / Math.max(BW / BH, 1));
		} else { // 縦長
			x = x / (Math.min(AW, AH) / Math.max(BH / BW, 1));
			y = y / (Math.min(AW, AH) / Math.max(BW / BH, 1));
		}
		const scaleX = Math.abs(x - startX);
		const scaleY = Math.abs(y - startY);
		const layerIndex = layers.findIndex((l) => l.id === id);
		const layer = layerIndex !== -1 ? (layers[layerIndex] as Extract<ImageEffectorLayer, { fxId: 'fill' } | { fxId: 'blur' } | { fxId: 'pixelate' }>) : null;
		if (layer != null) {
			layer.params.offsetX = (x + startX) - 1;
			layer.params.offsetY = (y + startY) - 1;
			layer.params.scaleX = scaleX;
			layer.params.scaleY = scaleY;
			layers[layerIndex] = layer;
		}
	}
	function move(ev: PointerEvent) {
		_move(ev.offsetX, ev.offsetY);
	}
	function up() {
		canvasEl.value?.removeEventListener('pointermove', move);
		canvasEl.value?.removeEventListener('pointerup', up);
		canvasEl.value?.removeEventListener('pointercancel', up);
		canvasEl.value?.releasePointerCapture(ev.pointerId);
		penMode.value = null;
	}
	canvasEl.value.addEventListener('pointermove', move);
	canvasEl.value.addEventListener('pointerup', up);
	canvasEl.value.setPointerCapture(ev.pointerId);
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
        _createTextVNode(_toDisplayString(_unref(i18n).ts._imageEffector.title), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createVNode(MkPreviewWithControls, null, {
          preview: _withCtx(() => [
            _createElementVNode("canvas", {
              ref_key: "canvasEl", ref: canvasEl,
              class: _normalizeClass(_ctx.$style.previewCanvas),
              onPointerdown: _withModifiers(onImagePointerdown, ["prevent","stop"])
            }, null, 32 /* NEED_HYDRATION */),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.previewContainer)
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(["_acrylic", _ctx.$style.previewTitle])
              }, _toDisplayString(_unref(i18n).ts.preview), 1 /* TEXT */),
              _createElementVNode("div", {
                class: _normalizeClass(["_acrylic", _ctx.$style.editControls])
              }, [
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", [_ctx.$style.previewControlsButton, penMode.value != null ? _ctx.$style.active : null]]),
                  onClick: showPenMenu
                }, [
                  _hoisted_2
                ], 2 /* CLASS */)
              ]),
              _createElementVNode("div", {
                class: _normalizeClass(["_acrylic", _ctx.$style.previewControls])
              }, [
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", [_ctx.$style.previewControlsButton, !enabled.value ? _ctx.$style.active : null]]),
                  onClick: _cache[3] || (_cache[3] = ($event: any) => (enabled.value = false))
                }, "Before", 2 /* CLASS */),
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", [_ctx.$style.previewControlsButton, enabled.value ? _ctx.$style.active : null]]),
                  onClick: _cache[4] || (_cache[4] = ($event: any) => (enabled.value = true))
                }, "After", 2 /* CLASS */)
              ])
            ])
          ]),
          controls: _withCtx(() => [
            _createElementVNode("div", { class: "_spacer _gaps" }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(layers, (layer, i) => {
                return (_openBlock(), _createBlock(XLayer, {
                  key: layer.id,
                  onDel: _cache[5] || (_cache[5] = ($event: any) => (onLayerDelete(layer))),
                  onSwapUp: _cache[6] || (_cache[6] = ($event: any) => (onLayerSwapUp(layer))),
                  onSwapDown: _cache[7] || (_cache[7] = ($event: any) => (onLayerSwapDown(layer))),
                  layer: layers[i],
                  "onUpdate:layer": _cache[8] || (_cache[8] = ($event: any) => ((layers[i]) = $event))
                }, null, 8 /* PROPS */, ["layer"]))
              }), 128 /* KEYED_FRAGMENT */)),
              _createVNode(MkButton, {
                rounded: "",
                primary: "",
                style: "margin: 0 auto;",
                onClick: addEffect
              }, {
                default: _withCtx(() => [
                  _hoisted_3,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._imageEffector.addEffect), 1 /* TEXT */)
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
