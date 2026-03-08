import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-upload" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-reload" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-right" })
import { computed, onMounted, ref, useTemplateRef, watch } from 'vue'
import * as Misskey from 'misskey-js'
import type { UploaderFeatures, UploaderItem } from '@/composables/use-uploader.js'
import MkModalWindow from '@/components/MkModalWindow.vue'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { ensureSignin } from '@/i.js'
import { useUploader } from '@/composables/use-uploader.js'
import MkUploaderItems from '@/components/MkUploaderItems.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUploaderDialog',
  props: {
    files: { type: Array, required: true },
    folderId: { type: String, required: false },
    multiple: { type: Boolean, required: false, default: true },
    features: { type: null, required: false }
  },
  emits: ["done", "canceled", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const $i = ensureSignin();
const dialog = useTemplateRef('dialog');
const uploader = useUploader({
	multiple: props.multiple,
	folderId: props.folderId,
	features: props.features,
});
onMounted(() => {
	uploader.addFiles(props.files);
});
const items = uploader.items;
const firstUploadAttempted = ref(false);
const canRetry = computed(() => firstUploadAttempted.value && uploader.readyForUpload.value);
const canDone = computed(() => items.value.some(item => item.uploaded != null));
const overallProgress = computed(() => {
	const max = items.value.length;
	if (max === 0) return 0;
	const v = items.value.reduce((acc, item) => {
		if (item.uploaded) return acc + 1;
		if (item.progress) return acc + (item.progress.value / item.progress.max);
		return acc;
	}, 0);
	return Math.round((v / max) * 100);
});
watch(items, () => {
	if (items.value.length === 0) {
		emit('canceled');
		dialog.value?.close();
		return;
	}
	if (items.value.every(item => item.uploaded)) {
		emit('done', items.value.map(item => item.uploaded!));
		dialog.value?.close();
	}
}, { deep: true });
async function cancel() {
	const { canceled } = await os.confirm({
		type: 'question',
		text: i18n.ts._uploader.abortConfirm,
		okText: i18n.ts.yes,
		cancelText: i18n.ts.no,
	});
	if (canceled) return;
	uploader.abortAll();
	emit('canceled');
	dialog.value?.close();
}
function upload() {
	firstUploadAttempted.value = true;
	uploader.upload();
}
async function abortWithConfirm() {
	const { canceled } = await os.confirm({
		type: 'question',
		text: i18n.ts._uploader.abortConfirm,
		okText: i18n.ts.yes,
		cancelText: i18n.ts.no,
	});
	if (canceled) return;
	uploader.abortAll();
}
async function done() {
	if (!uploader.allItemsUploaded.value) {
		const { canceled } = await os.confirm({
			type: 'question',
			text: i18n.ts._uploader.doneConfirm,
			okText: i18n.ts.yes,
			cancelText: i18n.ts.no,
		});
		if (canceled) return;
	}
	emit('done', items.value.filter(item => item.uploaded != null).map(item => item.uploaded!));
	dialog.value?.close();
}
async function chooseFile(ev: PointerEvent) {
	const newFiles = await os.chooseFileFromPc({ multiple: true });
	uploader.addFiles(newFiles);
}
function showPerItemMenu(item: UploaderItem, ev: PointerEvent) {
	const menu = uploader.getMenu(item);
	os.popupMenu(menu, ev.currentTarget ?? ev.target);
}
function showPerItemMenuViaContextmenu(item: UploaderItem, ev: PointerEvent) {
	const menu = uploader.getMenu(item);
	os.contextMenu(menu, ev);
}

return (_ctx: any,_cache: any) => {
  const _component_MkTip = _resolveComponent("MkTip")

  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialog", ref: dialog,
      width: 800,
      height: 500,
      onClose: _cache[0] || (_cache[0] = ($event: any) => (cancel())),
      onClosed: _cache[1] || (_cache[1] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createTextVNode(" "),
        _createTextVNode(_toDisplayString(_unref(i18n).tsx.uploadNFiles({ n: __props.files.length })), 1 /* TEXT */)
      ]),
      footer: _withCtx(() => [
        _createElementVNode("div", { class: "_buttonsCenter" }, [
          (_unref(uploader).uploading.value)
            ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              rounded: "",
              onClick: _cache[2] || (_cache[2] = ($event: any) => (abortWithConfirm()))
            }, {
              default: _withCtx(() => [
                _hoisted_3,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.abort), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : (!firstUploadAttempted.value)
              ? (_openBlock(), _createBlock(MkButton, {
                key: 1,
                primary: "",
                rounded: "",
                disabled: !_unref(uploader).readyForUpload.value,
                onClick: _cache[3] || (_cache[3] = ($event: any) => (upload()))
              }, {
                default: _withCtx(() => [
                  _hoisted_4,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.upload), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["disabled"]))
            : _createCommentVNode("v-if", true),
          (canRetry.value)
            ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              rounded: "",
              onClick: _cache[4] || (_cache[4] = ($event: any) => (upload()))
            }, {
              default: _withCtx(() => [
                _hoisted_5,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.retry), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : _createCommentVNode("v-if", true),
          (canDone.value)
            ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              rounded: "",
              onClick: _cache[5] || (_cache[5] = ($event: any) => (done()))
            }, {
              default: _withCtx(() => [
                _hoisted_6,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.done), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          _createElementVNode("div", {
            class: _normalizeClass([_ctx.$style.overallProgress, canRetry.value ? _ctx.$style.overallProgressError : null]),
            style: _normalizeStyle({ '--op': `${overallProgress.value}%` })
          }, null, 6 /* CLASS, STYLE */),
          _createElementVNode("div", { class: "_gaps_s _spacer" }, [
            _createVNode(_component_MkTip, { k: "uploader" }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._uploader.tip), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkUploaderItems, {
              items: _unref(items),
              onShowMenu: _cache[6] || (_cache[6] = (item, ev) => showPerItemMenu(item, ev)),
              onShowMenuViaContextmenu: _cache[7] || (_cache[7] = (item, ev) => showPerItemMenuViaContextmenu(item, ev))
            }, null, 8 /* PROPS */, ["items"]),
            (props.multiple)
              ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                _createVNode(MkButton, {
                  style: "margin: auto;",
                  iconOnly: true,
                  rounded: "",
                  onClick: _cache[8] || (_cache[8] = ($event: any) => (chooseFile($event)))
                }, {
                  default: _withCtx(() => [
                    _hoisted_2
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["iconOnly"])
              ]))
              : _createCommentVNode("v-if", true),
            _createElementVNode("div", null, _toDisplayString(_unref(i18n).tsx._uploader.maxFileSizeIsX({ x: _unref($i).policies.maxFileSizeMb + 'MB' })), 1 /* TEXT */)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "height"]))
}
}

})
