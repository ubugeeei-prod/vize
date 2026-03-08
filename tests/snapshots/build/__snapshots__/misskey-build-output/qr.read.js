import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-photo-plus" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-play" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-pause" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-camera-rotate" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bolt" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bolt-off" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bolt-filled" })
import QrScanner from 'qr-scanner'
import { onActivated, onDeactivated, onMounted, onUnmounted, ref, shallowRef, useTemplateRef, watch } from 'vue'
import * as misskey from 'misskey-js'
import { getScrollContainer } from '@@/js/scroll.js'
import type { ApShowResponse } from 'misskey-js/entities.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import MkUserInfo from '@/components/MkUserInfo.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkNote from '@/components/MkNote.vue'
import MkTab from '@/components/MkTab.vue'
import MkButton from '@/components/MkButton.vue'
import MkQrReadRawViewer from '@/pages/qr.read.raw-viewer.vue'
const LIST_RERENDER_INTERVAL = 1500;

export default /*@__PURE__*/_defineComponent({
  __name: 'qr.read',
  setup(__props) {

const rootEl = useTemplateRef('rootEl');
const videoEl = useTemplateRef('videoEl');
const overlayEl = useTemplateRef('overlayEl');
const scannerInstance = shallowRef<QrScanner | null>(null);
const tab = ref<'users' | 'notes' | 'all'>('users');
// higher is recent
const results = ref(new Set<string>());
// lower is recent
const uris = ref<string[]>([]);
const sources = new Map<string, ApShowResponse | null>();
const users = ref<(misskey.entities.UserDetailed)[]>([]);
const usersCount = ref(0);
const notes = ref<misskey.entities.Note[]>([]);
const notesCount = ref(0);
const timer = ref<number | null>(null);
function updateLists() {
	const responses = uris.value.map(uri => sources.get(uri)).filter((r): r is ApShowResponse => !!r);
	users.value = responses.filter(r => r.type === 'User').map(r => r.object).filter((u): u is misskey.entities.UserDetailed => !!u);
	usersCount.value = users.value.length;
	notes.value = responses.filter(r => r.type === 'Note').map(r => r.object).filter((n): n is misskey.entities.Note => !!n);
	notesCount.value = notes.value.length;
	updateRequired.value = false;
}
const updateRequired = ref(false);
watch(uris, () => {
	if (timer.value) {
		updateRequired.value = true;
		return;
	}
	updateLists();
	timer.value = window.setTimeout(() => {
		timer.value = null;
		if (updateRequired.value) {
			updateLists();
		}
	}, LIST_RERENDER_INTERVAL) as number;
});
watch(tab, () => {
	if (timer.value) {
		window.clearTimeout(timer.value);
		timer.value = null;
	}
	updateLists();
});
async function processResult(result: QrScanner.ScanResult) {
	if (!result) return;
	const trimmed = result.data.trim();
	if (!trimmed) return;
	const haveExisted = results.value.has(trimmed);
	results.value.add(trimmed);
	try {
		new URL(trimmed);
	} catch {
		if (!haveExisted) {
			tab.value = 'all';
		}
		return;
	}
	if (uris.value[0] !== trimmed) {
		// 並べ替え
		uris.value = [trimmed, ...uris.value.slice(0, 29).filter(u => u !== trimmed)];
	}
	if (sources.has(trimmed)) return;
	// Start fetching user info
	sources.set(trimmed, null);
	await misskeyApi('ap/show', { uri: trimmed })
		.then(data => {
			if (data.type === 'User') {
				sources.set(trimmed, data);
				tab.value = 'users';
			} else if (data.type === 'Note') {
				sources.set(trimmed, data);
				tab.value = 'notes';
			}
			updateLists();
		})
		.catch(err => {
			tab.value = 'all';
			throw err;
		});
}
const qrStarted = ref(true);
const flashCanToggle = ref(false);
const flash = ref(false);
async function upload() {
	os.chooseFileFromPc({ multiple: true }).then(files => {
		if (files.length === 0) return;
		for (const file of files) {
			QrScanner.scanImage(file, { returnDetailedScanResult: true })
				.then(result => {
					processResult(result);
				})
				.catch(err => {
					if (err.toString().includes('No QR code found')) {
						os.alert({
							type: 'info',
							text: i18n.ts._qr.noQrCodeFound,
						});
					} else {
						os.alert({
							type: 'error',
							text: err.toString(),
						});
						console.error(err);
					}
				});
		}
	});
}
async function chooseCamera() {
	if (!scannerInstance.value) return;
	const cameras = await QrScanner.listCameras(true);
	if (cameras.length === 0) {
		os.alert({
			type: 'error',
		});
		return;
	}
	const select = await os.select({
		title: i18n.ts._qr.chooseCamera,
		items: cameras.map(camera => ({
			label: camera.label,
			value: camera.id,
		})),
	});
	if (select.canceled) return;
	if (select.result == null) return;
	await scannerInstance.value.setCamera(select.result);
	flashCanToggle.value = await scannerInstance.value.hasFlash();
	flash.value = scannerInstance.value.isFlashOn();
}
async function toggleFlash(to = false) {
	if (!scannerInstance.value) return;
	flash.value = to;
	if (flash.value) {
		await scannerInstance.value.turnFlashOn();
	} else {
		await scannerInstance.value.turnFlashOff();
	}
}
async function startQr() {
	if (!scannerInstance.value) return;
	await scannerInstance.value.start();
	qrStarted.value = true;
}
function stopQr() {
	if (!scannerInstance.value) return;
	scannerInstance.value.stop();
	qrStarted.value = false;
}
onActivated(() => {
	startQr;
});
onDeactivated(() => {
	stopQr;
});
const alertLock = ref(false);
onMounted(() => {
	if (!videoEl.value || !overlayEl.value) {
		os.alert({
			type: 'error',
			text: i18n.ts.somethingHappened,
		});
		return;
	}
	scannerInstance.value = new QrScanner(
		videoEl.value,
		processResult,
		{
			highlightScanRegion: true,
			highlightCodeOutline: true,
			overlay: overlayEl.value,
			calculateScanRegion(video: HTMLVideoElement): QrScanner.ScanRegion {
				const aspectRatio = video.videoWidth / video.videoHeight;
				const SHORT_SIDE_SIZE_DOWNSCALED = 360;
				return {
					x: 0,
					y: 0,
					width: video.videoWidth,
					height: video.videoHeight,
					downScaledWidth: aspectRatio > 1 ? Math.round(SHORT_SIDE_SIZE_DOWNSCALED * aspectRatio) : SHORT_SIDE_SIZE_DOWNSCALED,
					downScaledHeight: aspectRatio > 1 ? SHORT_SIDE_SIZE_DOWNSCALED : Math.round(SHORT_SIDE_SIZE_DOWNSCALED / aspectRatio),
				};
			},
			onDecodeError(err) {
				if (err.toString().includes('No QR code found')) return;
				if (alertLock.value) return;
				alertLock.value = true;
				os.alert({
					type: 'error',
					text: err.toString(),
				}).finally(() => {
					alertLock.value = false;
				});
			},
		},
	);
	scannerInstance.value.start()
		.then(async () => {
			qrStarted.value = true;
			if (!scannerInstance.value) return;
			flashCanToggle.value = await scannerInstance.value.hasFlash();
			flash.value = scannerInstance.value.isFlashOn();
		})
		.catch(err => {
			qrStarted.value = false;
			os.alert({
				type: 'error',
				text: err.toString(),
			});
			console.error(err);
		});
});
onUnmounted(() => {
	if (timer.value) {
		window.clearTimeout(timer.value);
		timer.value = null;
	}
	scannerInstance.value?.destroy();
});

return (_ctx: any,_cache: any) => {
  const _component_MkStickyContainer = _resolveComponent("MkStickyContainer")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", {
      ref_key: "rootEl", ref: rootEl,
      class: _normalizeClass(_ctx.$style.root),
      style: {
  		'--MI-QrReadViewHeight': 'calc(100cqh - var(--MI-stickyTop, 0px) - var(--MI-stickyBottom, 0px))',
  		'--MI-QrReadVideoHeight': 'min(calc(var(--MI-QrReadViewHeight) * 0.3), 512px)',
  	}
    }, [ _createVNode(_component_MkStickyContainer, null, {
        header: _withCtx(() => [
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.view)
          }, [
            _createElementVNode("video", {
              ref_key: "videoEl", ref: videoEl,
              class: _normalizeClass(_ctx.$style.video),
              autoplay: "",
              muted: "",
              playsinline: ""
            }, null, 512 /* NEED_PATCH */),
            _createElementVNode("div", { ref_key: "overlayEl", ref: overlayEl }, null, 512 /* NEED_PATCH */),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.controls)
            }, [
              _createVNode(MkButton, {
                iconOnly: "",
                onClick: upload
              }, {
                default: _withCtx(() => [
                  _hoisted_1
                ]),
                _: 1 /* STABLE */
              }),
              (qrStarted.value)
                ? (_openBlock(), _createBlock(MkButton, {
                  key: 0,
                  iconOnly: "",
                  onClick: stopQr
                }, {
                  default: _withCtx(() => [
                    _hoisted_2
                  ]),
                  _: 1 /* STABLE */
                }))
                : (_openBlock(), _createBlock(MkButton, {
                  key: 1,
                  iconOnly: "",
                  danger: "",
                  onClick: startQr
                }, {
                  default: _withCtx(() => [
                    _hoisted_3
                  ]),
                  _: 1 /* STABLE */
                })),
              _createVNode(MkButton, {
                iconOnly: "",
                onClick: chooseCamera
              }, {
                default: _withCtx(() => [
                  _hoisted_4
                ]),
                _: 1 /* STABLE */
              }),
              (!flashCanToggle.value)
                ? (_openBlock(), _createBlock(MkButton, {
                  key: 0,
                  iconOnly: "",
                  disabled: ""
                }, {
                  default: _withCtx(() => [
                    _hoisted_5
                  ]),
                  _: 1 /* STABLE */
                }))
                : (!flash.value)
                  ? (_openBlock(), _createBlock(MkButton, {
                    key: 1,
                    iconOnly: "",
                    onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleFlash(true)))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_6
                    ]),
                    _: 1 /* STABLE */
                  }))
                : (_openBlock(), _createBlock(MkButton, {
                  key: 2,
                  iconOnly: "",
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (toggleFlash(false)))
                }, {
                  default: _withCtx(() => [
                    _hoisted_7
                  ]),
                  _: 1 /* STABLE */
                }))
            ])
          ])
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", {
            class: _normalizeClass(['_spacer', _ctx.$style.contents]),
            style: {
  				'--MI_SPACER-w': '800px'
  			}
          }, [
            _createVNode(_component_MkStickyContainer, null, {
              header: _withCtx(() => [
                _createVNode(MkTab, {
                  tabs: [
  							{ key: 'users', label: _unref(i18n).ts.users },
  							{ key: 'notes', label: _unref(i18n).ts.notes },
  							{ key: 'all', label: _unref(i18n).ts.all },
  						],
                  class: _normalizeClass(_ctx.$style.tab),
                  modelValue: tab.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((tab).value = $event))
                }, null, 8 /* PROPS */, ["tabs", "modelValue"])
              ]),
              default: _withCtx(() => [
                (tab.value === 'users')
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass([_ctx.$style.users, '_margin']),
                    style: "padding-bottom: var(--MI-margin);"
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(users.value, (user) => {
                      return (_openBlock(), _createBlock(MkUserInfo, {
                        key: user.id,
                        user: user
                      }, null, 8 /* PROPS */, ["user"]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]))
                  : (tab.value === 'notes')
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 1,
                      class: "_margin _gaps",
                      style: "padding-bottom: var(--MI-margin);"
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(notes.value, (note) => {
                        return (_openBlock(), _createBlock(MkNote, {
                          key: note.id,
                          note: note,
                          class: _normalizeClass(_ctx.$style.note)
                        }, null, 8 /* PROPS */, ["note"]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]))
                  : (tab.value === 'all')
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 2,
                      class: "_margin _gaps",
                      style: "padding-bottom: var(--MI-margin);"
                    }, [
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(Array.from(results.value).reverse(), (result) => {
                        return (_openBlock(), _createBlock(MkQrReadRawViewer, {
                          key: result,
                          data: result
                        }, null, 8 /* PROPS */, ["data"]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ]))
                  : _createCommentVNode("v-if", true)
              ]),
              _: 1 /* STABLE */
            })
          ])
        ]),
        _: 1 /* STABLE */
      }) ], 512 /* NEED_PATCH */))
}
}

})
