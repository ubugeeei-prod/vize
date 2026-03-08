import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-music" })
const _hoisted_3 = { style: "display: block;" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
import { useTemplateRef, watch, computed, ref, onDeactivated, onActivated, onMounted } from 'vue'
import * as Misskey from 'misskey-js'
import type { MenuItem } from '@/types/menu.js'
import type { Keymap } from '@/utility/hotkey.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import bytes from '@/filters/bytes.js'
import { hms } from '@/filters/hms.js'
import MkMediaRange from '@/components/MkMediaRange.vue'
import { $i, iAmModerator } from '@/i.js'
import { prefer } from '@/preferences.js'
import { canRevealFile, shouldHideFileByDefault } from '@/utility/sensitive-file.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMediaAudio',
  props: {
    audio: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const keymap = {
	'up': {
		allowRepeat: true,
		callback: () => {
			if (hasFocus() && audioEl.value) {
				volume.value = Math.min(volume.value + 0.1, 1);
			}
		},
	},
	'down': {
		allowRepeat: true,
		callback: () => {
			if (hasFocus() && audioEl.value) {
				volume.value = Math.max(volume.value - 0.1, 0);
			}
		},
	},
	'left': {
		allowRepeat: true,
		callback: () => {
			if (hasFocus() && audioEl.value) {
				audioEl.value.currentTime = Math.max(audioEl.value.currentTime - 5, 0);
			}
		},
	},
	'right': {
		allowRepeat: true,
		callback: () => {
			if (hasFocus() && audioEl.value) {
				audioEl.value.currentTime = Math.min(audioEl.value.currentTime + 5, audioEl.value.duration);
			}
		},
	},
	'space': () => {
		if (hasFocus()) {
			togglePlayPause();
		}
	},
} as const satisfies Keymap;
// PlayerElもしくはその子要素にフォーカスがあるかどうか
function hasFocus() {
	if (!playerEl.value) return false;
	return playerEl.value === window.document.activeElement || playerEl.value.contains(window.document.activeElement);
}
const playerEl = useTemplateRef('playerEl');
const audioEl = useTemplateRef('audioEl');
const hide = ref(shouldHideFileByDefault(props.audio));
async function reveal() {
	if (!(await canRevealFile(props.audio))) {
		return;
	}
	hide.value = false;
}
// Menu
const menuShowing = ref(false);
function showMenu(ev: MouseEvent) {
	const menu: MenuItem[] = [
		// TODO: 再生キューに追加
		{
			type: 'switch',
			text: i18n.ts._mediaControls.loop,
			icon: 'ti ti-repeat',
			ref: loop,
		},
		{
			type: 'radio',
			text: i18n.ts._mediaControls.playbackRate,
			icon: 'ti ti-clock-play',
			ref: speed,
			options: {
				'0.25x': 0.25,
				'0.5x': 0.5,
				'0.75x': 0.75,
				'1.0x': 1,
				'1.25x': 1.25,
				'1.5x': 1.5,
				'2.0x': 2,
			},
		},
		{
			type: 'divider',
		},
		{
			text: i18n.ts.hide,
			icon: 'ti ti-eye-off',
			action: () => {
				hide.value = true;
			},
		},
	];
	if (iAmModerator) {
		menu.push({
			text: props.audio.isSensitive ? i18n.ts.unmarkAsSensitive : i18n.ts.markAsSensitive,
			icon: props.audio.isSensitive ? 'ti ti-eye' : 'ti ti-eye-exclamation',
			danger: true,
			action: () => toggleSensitive(props.audio),
		});
	}
	const details: MenuItem[] = [];
	if ($i?.id === props.audio.userId) {
		details.push({
			type: 'link',
			text: i18n.ts._fileViewer.title,
			icon: 'ti ti-info-circle',
			to: `/my/drive/file/${props.audio.id}`,
		});
	}
	if (iAmModerator) {
		details.push({
			type: 'link',
			text: i18n.ts.moderation,
			icon: 'ti ti-photo-exclamation',
			to: `/admin/file/${props.audio.id}`,
		});
	}
	if (details.length > 0) {
		menu.push({ type: 'divider' }, ...details);
	}
	if (prefer.s.devMode) {
		menu.push({ type: 'divider' }, {
			icon: 'ti ti-hash',
			text: i18n.ts.copyFileId,
			action: () => {
				copyToClipboard(props.audio.id);
			},
		});
	}
	menuShowing.value = true;
	os.popupMenu(menu, ev.currentTarget ?? ev.target, {
		align: 'right',
		onClosing: () => {
			menuShowing.value = false;
		},
	});
}
async function toggleSensitive(file: Misskey.entities.DriveFile) {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: file.isSensitive ? i18n.ts.unmarkAsSensitiveConfirm : i18n.ts.markAsSensitiveConfirm,
	});
	if (canceled) return;
	os.apiWithDialog('drive/files/update', {
		fileId: file.id,
		isSensitive: !file.isSensitive,
	});
}
// MediaControl: Common State
const oncePlayed = ref(false);
const isReady = ref(false);
const isPlaying = ref(false);
const isActuallyPlaying = ref(false);
const elapsedTimeMs = ref(0);
const durationMs = ref(0);
const rangePercent = computed({
	get: () => {
		return (elapsedTimeMs.value / durationMs.value) || 0;
	},
	set: (to) => {
		if (!audioEl.value) return;
		audioEl.value.currentTime = to * durationMs.value / 1000;
	},
});
const volume = ref(.25);
const speed = ref(1);
const loop = ref(false); // TODO: ドライブファイルのフラグに置き換える
const bufferedEnd = ref(0);
const bufferedDataRatio = computed(() => {
	if (!audioEl.value) return 0;
	return bufferedEnd.value / audioEl.value.duration;
});
// MediaControl Events
function togglePlayPause() {
	if (!isReady.value || !audioEl.value) return;
	if (isPlaying.value) {
		audioEl.value.pause();
		isPlaying.value = false;
	} else {
		audioEl.value.play();
		isPlaying.value = true;
		oncePlayed.value = true;
	}
}
function toggleMute() {
	if (volume.value === 0) {
		volume.value = .25;
	} else {
		volume.value = 0;
	}
}
let onceInit = false;
let mediaTickFrameId: number | null = null;
let stopAudioElWatch: () => void;
function init() {
	if (onceInit) return;
	onceInit = true;
	stopAudioElWatch = watch(audioEl, () => {
		if (audioEl.value) {
			isReady.value = true;
			function updateMediaTick() {
				if (audioEl.value) {
					try {
						bufferedEnd.value = audioEl.value.buffered.end(0);
					} catch (err) {
						bufferedEnd.value = 0;
					}
					elapsedTimeMs.value = audioEl.value.currentTime * 1000;
					if (audioEl.value.loop !== loop.value) {
						loop.value = audioEl.value.loop;
					}
				}
				mediaTickFrameId = window.requestAnimationFrame(updateMediaTick);
			}
			updateMediaTick();
			audioEl.value.addEventListener('play', () => {
				isActuallyPlaying.value = true;
			});
			audioEl.value.addEventListener('pause', () => {
				isActuallyPlaying.value = false;
				isPlaying.value = false;
			});
			audioEl.value.addEventListener('ended', () => {
				oncePlayed.value = false;
				isActuallyPlaying.value = false;
				isPlaying.value = false;
			});
			durationMs.value = audioEl.value.duration * 1000;
			audioEl.value.addEventListener('durationchange', () => {
				if (audioEl.value) {
					durationMs.value = audioEl.value.duration * 1000;
				}
			});
			audioEl.value.volume = volume.value;
		}
	}, {
		immediate: true,
	});
}
watch(volume, (to) => {
	if (audioEl.value) audioEl.value.volume = to;
});
watch(speed, (to) => {
	if (audioEl.value) audioEl.value.playbackRate = to;
});
watch(loop, (to) => {
	if (audioEl.value) audioEl.value.loop = to;
});
onMounted(() => {
	init();
});
onActivated(() => {
	init();
});
onDeactivated(() => {
	isReady.value = false;
	isPlaying.value = false;
	isActuallyPlaying.value = false;
	elapsedTimeMs.value = 0;
	durationMs.value = 0;
	bufferedEnd.value = 0;
	hide.value = (prefer.s.nsfw === 'force' || prefer.s.dataSaver.media) ? true : (props.audio.isSensitive && prefer.s.nsfw !== 'ignore');
	stopAudioElWatch();
	onceInit = false;
	if (mediaTickFrameId) {
		window.cancelAnimationFrame(mediaTickFrameId);
		mediaTickFrameId = null;
	}
});

return (_ctx: any,_cache: any) => {
  const _directive_hotkey = _resolveDirective("hotkey")

  return _withDirectives((_openBlock(), _createElementBlock("div", {
      ref_key: "playerEl", ref: playerEl,
      tabindex: "0",
      class: _normalizeClass([
  		_ctx.$style.audioContainer,
  		(__props.audio.isSensitive && _unref(prefer).s.highlightSensitiveMedia) && _ctx.$style.sensitive,
  	]),
      onContextmenu: _withModifiers(() => {}, ["stop"]),
      onKeydown: _withModifiers(() => {}, ["stop"])
    }, [ (hide.value) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          class: _normalizeClass(_ctx.$style.hidden),
          onClick: reveal
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.hiddenTextWrapper)
          }, [ (__props.audio.isSensitive) ? (_openBlock(), _createElementBlock("b", {
                key: 0,
                style: "display: block;"
              }, [ _hoisted_1, _createTextVNode(), _toDisplayString(_unref(i18n).ts.sensitive), _toDisplayString(_unref(prefer).s.dataSaver.media ? ` (${_unref(i18n).ts.audio}${__props.audio.size ? ' ' + bytes(__props.audio.size) : ''})` : '') ])) : (_openBlock(), _createElementBlock("b", {
                key: 1,
                style: "display: block;"
              }, [ _hoisted_2, _createTextVNode(), _toDisplayString(_unref(prefer).s.dataSaver.media && __props.audio.size ? bytes(__props.audio.size) : _unref(i18n).ts.audio) ])), _createElementVNode("span", _hoisted_3, _toDisplayString(_unref(i18n).ts.clickToShow), 1 /* TEXT */) ]) ])) : (_unref(prefer).s.useNativeUiForVideoAudioPlayer) ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: _normalizeClass(_ctx.$style.nativeAudioContainer)
          }, [ _createElementVNode("audio", {
              ref_key: "audioEl", ref: audioEl,
              preload: "metadata",
              controls: "",
              class: _normalizeClass(_ctx.$style.nativeAudio),
              onKeydown: _withModifiers(() => {}, ["prevent"])
            }, [ _createElementVNode("source", { src: __props.audio.url }, null, 8 /* PROPS */, ["src"]) ], 40 /* PROPS, NEED_HYDRATION */, ["onKeydown"]) ])) : (_openBlock(), _createElementBlock("div", {
          key: 2,
          class: _normalizeClass(_ctx.$style.audioControls)
        }, [ _createElementVNode("audio", {
            ref_key: "audioEl", ref: audioEl,
            preload: "metadata",
            onKeydown: _cache[0] || (_cache[0] = _withModifiers(() => {}, ["prevent"]))
          }, [ _createElementVNode("source", { src: __props.audio.url }, null, 8 /* PROPS */, ["src"]) ], 32 /* NEED_HYDRATION */), _createElementVNode("div", {
            class: _normalizeClass([_ctx.$style.controlsChild, _ctx.$style.controlsLeft])
          }, [ _createElementVNode("button", {
              class: _normalizeClass(['_button', _ctx.$style.controlButton]),
              tabindex: "-1",
              onClick: _withModifiers(togglePlayPause, ["stop"])
            }, [ (isPlaying.value) ? (_openBlock(), _createElementBlock("i", {
                  key: 0,
                  class: "ti ti-player-pause-filled"
                })) : (_openBlock(), _createElementBlock("i", {
                  key: 1,
                  class: "ti ti-player-play-filled"
                })) ]) ]), _createElementVNode("div", {
            class: _normalizeClass([_ctx.$style.controlsChild, _ctx.$style.controlsRight])
          }, [ _createElementVNode("button", {
              class: _normalizeClass(['_button', _ctx.$style.controlButton]),
              tabindex: "-1",
              onClick: _cache[1] || (_cache[1] = _withModifiers(() => {}, ["stop"])),
              onMousedown: _withModifiers(showMenu, ["prevent","stop"])
            }, [ _hoisted_4 ], 32 /* NEED_HYDRATION */) ]), _createElementVNode("div", {
            class: _normalizeClass([_ctx.$style.controlsChild, _ctx.$style.controlsTime])
          }, _toDisplayString(_unref(hms)(elapsedTimeMs.value)), 1 /* TEXT */), _createElementVNode("div", {
            class: _normalizeClass([_ctx.$style.controlsChild, _ctx.$style.controlsVolume])
          }, [ _createElementVNode("button", {
              class: _normalizeClass(['_button', _ctx.$style.controlButton]),
              tabindex: "-1",
              onClick: _withModifiers(toggleMute, ["stop"])
            }, [ (volume.value === 0) ? (_openBlock(), _createElementBlock("i", {
                  key: 0,
                  class: "ti ti-volume-3"
                })) : (_openBlock(), _createElementBlock("i", {
                  key: 1,
                  class: "ti ti-volume"
                })) ]), _createVNode(MkMediaRange, {
              class: _normalizeClass(_ctx.$style.volumeSeekbar),
              modelValue: volume.value,
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((volume).value = $event))
            }, null, 8 /* PROPS */, ["modelValue"]) ]), _createVNode(MkMediaRange, {
            class: _normalizeClass(_ctx.$style.seekbarRoot),
            buffer: bufferedDataRatio.value,
            modelValue: rangePercent.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((rangePercent).value = $event))
          }, null, 8 /* PROPS */, ["buffer", "modelValue"]) ])) ], 42 /* CLASS, PROPS, NEED_HYDRATION */, ["onContextmenu", "onKeydown"])), [ [_directive_hotkey, _unref(keymap)] ])
}
}

})
