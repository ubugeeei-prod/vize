import { useModel as _useModel } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { ref, onMounted, computed } from 'vue'
import * as Misskey from 'misskey-js'
import type { WatermarkPreset } from '@/utility/watermark/WatermarkRenderer.js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkRange from '@/components/MkRange.vue'
import FormSlot from '@/components/form/slot.vue'
import MkPositionSelector from '@/components/MkPositionSelector.vue'
import * as os from '@/os.js'
import { selectFile } from '@/utility/drive.js'
import { misskeyApi } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkWatermarkEditorDialog.Layer',
  props: {
    "layer": { required: true },
    "layerModifiers": {},
  },
  emits: ["update:layer"],
  setup(__props) {

const layer = _useModel(__props, "layer")
const layerPreserveBoundingRect = computed({
	get: () => {
		if (layer.value.type === 'text' || layer.value.type === 'image') {
			return !layer.value.noBoundingBoxExpansion;
		}
		return false;
	},
	set: (v: boolean) => {
		if (layer.value.type === 'text' || layer.value.type === 'image') {
			layer.value.noBoundingBoxExpansion = !v;
		}
	},
});
const driveFile = ref<Misskey.entities.DriveFile | null>(null);
const driveFileError = ref(false);
onMounted(async () => {
	if (layer.value.type === 'image' && layer.value.imageId != null) {
		await misskeyApi('drive/files/show', {
			fileId: layer.value.imageId,
		}).then((res) => {
			driveFile.value = res;
		}).catch((err) => {
			driveFileError.value = true;
		});
	}
});
function chooseFile(ev: PointerEvent) {
	selectFile({
		anchorElement: ev.currentTarget ?? ev.target,
		multiple: false,
		label: i18n.ts.selectFile,
		features: {
			watermark: false,
		},
	}).then((file) => {
		if (layer.value.type !== 'image') return;
		if (!file.type.startsWith('image')) {
			os.alert({
				type: 'warning',
				title: i18n.ts._watermarkEditor.driveFileTypeWarn,
				text: i18n.ts._watermarkEditor.driveFileTypeWarnDescription,
			});
			return;
		}
		layer.value.imageId = file.id;
		layer.value.imageUrl = file.url;
		driveFileError.value = false;
	});
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_gaps", _ctx.$style.root])
    }, [ (layer.value.type === 'text') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(MkInput, {
            modelValue: layer.value.text,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((layer.value.text) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.text), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(FormSlot, null, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.position), 1 /* TEXT */)
            ]),
            default: _withCtx(() => [
              _createVNode(MkPositionSelector, {
                x: layer.value.align.x,
                "onUpdate:x": _cache[1] || (_cache[1] = ($event: any) => ((layer.value.align.x) = $event)),
                y: layer.value.align.y,
                "onUpdate:y": _cache[2] || (_cache[2] = ($event: any) => ((layer.value.align.y) = $event))
              }, null, 8 /* PROPS */, ["x", "y"])
            ]),
            _: 1 /* STABLE */
          }), _createVNode(MkRange, {
            modelValue: layer.value.align.margin ?? 0,
            min: 0,
            max: 0.25,
            step: 0.01,
            textConverter: (v) => (v * 100).toFixed(1) + '%',
            continuousUpdate: "",
            "onUpdate:modelValue": _cache[3] || (_cache[3] = (v) => layer.value.align.margin = v)
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.margin), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue", "min", "max", "step", "textConverter"]), _createVNode(MkRange, {
            min: 0,
            max: 1,
            step: 0.01,
            textConverter: (v) => (v * 100).toFixed(1) + '%',
            continuousUpdate: "",
            modelValue: layer.value.scale,
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((layer.value.scale) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.scale), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]), _createVNode(MkRange, {
            min: -1,
            max: 1,
            step: 0.01,
            continuousUpdate: "",
            modelValue: layer.value.angle,
            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((layer.value.angle) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.angle), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
            min: 0,
            max: 1,
            step: 0.01,
            textConverter: (v) => (v * 100).toFixed(1) + '%',
            continuousUpdate: "",
            modelValue: layer.value.opacity,
            "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((layer.value.opacity) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.opacity), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]), _createVNode(MkSwitch, {
            modelValue: layer.value.repeat,
            "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((layer.value.repeat) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.repeat), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
            modelValue: layerPreserveBoundingRect.value,
            "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((layerPreserveBoundingRect).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.preserveBoundingRect), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : (layer.value.type === 'image') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createVNode(MkButton, {
              inline: "",
              rounded: "",
              primary: "",
              onClick: chooseFile
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.selectFile), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }), _createVNode(FormSlot, null, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.position), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createVNode(MkPositionSelector, {
                  x: layer.value.align.x,
                  "onUpdate:x": _cache[9] || (_cache[9] = ($event: any) => ((layer.value.align.x) = $event)),
                  y: layer.value.align.y,
                  "onUpdate:y": _cache[10] || (_cache[10] = ($event: any) => ((layer.value.align.y) = $event))
                }, null, 8 /* PROPS */, ["x", "y"])
              ]),
              _: 1 /* STABLE */
            }), _createVNode(MkRange, {
              modelValue: layer.value.align.margin ?? 0,
              min: 0,
              max: 0.25,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              "onUpdate:modelValue": _cache[11] || (_cache[11] = (v) => layer.value.align.margin = v)
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.margin), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue", "min", "max", "step", "textConverter"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.scale,
              "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event: any) => ((layer.value.scale) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.scale), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]), _createVNode(MkRange, {
              min: -1,
              max: 1,
              step: 0.01,
              continuousUpdate: "",
              modelValue: layer.value.angle,
              "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event: any) => ((layer.value.angle) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.angle), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.opacity,
              "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event: any) => ((layer.value.opacity) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.opacity), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]), _createVNode(MkSwitch, {
              modelValue: layer.value.repeat,
              "onUpdate:modelValue": _cache[15] || (_cache[15] = ($event: any) => ((layer.value.repeat) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.repeat), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
              modelValue: layer.value.cover,
              "onUpdate:modelValue": _cache[16] || (_cache[16] = ($event: any) => ((layer.value.cover) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.cover), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
              modelValue: layerPreserveBoundingRect.value,
              "onUpdate:modelValue": _cache[17] || (_cache[17] = ($event: any) => ((layerPreserveBoundingRect).value = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.preserveBoundingRect), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : (layer.value.type === 'qr') ? (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _createVNode(MkInput, {
              debounce: "",
              modelValue: layer.value.data,
              "onUpdate:modelValue": _cache[18] || (_cache[18] = ($event: any) => ((layer.value.data) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.text), 1 /* TEXT */)
              ]),
              caption: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.leaveBlankToAccountUrl), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]), _createVNode(FormSlot, null, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.position), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createVNode(MkPositionSelector, {
                  x: layer.value.align.x,
                  "onUpdate:x": _cache[19] || (_cache[19] = ($event: any) => ((layer.value.align.x) = $event)),
                  y: layer.value.align.y,
                  "onUpdate:y": _cache[20] || (_cache[20] = ($event: any) => ((layer.value.align.y) = $event))
                }, null, 8 /* PROPS */, ["x", "y"])
              ]),
              _: 1 /* STABLE */
            }), _createVNode(MkRange, {
              modelValue: layer.value.align.margin ?? 0,
              min: 0,
              max: 0.25,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              "onUpdate:modelValue": _cache[21] || (_cache[21] = (v) => layer.value.align.margin = v)
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.margin), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue", "min", "max", "step", "textConverter"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.scale,
              "onUpdate:modelValue": _cache[22] || (_cache[22] = ($event: any) => ((layer.value.scale) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.scale), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.opacity,
              "onUpdate:modelValue": _cache[23] || (_cache[23] = ($event: any) => ((layer.value.opacity) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.opacity), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : (layer.value.type === 'stripe') ? (_openBlock(), _createElementBlock(_Fragment, { key: 3 }, [ _createVNode(MkRange, {
              min: 1,
              max: 30,
              step: 0.01,
              continuousUpdate: "",
              modelValue: layer.value.frequency,
              "onUpdate:modelValue": _cache[24] || (_cache[24] = ($event: any) => ((layer.value.frequency) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.stripeFrequency), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              continuousUpdate: "",
              modelValue: layer.value.threshold,
              "onUpdate:modelValue": _cache[25] || (_cache[25] = ($event: any) => ((layer.value.threshold) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.stripeWidth), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: -1,
              max: 1,
              step: 0.01,
              continuousUpdate: "",
              modelValue: layer.value.angle,
              "onUpdate:modelValue": _cache[26] || (_cache[26] = ($event: any) => ((layer.value.angle) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.angle), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.opacity,
              "onUpdate:modelValue": _cache[27] || (_cache[27] = ($event: any) => ((layer.value.opacity) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.opacity), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : (layer.value.type === 'polkadot') ? (_openBlock(), _createElementBlock(_Fragment, { key: 4 }, [ _createVNode(MkRange, {
              min: -1,
              max: 1,
              step: 0.01,
              continuousUpdate: "",
              modelValue: layer.value.angle,
              "onUpdate:modelValue": _cache[28] || (_cache[28] = ($event: any) => ((layer.value.angle) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.angle), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 10,
              step: 0.01,
              continuousUpdate: "",
              modelValue: layer.value.scale,
              "onUpdate:modelValue": _cache[29] || (_cache[29] = ($event: any) => ((layer.value.scale) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.scale), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.majorRadius,
              "onUpdate:modelValue": _cache[30] || (_cache[30] = ($event: any) => ((layer.value.majorRadius) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.polkadotMainDotRadius), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.majorOpacity,
              "onUpdate:modelValue": _cache[31] || (_cache[31] = ($event: any) => ((layer.value.majorOpacity) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.polkadotMainDotOpacity), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 16,
              step: 1,
              continuousUpdate: "",
              modelValue: layer.value.minorDivisions,
              "onUpdate:modelValue": _cache[32] || (_cache[32] = ($event: any) => ((layer.value.minorDivisions) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.polkadotSubDotDivisions), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.minorRadius,
              "onUpdate:modelValue": _cache[33] || (_cache[33] = ($event: any) => ((layer.value.minorRadius) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.polkadotSubDotRadius), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.minorOpacity,
              "onUpdate:modelValue": _cache[34] || (_cache[34] = ($event: any) => ((layer.value.minorOpacity) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.polkadotSubDotOpacity), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : (layer.value.type === 'checker') ? (_openBlock(), _createElementBlock(_Fragment, { key: 5 }, [ _createVNode(MkRange, {
              min: -1,
              max: 1,
              step: 0.01,
              continuousUpdate: "",
              modelValue: layer.value.angle,
              "onUpdate:modelValue": _cache[35] || (_cache[35] = ($event: any) => ((layer.value.angle) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.angle), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 10,
              step: 0.01,
              continuousUpdate: "",
              modelValue: layer.value.scale,
              "onUpdate:modelValue": _cache[36] || (_cache[36] = ($event: any) => ((layer.value.scale) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.scale), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkRange, {
              min: 0,
              max: 1,
              step: 0.01,
              textConverter: (v) => (v * 100).toFixed(1) + '%',
              continuousUpdate: "",
              modelValue: layer.value.opacity,
              "onUpdate:modelValue": _cache[37] || (_cache[37] = ($event: any) => ((layer.value.opacity) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._watermarkEditor.opacity), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
