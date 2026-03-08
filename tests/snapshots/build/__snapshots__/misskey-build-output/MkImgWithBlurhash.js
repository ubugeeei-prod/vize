import { defineComponent as _defineComponent } from 'vue'
import { TransitionGroup as _TransitionGroup, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, withDirectives as _withDirectives, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue"

import { computed, nextTick, onMounted, onUnmounted, useTemplateRef, watch, ref } from 'vue'
import { genId } from '@/utility/id.js'
import { render } from 'buraha'
import { prefer } from '@/preferences.js'

import DrawBlurhash from '@/workers/draw-blurhash?worker';
import TestWebGL2 from '@/workers/test-webgl2?worker';
import { WorkerMultiDispatch } from '@@/js/worker-multi-dispatch.js';
import { extractAvgColorFromBlurhash } from '@@/js/extract-avg-color-from-blurhash.js';

// テスト環境で Web Worker インスタンスは作成できない
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-expect-error
const isTest = (import.meta.env.MODE === 'test' || window.Cypress != null);

const canvasPromise = new Promise<WorkerMultiDispatch | HTMLCanvasElement>(resolve => {
	if (isTest) {
		const canvas = window.document.createElement('canvas');
		canvas.width = 64;
		canvas.height = 64;
		resolve(canvas);
		return;
	}

	const testWorker = new TestWebGL2();
	testWorker.addEventListener('message', event => {
		if (event.data.result) {
			const workers = new WorkerMultiDispatch(
				() => new DrawBlurhash(),
				Math.min(navigator.hardwareConcurrency - 1, 4),
			);
			resolve(workers);
		} else {
			const canvas = window.document.createElement('canvas');
			canvas.width = 64;
			canvas.height = 64;
			resolve(canvas);
		}
		testWorker.terminate();
	});
});

export default /*@__PURE__*/_defineComponent({
  __name: 'MkImgWithBlurhash',
  props: {
    transition: { type: Object, required: false, default: null },
    src: { type: String, required: false, default: null },
    hash: { type: String, required: false },
    alt: { type: String, required: false, default: '' },
    title: { type: String, required: false, default: null },
    height: { type: Number, required: false, default: 64 },
    width: { type: Number, required: false, default: 64 },
    cover: { type: Boolean, required: false, default: true },
    forceBlurhash: { type: Boolean, required: false, default: false },
    onlyAvgColor: { type: Boolean, required: false, default: false }
  },
  setup(__props: any) {

const props = __props
const viewId = genId();
const canvas = useTemplateRef('canvas');
const root = useTemplateRef('root');
const img = useTemplateRef('img');
const loaded = ref(false);
const canvasWidth = ref(64);
const canvasHeight = ref(64);
const imgWidth = ref(props.width);
const imgHeight = ref(props.height);
const bitmapTmp = ref<CanvasImageSource | undefined>();
const hide = computed(() => !loaded.value || props.forceBlurhash);
function waitForDecode() {
	if (props.src != null && props.src !== '') {
		nextTick()
			.then(() => img.value?.decode())
			.then(() => {
				loaded.value = true;
			}, error => {
				console.log('Error occurred during decoding image', img.value, error);
			});
	} else {
		loaded.value = false;
	}
}
watch([() => props.width, () => props.height, root], () => {
	const ratio = props.width / props.height;
	if (ratio > 1) {
		canvasWidth.value = Math.round(64 * ratio);
		canvasHeight.value = 64;
	} else {
		canvasWidth.value = 64;
		canvasHeight.value = Math.round(64 / ratio);
	}
	const clientWidth = root.value?.clientWidth ?? 300;
	imgWidth.value = clientWidth;
	imgHeight.value = Math.round(clientWidth / ratio);
}, {
	immediate: true,
});
function drawImage(bitmap: CanvasImageSource) {
	// canvasがない（mountedされていない）場合はTmpに保存しておく
	if (!canvas.value) {
		bitmapTmp.value = bitmap;
		return;
	}
	// canvasがあれば描画する
	bitmapTmp.value = undefined;
	const ctx = canvas.value.getContext('2d');
	if (!ctx) return;
	ctx.drawImage(bitmap, 0, 0, canvasWidth.value, canvasHeight.value);
}
function drawAvg() {
	if (!canvas.value) return;
	const color = (props.hash != null && extractAvgColorFromBlurhash(props.hash)) || '#888';
	const ctx = canvas.value.getContext('2d');
	if (!ctx) return;
	// avgColorでお茶をにごす
	ctx.beginPath();
	ctx.fillStyle = color;
	ctx.fillRect(0, 0, canvasWidth.value, canvasHeight.value);
}
async function draw() {
	if (isTest && props.hash == null) return;
	drawAvg();
	if (props.hash == null) return;
	if (props.onlyAvgColor) return;
	const work = await canvasPromise;
	if (work instanceof WorkerMultiDispatch) {
		work.postMessage(
			{
				id: viewId,
				hash: props.hash,
			},
			undefined,
		);
	} else {
		try {
			render(props.hash, work);
			drawImage(work);
		} catch (error) {
			console.error('Error occurred during drawing blurhash', error);
		}
	}
}
function workerOnMessage(event: MessageEvent) {
	if (event.data.id !== viewId) return;
	drawImage(event.data.bitmap as ImageBitmap);
}
canvasPromise.then(work => {
	if (work instanceof WorkerMultiDispatch) {
		work.addListener(workerOnMessage);
	}
	draw();
});
watch(() => props.src, () => {
	waitForDecode();
});
watch(() => props.hash, () => {
	draw();
});
onMounted(() => {
	// drawImageがmountedより先に呼ばれている場合はここで描画する
	if (bitmapTmp.value) {
		drawImage(bitmapTmp.value);
	}
	waitForDecode();
});
onUnmounted(() => {
	canvasPromise.then(work => {
		if (work instanceof WorkerMultiDispatch) {
			work.removeListener(workerOnMessage);
		}
	});
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      ref_key: "root", ref: root,
      class: _normalizeClass(['chromatic-ignore', _ctx.$style.root, { [_ctx.$style.cover]: __props.cover }]),
      title: __props.title ?? ''
    }, [ _createVNode(_TransitionGroup, {
        duration: _unref(prefer).s.animation && props.transition?.duration || undefined,
        enterActiveClass: _unref(prefer).s.animation && props.transition?.enterActiveClass || undefined,
        leaveActiveClass: _unref(prefer).s.animation && (props.transition?.leaveActiveClass ?? _ctx.$style.transition_leaveActive) || undefined,
        enterFromClass: _unref(prefer).s.animation && props.transition?.enterFromClass || undefined,
        leaveToClass: _unref(prefer).s.animation && props.transition?.leaveToClass || undefined,
        enterToClass: _unref(prefer).s.animation && props.transition?.enterToClass || undefined,
        leaveFromClass: _unref(prefer).s.animation && props.transition?.leaveFromClass || undefined
      }, {
        default: _withCtx(() => [
          _withDirectives(_createElementVNode("canvas", {
            key: "canvas",
            ref_key: "canvas", ref: canvas,
            class: _normalizeClass(_ctx.$style.canvas),
            width: canvasWidth.value,
            height: canvasHeight.value,
            title: __props.title ?? undefined,
            draggable: "false",
            tabindex: "-1",
            style: "-webkit-user-drag: none;"
          }, null, 8 /* PROPS */, ["width", "height", "title"]), [
            [_vShow, hide.value]
          ]),
          _withDirectives(_createElementVNode("img", {
            key: "img",
            ref_key: "img", ref: img,
            height: imgHeight.value ?? undefined,
            width: imgWidth.value ?? undefined,
            class: _normalizeClass(_ctx.$style.img),
            src: __props.src ?? undefined,
            title: __props.title ?? undefined,
            alt: __props.alt ?? undefined,
            loading: "eager",
            decoding: "async",
            draggable: "false",
            tabindex: "-1",
            style: "-webkit-user-drag: none;"
          }, null, 8 /* PROPS */, ["height", "width", "src", "title", "alt"]), [
            [_vShow, !hide.value]
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["duration", "enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "enterToClass", "leaveFromClass"]) ], 10 /* CLASS, PROPS */, ["title"]))
}
}

})
