import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { defineAsyncComponent, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'
import type { WatermarkPreset } from '@/utility/watermark/WatermarkRenderer.js'
import { WatermarkRenderer } from '@/utility/watermark/WatermarkRenderer.js'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { deepClone } from '@/utility/clone.js'
import MkFolder from '@/components/MkFolder.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'drive.WatermarkItem',
  props: {
    preset: { type: null, required: true }
  },
  emits: ["updatePreset", "del"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
async function edit() {
	const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkWatermarkEditorDialog.vue')), {
		presetEditMode: true,
		preset: deepClone(props.preset),
		layers: deepClone(props.preset.layers),
	}, {
		presetOk: (preset) => {
			emit('updatePreset', preset);
		},
		closed: () => dispose(),
	});
}
function del(ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.delete,
		action: () => {
			emit('del');
		},
	}], ev.currentTarget ?? ev.target);
}
const canvasEl = useTemplateRef('canvasEl');
const sampleImage = new Image();
sampleImage.src = '/client-assets/sample/3-2.jpg';
let renderer: WatermarkRenderer | null = null;
onMounted(() => {
	sampleImage.onload = async () => {
		watch(canvasEl, async () => {
			if (canvasEl.value == null) return;
			renderer = new WatermarkRenderer({
				canvas: canvasEl.value,
				renderWidth: 1500,
				renderHeight: 1000,
				image: sampleImage,
			});
			await renderer.render(props.preset.layers);
		}, { immediate: true });
	};
});
onUnmounted(() => {
	if (renderer != null) {
		renderer.destroy();
		renderer = null;
	}
});
watch(() => props.preset, async () => {
	if (renderer != null) {
		await renderer.render(props.preset.layers);
	}
}, { deep: true });

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkFolder, {
      defaultOpen: false,
      canPage: false
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      label: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.preset) + ": " + _toDisplayString(__props.preset.name === '' ? '(' + _unref(i18n).ts.noName + ')' : __props.preset.name), 1 /* TEXT */)
      ]),
      footer: _withCtx(() => [
        _createElementVNode("div", { class: "_buttons" }, [
          _createVNode(MkButton, { onClick: edit }, {
            default: _withCtx(() => [
              _hoisted_2,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.edit), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            danger: "",
            iconOnly: "",
            style: "margin-left: auto;",
            onClick: del
          }, {
            default: _withCtx(() => [
              _hoisted_3
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", null, [
          _createElementVNode("canvas", {
            ref_key: "canvasEl", ref: canvasEl,
            class: _normalizeClass(_ctx.$style.previewCanvas)
          }, null, 512 /* NEED_PATCH */)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["defaultOpen", "canPage"]))
}
}

})
