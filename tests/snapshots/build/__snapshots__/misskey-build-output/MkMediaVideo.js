import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-movie" })
const _hoisted_3 = { style: "display: block;" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-play-filled" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye-exclamation" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
import { ref, useTemplateRef, computed, watch, onDeactivated, onActivated, onMounted } from 'vue'
import * as Misskey from 'misskey-js'
import type { MenuItem } from '@/types/menu.js'
import type { Keymap } from '@/utility/hotkey.js'
import { copyToClipboard } from '@/utility/copy-to-clipboard'
import bytes from '@/filters/bytes.js'
import { hms } from '@/filters/hms.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { exitFullscreen, requestFullscreen } from '@/utility/fullscreen.js'
import hasAudio from '@/utility/media-has-audio.js'
import MkMediaRange from '@/components/MkMediaRange.vue'
import { $i, iAmModerator } from '@/i.js'
import { prefer } from '@/preferences.js'
import { shouldHideFileByDefault, canRevealFile } from '@/utility/sensitive-file.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkMediaVideo',
  props: {
    video: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const keymap = {
	'up': {
		allowRepeat: true,
		callback: () => {
			if (hasFocus() && videoEl.value) {
				volume.value = Math.min(volume.value + 0.1, 1);
			}
		},
	},
	'down': {
		allowRepeat: true,
		callback: () => {
			if (hasFocus() && videoEl.value) {
				volume.value = Math.max(volume.value - 0.1, 0);
			}
		},
	},
	'left': {
		allowRepeat: true,
		callback: () => {
			if (hasFocus() && videoEl.value) {
				videoEl.value.currentTime = Math.max(videoEl.value.currentTime - 5, 0);
			}
		},
	},
	'right': {
		allowRepeat: true,
		callback: () => {
			if (hasFocus() && videoEl.value) {
				videoEl.value.currentTime = Math.min(videoEl.value.currentTime + 5, videoEl.value.duration);
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
// eslint-disable-next-line vue/no-setup-props-reactivity-loss
const hide = ref(shouldHideFileByDefault(props.video));
async function reveal() {
	if (!(await canRevealFile(props.video))) {
		return;
	}
	hide.value = false;
}
// Menu
const menuShowing = ref(false);
function showMenu(ev: PointerEvent) {
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
		...(window.document.pictureInPictureEnabled ? [{
			text: i18n.ts._mediaControls.pip,
			icon: 'ti ti-picture-in-picture',
			action: togglePictureInPicture,
		}] : []),
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
			text: props.video.isSensitive ? i18n.ts.unmarkAsSensitive : i18n.ts.markAsSensitive,
			icon: props.video.isSensitive ? 'ti ti-eye' : 'ti ti-eye-exclamation',
			danger: true,
			action: () => toggleSensitive(props.video),
		});
	}
	const details: MenuItem[] = [];
	if ($i?.id === props.video.userId) {
		details.push({
			type: 'link',
			text: i18n.ts._fileViewer.title,
			icon: 'ti ti-info-circle',
			to: `/my/drive/file/${props.video.id}`,
		});
	}
	if (iAmModerator) {
		details.push({
			type: 'link',
			text: i18n.ts.moderation,
			icon: 'ti ti-photo-exclamation',
			to: `/admin/file/${props.video.id}`,
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
				copyToClipboard(props.video.id);
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
// MediaControl: Video State
const videoEl = useTemplateRef('videoEl');
const playerEl = useTemplateRef('playerEl');
const isHoverring = ref(false);
const controlsShowing = computed(() => {
	if (!oncePlayed.value) return true;
	if (isHoverring.value) return true;
	if (menuShowing.value) return true;
	return false;
});
const isFullscreen = ref(false);
let controlStateTimer: number | null = null;
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
		if (!videoEl.value) return;
		videoEl.value.currentTime = to * durationMs.value / 1000;
	},
});
const volume = ref(.25);
const speed = ref(1);
const loop = ref(false); // TODO: ドライブファイルのフラグに置き換える
const bufferedEnd = ref(0);
const bufferedDataRatio = computed(() => {
	if (!videoEl.value) return 0;
	return bufferedEnd.value / videoEl.value.duration;
});
// MediaControl Events
function onMouseOver() {
	if (controlStateTimer) {
		window.clearTimeout(controlStateTimer);
	}
	isHoverring.value = true;
	controlStateTimer = window.setTimeout(() => {
		isHoverring.value = false;
	}, 3000);
}
function onMouseMove() {
	if (controlStateTimer) {
		window.clearTimeout(controlStateTimer);
	}
	isHoverring.value = true;
	controlStateTimer = window.setTimeout(() => {
		isHoverring.value = false;
	}, 3000);
}
function onMouseLeave() {
	if (controlStateTimer) {
		window.clearTimeout(controlStateTimer);
	}
	controlStateTimer = window.setTimeout(() => {
		isHoverring.value = false;
	}, 100);
}
function togglePlayPause() {
	if (!isReady.value || !videoEl.value) return;
	if (isPlaying.value) {
		videoEl.value.pause();
		isPlaying.value = false;
	} else {
		videoEl.value.play();
		isPlaying.value = true;
		oncePlayed.value = true;
	}
}
function toggleFullscreen() {
	if (playerEl.value == null || videoEl.value == null) return;
	if (isFullscreen.value) {
		exitFullscreen({
			videoEl: videoEl.value,
		});
		isFullscreen.value = false;
	} else {
		requestFullscreen({
			videoEl: videoEl.value,
			playerEl: playerEl.value,
			options: {
				navigationUI: 'hide',
			},
		});
		isFullscreen.value = true;
	}
}
function togglePictureInPicture() {
	if (videoEl.value) {
		if (window.document.pictureInPictureElement) {
			window.document.exitPictureInPicture();
		} else {
			videoEl.value.requestPictureInPicture();
		}
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
let stopVideoElWatch: () => void;
function init() {
	if (onceInit) return;
	onceInit = true;
	stopVideoElWatch = watch(videoEl, () => {
		if (videoEl.value) {
			isReady.value = true;
			function updateMediaTick() {
				if (videoEl.value) {
					try {
						bufferedEnd.value = videoEl.value.buffered.end(0);
					} catch (err) {
						bufferedEnd.value = 0;
					}
					elapsedTimeMs.value = videoEl.value.currentTime * 1000;
					if (videoEl.value.loop !== loop.value) {
						loop.value = videoEl.value.loop;
					}
				}
				mediaTickFrameId = window.requestAnimationFrame(updateMediaTick);
			}
			updateMediaTick();
			videoEl.value.addEventListener('play', () => {
				isActuallyPlaying.value = true;
			});
			videoEl.value.addEventListener('pause', () => {
				isActuallyPlaying.value = false;
				isPlaying.value = false;
			});
			videoEl.value.addEventListener('ended', () => {
				oncePlayed.value = false;
				isActuallyPlaying.value = false;
				isPlaying.value = false;
			});
			durationMs.value = videoEl.value.duration * 1000;
			videoEl.value.addEventListener('durationchange', () => {
				if (videoEl.value) {
					durationMs.value = videoEl.value.duration * 1000;
				}
			});
			videoEl.value.volume = volume.value;
			hasAudio(videoEl.value).then(had => {
				if (!had && videoEl.value) {
					videoEl.value.loop = videoEl.value.muted = true;
					videoEl.value.play();
				}
			});
		}
	}, {
		immediate: true,
	});
}
watch(volume, (to) => {
	if (videoEl.value) videoEl.value.volume = to;
});
watch(speed, (to) => {
	if (videoEl.value) videoEl.value.playbackRate = to;
});
watch(loop, (to) => {
	if (videoEl.value) videoEl.value.loop = to;
});
watch(hide, (to) => {
	if (videoEl.value && to && isFullscreen.value) {
		exitFullscreen({
			videoEl: videoEl.value,
		});
		isFullscreen.value = false;
	}
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
	hide.value = (prefer.s.nsfw === 'force' || prefer.s.dataSaver.media) ? true : (props.video.isSensitive && prefer.s.nsfw !== 'ignore');
	stopVideoElWatch();
	onceInit = false;
	if (mediaTickFrameId) {
		window.cancelAnimationFrame(mediaTickFrameId);
		mediaTickFrameId = null;
	}
	if (controlStateTimer) {
		window.clearTimeout(controlStateTimer);
		controlStateTimer = null;
	}
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _directive_hotkey = _resolveDirective("hotkey")

  return _withDirectives((_openBlock(), _createElementBlock("div", {
      ref_key: "playerEl", ref: playerEl,
      tabindex: "0",
      class: _normalizeClass([
  		_ctx.$style.videoContainer,
  		controlsShowing.value && _ctx.$style.active,
  		(__props.video.isSensitive && _unref(prefer).s.highlightSensitiveMedia) && _ctx.$style.sensitive,
  	]),
      onMouseoverPassive: onMouseOver,
      onMousemovePassive: onMouseMove,
      onMouseleavePassive: onMouseLeave,
      onContextmenu: _withModifiers(() => {}, ["stop"]),
      onKeydown: _withModifiers(() => {}, ["stop"])
    }, [ (hide.value) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          class: _normalizeClass(_ctx.$style.hidden),
          onClick: reveal
        }, [ _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.hiddenTextWrapper)
          }, [ (__props.video.isSensitive) ? (_openBlock(), _createElementBlock("b", {
                key: 0,
                style: "display: block;"
              }, [ _hoisted_1, _createTextVNode(), _toDisplayString(_unref(i18n).ts.sensitive), _toDisplayString(_unref(prefer).s.dataSaver.media ? ` (${_unref(i18n).ts.video}${__props.video.size ? ' ' + bytes(__props.video.size) : ''})` : '') ])) : (_openBlock(), _createElementBlock("b", {
                key: 1,
                style: "display: block;"
              }, [ _hoisted_2, _createTextVNode(), _toDisplayString(_unref(prefer).s.dataSaver.media && __props.video.size ? bytes(__props.video.size) : _unref(i18n).ts.video) ])), _createElementVNode("span", _hoisted_3, _toDisplayString(_unref(i18n).ts.clickToShow), 1 /* TEXT */) ]) ])) : (_unref(prefer).s.useNativeUiForVideoAudioPlayer) ? (_openBlock(), _createElementBlock("div", {
            key: 1,
            class: _normalizeClass(_ctx.$style.videoRoot)
          }, [ _createElementVNode("video", {
              ref_key: "videoEl", ref: videoEl,
              class: _normalizeClass(_ctx.$style.video),
              poster: __props.video.thumbnailUrl ?? undefined,
              title: __props.video.comment ?? undefined,
              alt: __props.video.comment,
              preload: "metadata",
              controls: "",
              onKeydown: _withModifiers(() => {}, ["prevent"])
            }, [ _createElementVNode("source", { src: __props.video.url }, null, 8 /* PROPS */, ["src"]) ], 40 /* PROPS, NEED_HYDRATION */, ["poster", "title", "alt", "onKeydown"]), _createElementVNode("i", {
              class: _normalizeClass(["ti ti-eye-off", _ctx.$style.hide]),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (hide.value = true))
            }), _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.indicators)
            }, [ (__props.video.comment) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.indicator)
                }, "ALT")) : _createCommentVNode("v-if", true), (__props.video.isSensitive) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.indicator),
                  style: "color: var(--MI_THEME-warn);",
                  title: _unref(i18n).ts.sensitive
                }, [ _hoisted_4 ])) : _createCommentVNode("v-if", true) ]) ])) : (_openBlock(), _createElementBlock("div", {
          key: 2,
          class: _normalizeClass(_ctx.$style.videoRoot)
        }, [ _createElementVNode("video", {
            ref_key: "videoEl", ref: videoEl,
            class: _normalizeClass(_ctx.$style.video),
            poster: __props.video.thumbnailUrl ?? undefined,
            title: __props.video.comment ?? undefined,
            alt: __props.video.comment,
            preload: "metadata",
            playsinline: "",
            onKeydown: _withModifiers(() => {}, ["prevent"]),
            onClick: _withModifiers(togglePlayPause, ["self"])
          }, [ _createElementVNode("source", { src: __props.video.url }, null, 8 /* PROPS */, ["src"]) ], 40 /* PROPS, NEED_HYDRATION */, ["poster", "title", "alt", "onKeydown"]), (isReady.value && !isPlaying.value) ? (_openBlock(), _createElementBlock("button", {
              key: 0,
              class: _normalizeClass(["_button", _ctx.$style.videoOverlayPlayButton]),
              onClick: togglePlayPause
            }, [ _hoisted_5 ])) : (!isActuallyPlaying.value) ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                class: _normalizeClass(_ctx.$style.videoLoading)
              }, [ _createVNode(_component_MkLoading) ])) : _createCommentVNode("v-if", true), _createElementVNode("i", {
            class: _normalizeClass(["ti ti-eye-off", _ctx.$style.hide]),
            onClick: _cache[1] || (_cache[1] = ($event: any) => (hide.value = true))
          }), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.indicators)
          }, [ (__props.video.comment) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.indicator)
              }, "ALT")) : _createCommentVNode("v-if", true), (__props.video.isSensitive) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.indicator),
                style: "color: var(--MI_THEME-warn);",
                title: _unref(i18n).ts.sensitive
              }, [ _hoisted_6 ])) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.videoControls),
            onClick: _withModifiers(togglePlayPause, ["self"])
          }, [ _createElementVNode("div", {
              class: _normalizeClass([_ctx.$style.controlsChild, _ctx.$style.controlsLeft])
            }, [ _createElementVNode("button", {
                class: _normalizeClass(["_button", _ctx.$style.controlButton]),
                onClick: togglePlayPause
              }, [ (isPlaying.value) ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-player-pause-filled"
                  })) : (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-player-play-filled"
                  })) ]) ]), _createElementVNode("div", {
              class: _normalizeClass([_ctx.$style.controlsChild, _ctx.$style.controlsRight])
            }, [ _createElementVNode("button", {
                class: _normalizeClass(["_button", _ctx.$style.controlButton]),
                onClick: showMenu
              }, [ _hoisted_7 ]), _createElementVNode("button", {
                class: _normalizeClass(["_button", _ctx.$style.controlButton]),
                onClick: toggleFullscreen
              }, [ (isFullscreen.value) ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-arrows-minimize"
                  })) : (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-arrows-maximize"
                  })) ]) ]), _createElementVNode("div", {
              class: _normalizeClass([_ctx.$style.controlsChild, _ctx.$style.controlsTime])
            }, _toDisplayString(_unref(hms)(elapsedTimeMs.value)), 1 /* TEXT */), _createElementVNode("div", {
              class: _normalizeClass([_ctx.$style.controlsChild, _ctx.$style.controlsVolume])
            }, [ _createElementVNode("button", {
                class: _normalizeClass(["_button", _ctx.$style.controlButton]),
                onClick: toggleMute
              }, [ (volume.value === 0) ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-volume-3"
                  })) : (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-volume"
                  })) ]), _createVNode(MkMediaRange, {
                sliderBgWhite: true,
                class: _normalizeClass(_ctx.$style.volumeSeekbar),
                modelValue: volume.value,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((volume).value = $event))
              }, null, 8 /* PROPS */, ["sliderBgWhite", "modelValue"]) ]), _createVNode(MkMediaRange, {
              sliderBgWhite: true,
              class: _normalizeClass(_ctx.$style.seekbarRoot),
              buffer: bufferedDataRatio.value,
              modelValue: rangePercent.value,
              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((rangePercent).value = $event))
            }, null, 8 /* PROPS */, ["sliderBgWhite", "buffer", "modelValue"]) ]) ])) ], 42 /* CLASS, PROPS, NEED_HYDRATION */, ["onContextmenu", "onKeydown"])), [ [_directive_hotkey, _unref(keymap)] ])
}
}

})
