import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, TransitionGroup as _TransitionGroup, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("img", { src: "/client-assets/drop-and-fusion/gameover.png", style: "width: 200px; max-width: 100%; display: block; margin: auto; margin-bottom: -5px;" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-play" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-stop" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-track-next" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-track-next" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("div", null, "FUSION RECIPE")
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-big-right" })
import { computed, onDeactivated, onMounted, onUnmounted, ref, shallowRef, watch, useTemplateRef } from 'vue'
import * as Matter from 'matter-js'
import * as Misskey from 'misskey-js'
import { DropAndFusionGame } from 'misskey-bubble-game'
import { useInterval } from '@@/js/use-interval.js'
import { apiUrl } from '@@/js/config.js'
import type { Mono } from 'misskey-bubble-game'
import { definePage } from '@/page.js'
import MkRippleEffect from '@/components/MkRippleEffect.vue'
import * as os from '@/os.js'
import MkNumber from '@/components/MkNumber.vue'
import MkPlusOneEffect from '@/components/MkPlusOneEffect.vue'
import MkButton from '@/components/MkButton.vue'
import { claimAchievement } from '@/utility/achievements.js'
import { store } from '@/store.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { $i } from '@/i.js'
import * as sound from '@/utility/sound.js'
import MkRange from '@/components/MkRange.vue'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'
import { prefer } from '@/preferences.js'

type FrontendMonoDefinition = {
	id: string;
	img: string;
	imgSizeX: number;
	imgSizeY: number;
	spriteScale: number;
	sfxPitch: number;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'drop-and-fusion.game',
  props: {
    gameMode: { type: String, required: true },
    mute: { type: Boolean, required: true }
  },
  emits: ["end"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const NORAML_MONOS: FrontendMonoDefinition[] = [{
	id: '9377076d-c980-4d83-bdaf-175bc58275b7',
	sfxPitch: 0.25,
	img: '/client-assets/drop-and-fusion/normal_monos/exploding_head.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: 'be9f38d2-b267-4b1a-b420-904e22e80568',
	sfxPitch: 0.5,
	img: '/client-assets/drop-and-fusion/normal_monos/face_with_symbols_on_mouth.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: 'beb30459-b064-4888-926b-f572e4e72e0c',
	sfxPitch: 0.75,
	img: '/client-assets/drop-and-fusion/normal_monos/cold_face.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: 'feab6426-d9d8-49ae-849c-048cdbb6cdf0',
	sfxPitch: 1,
	img: '/client-assets/drop-and-fusion/normal_monos/zany_face.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: 'd6d8fed6-6d18-4726-81a1-6cf2c974df8a',
	sfxPitch: 1.5,
	img: '/client-assets/drop-and-fusion/normal_monos/pleading_face.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '249c728e-230f-4332-bbbf-281c271c75b2',
	sfxPitch: 2,
	img: '/client-assets/drop-and-fusion/normal_monos/face_with_open_mouth.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '23d67613-d484-4a93-b71e-3e81b19d6186',
	sfxPitch: 2.5,
	img: '/client-assets/drop-and-fusion/normal_monos/smiling_face_with_sunglasses.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '3cbd0add-ad7d-4685-bad0-29f6dddc0b99',
	sfxPitch: 3,
	img: '/client-assets/drop-and-fusion/normal_monos/grinning_squinting_face.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '8f86d4f4-ee02-41bf-ad38-1ce0ae457fb5',
	sfxPitch: 3.5,
	img: '/client-assets/drop-and-fusion/normal_monos/smiling_face_with_hearts.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '64ec4add-ce39-42b4-96cb-33908f3f118d',
	sfxPitch: 4,
	img: '/client-assets/drop-and-fusion/normal_monos/heart_suit.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}];
const YEN_MONOS: FrontendMonoDefinition[] = [{
	id: '880f9bd9-802f-4135-a7e1-fd0e0331f726',
	sfxPitch: 0.25,
	img: '/client-assets/drop-and-fusion/yen_monos/10000yen.png',
	imgSizeX: 512,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: 'e807beb6-374a-4314-9cc2-aa5f17d96b6b',
	sfxPitch: 0.5,
	img: '/client-assets/drop-and-fusion/yen_monos/5000yen.png',
	imgSizeX: 512,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: '033445b7-8f90-4fc9-beca-71a9e87cb530',
	sfxPitch: 0.75,
	img: '/client-assets/drop-and-fusion/yen_monos/2000yen.png',
	imgSizeX: 512,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: '410a09ec-5f7f-46f6-b26f-cbca4ccbd091',
	sfxPitch: 1,
	img: '/client-assets/drop-and-fusion/yen_monos/1000yen.png',
	imgSizeX: 512,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: '2aae82bc-3fa4-49ad-a6b5-94d888e809f5',
	sfxPitch: 1.5,
	img: '/client-assets/drop-and-fusion/yen_monos/500yen.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: 'a619bd67-d08f-4cc0-8c7e-c8072a4950cd',
	sfxPitch: 2,
	img: '/client-assets/drop-and-fusion/yen_monos/100yen.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: 'c1c5d8e4-17d6-4455-befd-12154d731faa',
	sfxPitch: 2.5,
	img: '/client-assets/drop-and-fusion/yen_monos/50yen.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: '7082648c-e428-44c4-887a-25c07a8ebdd5',
	sfxPitch: 3,
	img: '/client-assets/drop-and-fusion/yen_monos/10yen.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: '0d8d40d5-e6e0-4d26-8a95-b8d842363379',
	sfxPitch: 3.5,
	img: '/client-assets/drop-and-fusion/yen_monos/5yen.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 0.97,
}, {
	id: '9dec1b38-d99d-40de-8288-37367b983d0d',
	sfxPitch: 4,
	img: '/client-assets/drop-and-fusion/yen_monos/1yen.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 0.97,
}];
const SQUARE_MONOS: FrontendMonoDefinition[] = [{
	id: 'f75fd0ba-d3d4-40a4-9712-b470e45b0525',
	sfxPitch: 0.25,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_10.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '7b70f4af-1c01-45fd-af72-61b1f01e03d1',
	sfxPitch: 0.5,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_9.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '41607ef3-b6d6-4829-95b6-3737bf8bb956',
	sfxPitch: 0.75,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_8.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '8a8310d2-0374-460f-bb50-ca9cd3ee3416',
	sfxPitch: 1,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_7.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '1092e069-fe1a-450b-be97-b5d477ec398c',
	sfxPitch: 1.5,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_6.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '2294734d-7bb8-4781-bb7b-ef3820abf3d0',
	sfxPitch: 2,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_5.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: 'ea8a61af-e350-45f7-ba6a-366fcd65692a',
	sfxPitch: 2.5,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_4.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: 'd0c74815-fc1c-4fbe-9953-c92e4b20f919',
	sfxPitch: 3,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_3.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: 'd8fbd70e-611d-402d-87da-1a7fd8cd2c8d',
	sfxPitch: 3.5,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_2.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}, {
	id: '35e476ee-44bd-4711-ad42-87be245d3efd',
	sfxPitch: 4,
	img: '/client-assets/drop-and-fusion/square_monos/keycap_1.png',
	imgSizeX: 256,
	imgSizeY: 256,
	spriteScale: 1.12,
}];
const SWEETS_MONOS: FrontendMonoDefinition[] = [{
	id: '77f724c0-88be-4aeb-8e1a-a00ed18e3844',
	sfxPitch: 0.25,
	img: '/client-assets/drop-and-fusion/sweets_monos/shortcake_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: 'f3468ef4-2e1e-4906-8795-f147f39f7e1f',
	sfxPitch: 0.5,
	img: '/client-assets/drop-and-fusion/sweets_monos/pancakes_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: 'bcb41129-6f2d-44ee-89d3-86eb2df564ba',
	sfxPitch: 0.75,
	img: '/client-assets/drop-and-fusion/sweets_monos/shaved_ice_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: 'f058e1ad-1981-409b-b3a7-302de0a43744',
	sfxPitch: 1,
	img: '/client-assets/drop-and-fusion/sweets_monos/soft_ice_cream_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: 'd22cfe38-5a3b-4b9c-a1a6-907930a3d732',
	sfxPitch: 1.5,
	img: '/client-assets/drop-and-fusion/sweets_monos/doughnut_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: '79867083-a073-427e-ae82-07a70d9f3b4f',
	sfxPitch: 2,
	img: '/client-assets/drop-and-fusion/sweets_monos/custard_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: '2e152a12-a567-4100-b4d4-d15d81ba47b1',
	sfxPitch: 2.5,
	img: '/client-assets/drop-and-fusion/sweets_monos/chocolate_bar_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: '12250376-2258-4716-8eec-b3a7239461fc',
	sfxPitch: 3,
	img: '/client-assets/drop-and-fusion/sweets_monos/lollipop_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: '4d4f2668-4be7-44a3-aa3a-856df6e25aa6',
	sfxPitch: 3.5,
	img: '/client-assets/drop-and-fusion/sweets_monos/candy_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}, {
	id: 'c9984b40-4045-44c3-b260-d47b7b4625b2',
	sfxPitch: 4,
	img: '/client-assets/drop-and-fusion/sweets_monos/cookie_color.svg',
	imgSizeX: 32,
	imgSizeY: 32,
	spriteScale: 1,
}];
const monoDefinitions = computed(() => {
	return props.gameMode === 'normal' ? NORAML_MONOS :
		props.gameMode === 'square' ? SQUARE_MONOS :
		props.gameMode === 'yen' ? YEN_MONOS :
		props.gameMode === 'sweets' ? SWEETS_MONOS :
		props.gameMode === 'space' ? NORAML_MONOS :
		[] as never;
});
function getScoreUnit(gameMode: string) {
	return gameMode === 'normal' ? 'pt' :
		gameMode === 'square' ? 'pt' :
		gameMode === 'yen' ? '円' :
		gameMode === 'sweets' ? 'kcal' :
		'' as never;
}
function getMonoRenderOptions(mono: Mono) {
	const def = monoDefinitions.value.find(x => x.id === mono.id)!;
	return {
		sprite: {
			texture: def.img,
			xScale: (mono.sizeX / def.imgSizeX) * def.spriteScale,
			yScale: (mono.sizeY / def.imgSizeY) * def.spriteScale,
		},
	};
}
let viewScale = 1;
let seed: string = Date.now().toString();
let containerElRect: DOMRect | null = null;
let logs: ReturnType<DropAndFusionGame['getLogs']> | null = null;
let endedAtFrame = 0;
let bgmNodes: ReturnType<typeof sound.createSourceNode> | null = null;
let renderer: Matter.Render | null = null;
let monoTextures: Record<string, Blob> = {};
let monoTextureUrls: Record<string, string> = {};
let tickRaf: number | null = null;
let game = new DropAndFusionGame({
	seed: seed,
	gameMode: props.gameMode,
	getMonoRenderOptions,
});
attachGameEvents();
const containerEl = useTemplateRef('containerEl');
const canvasEl = useTemplateRef('canvasEl');
const dropperX = ref(0);
const currentPick = shallowRef<{ id: string; mono: Mono } | null>(null);
const stock = shallowRef<{ id: string; mono: Mono }[]>([]);
const holdingStock = shallowRef<{ id: string; mono: Mono } | null>(null);
const score = ref(0);
const combo = ref(0);
const comboPrev = ref(0);
const maxCombo = ref(0);
const dropReady = ref(true);
const isGameOver = ref(false);
const gameLoaded = ref(false);
const readyGo = ref<'ready' | 'go' | null>('ready');
const highScore = ref<number | null>(null);
const yenTotal = ref<number | null>(null);
const showConfig = ref(false);
const replaying = ref(false);
const replayPlaybackRate = ref(1);
const currentFrame = ref(0);
const bgmVolume = ref(prefer.s['game.dropAndFusion'].bgmVolume);
const sfxVolume = ref(prefer.s['game.dropAndFusion'].sfxVolume);
watch(replayPlaybackRate, (newValue) => {
	game.replayPlaybackRate = newValue;
});
watch(bgmVolume, (newValue) => {
	if (bgmNodes) {
		bgmNodes.gainNode.gain.value = props.mute ? 0 : newValue;
	}
});
function createRendererInstance(game: DropAndFusionGame) {
	return Matter.Render.create({
		engine: game.engine,
		canvas: canvasEl.value!,
		options: {
			width: game.GAME_WIDTH,
			height: game.GAME_HEIGHT,
			background: 'transparent', // transparent to hide
			wireframeBackground: 'transparent', // transparent to hide
			wireframes: false,
			showSleeping: false,
			pixelRatio: Math.max(2, window.devicePixelRatio),
		},
	});
}
function loadMonoTextures() {
	async function loadSingleMonoTexture(mono: FrontendMonoDefinition) {
		if (renderer == null) return;
		// Matter-js内にキャッシュがある場合はスキップ
		if (renderer.textures[mono.img]) return;
		let src = mono.img;
		if (monoTextureUrls[mono.img]) {
			src = monoTextureUrls[mono.img];
			// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
		} else if (monoTextures[mono.img]) {
			src = URL.createObjectURL(monoTextures[mono.img]);
			monoTextureUrls[mono.img] = src;
		} else {
			const res = await window.fetch(mono.img);
			const blob = await res.blob();
			monoTextures[mono.img] = blob;
			src = URL.createObjectURL(blob);
			monoTextureUrls[mono.img] = src;
		}
		const image = new Image();
		image.src = src;
		renderer.textures[mono.img] = image;
	}
	return Promise.all(monoDefinitions.value.map(x => loadSingleMonoTexture(x)));
}
function getTextureImageUrl(mono: Mono) {
	const def = monoDefinitions.value.find(x => x.id === mono.id)!;
	if (monoTextureUrls[def.img]) {
		return monoTextureUrls[def.img];
		// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
	} else if (monoTextures[def.img]) {
		// Gameクラス内にキャッシュがある場合はそれを使う
		const out = URL.createObjectURL(monoTextures[def.img]);
		monoTextureUrls[def.img] = out;
		return out;
	} else {
		return def.img;
	}
}
function tick() {
	const hasNextTick = game.tick();
	if (hasNextTick) {
		tickRaf = window.requestAnimationFrame(tick);
	} else {
		tickRaf = null;
	}
}
function tickReplay() {
	let hasNextTick;
	for (let i = 0; i < replayPlaybackRate.value; i++) {
		const log = logs!.find(x => x.frame === game.frame);
		if (log) {
			switch (log.operation) {
				case 'drop': {
					game.drop(log.x);
					break;
				}
				case 'hold': {
					game.hold();
					break;
				}
				case 'surrender': {
					game.surrender();
					break;
				}
				default:
					break;
			}
		}
		hasNextTick = game.tick();
		currentFrame.value = game.frame;
		if (!hasNextTick) break;
	}
	if (hasNextTick) {
		tickRaf = window.requestAnimationFrame(tickReplay);
	} else {
		tickRaf = null;
	}
}
async function start() {
	renderer = createRendererInstance(game);
	await loadMonoTextures();
	Matter.Render.lookAt(renderer, {
		min: { x: 0, y: 0 },
		max: { x: game.GAME_WIDTH, y: game.GAME_HEIGHT },
	});
	Matter.Render.run(renderer);
	game.start();
	window.requestAnimationFrame(tick);
	gameLoaded.value = true;
	window.setTimeout(() => {
		readyGo.value = 'go';
		window.setTimeout(() => {
			readyGo.value = null;
		}, 1000);
	}, 1500);
}
function onClick(ev: PointerEvent) {
	if (!containerElRect) return;
	if (replaying.value) return;
	const x = (ev.clientX - containerElRect.left) / viewScale;
	game.drop(x);
}
function onTouchend(ev: TouchEvent) {
	if (!containerElRect) return;
	if (replaying.value) return;
	const x = (ev.changedTouches[0].clientX - containerElRect.left) / viewScale;
	game.drop(x);
}
function onMousemove(ev: MouseEvent) {
	if (!containerElRect) return;
	const x = (ev.clientX - containerElRect.left);
	moveDropper(containerElRect, x);
}
function onTouchmove(ev: TouchEvent) {
	if (!containerElRect) return;
	const x = (ev.touches[0].clientX - containerElRect.left);
	moveDropper(containerElRect, x);
}
function moveDropper(rect: DOMRect, x: number) {
	dropperX.value = Math.min(rect.width * ((game.GAME_WIDTH - game.PLAYAREA_MARGIN) / game.GAME_WIDTH), Math.max(rect.width * (game.PLAYAREA_MARGIN / game.GAME_WIDTH), x));
}
function hold() {
	game.hold();
}
async function surrender() {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.ts.areYouSure,
	});
	if (canceled) return;
	game.surrender();
}
async function restart() {
	reset();
	game = new DropAndFusionGame({
		seed: seed,
		gameMode: props.gameMode,
		getMonoRenderOptions,
	});
	attachGameEvents();
	await start();
}
function reset() {
	dispose();
	seed = Date.now().toString();
	isGameOver.value = false;
	replaying.value = false;
	replayPlaybackRate.value = 1;
	currentPick.value = null;
	dropReady.value = true;
	stock.value = [];
	holdingStock.value = null;
	score.value = 0;
	combo.value = 0;
	comboPrev.value = 0;
	maxCombo.value = 0;
	gameLoaded.value = false;
	readyGo.value = null;
}
function dispose() {
	game.dispose();
	if (renderer) Matter.Render.stop(renderer);
	if (tickRaf) {
		window.cancelAnimationFrame(tickRaf);
	}
}
function backToTitle() {
	emit('end');
}
function replay() {
	replaying.value = true;
	dispose();
	game = new DropAndFusionGame({
		seed: seed,
		gameMode: props.gameMode,
		getMonoRenderOptions,
	});
	attachGameEvents();
	os.promiseDialog(loadMonoTextures(), async () => {
		renderer = createRendererInstance(game);
		Matter.Render.lookAt(renderer, {
			min: { x: 0, y: 0 },
			max: { x: game.GAME_WIDTH, y: game.GAME_HEIGHT },
		});
		Matter.Render.run(renderer);
		game.start();
		window.requestAnimationFrame(tickReplay);
	});
}
function endReplay() {
	replaying.value = false;
	dispose();
}
function exportLog() {
	if (!logs) return;
	const data = JSON.stringify({
		v: game.GAME_VERSION,
		m: props.gameMode,
		s: seed,
		d: new Date().toISOString(),
		l: DropAndFusionGame.serializeLogs(logs),
	});
	copyToClipboard(data);
}
function updateSettings<
	K extends keyof typeof prefer.s['game.dropAndFusion'],
	V extends typeof prefer.s['game.dropAndFusion'][K],
>(key: K, value: V) {
	const changes: { [P in K]?: V } = {};
	changes[key] = value;
	prefer.commit('game.dropAndFusion', {
		...prefer.s['game.dropAndFusion'],
		...changes,
	});
}
function loadImage(url: string) {
	return new Promise<HTMLImageElement>(res => {
		const img = new Image();
		img.src = url;
		img.addEventListener('load', () => {
			res(img);
		});
	});
}
function getGameImageDriveFile() {
	return new Promise<Misskey.entities.DriveFile | null>(res => {
		const dcanvas = window.document.createElement('canvas');
		dcanvas.width = game.GAME_WIDTH;
		dcanvas.height = game.GAME_HEIGHT;
		const ctx = dcanvas.getContext('2d');
		if (!ctx || !canvasEl.value) return res(null);
		Promise.all([
			loadImage('/client-assets/drop-and-fusion/frame-light.svg'),
			loadImage('/client-assets/drop-and-fusion/logo.png'),
		]).then((images) => {
			const [frame, logo] = images;
			ctx.fillStyle = '#fff';
			ctx.fillRect(0, 0, game.GAME_WIDTH, game.GAME_HEIGHT);
			ctx.drawImage(frame, 0, 0, game.GAME_WIDTH, game.GAME_HEIGHT);
			ctx.drawImage(canvasEl.value!, 0, 0, game.GAME_WIDTH, game.GAME_HEIGHT);
			ctx.fillStyle = '#000';
			ctx.font = '16px bold sans-serif';
			ctx.textBaseline = 'top';
			ctx.fillText(`SCORE: ${score.value.toLocaleString()}${getScoreUnit(props.gameMode)}`, 10, 10);
			ctx.globalAlpha = 0.7;
			ctx.drawImage(logo, game.GAME_WIDTH * 0.55, 6, game.GAME_WIDTH * 0.45, game.GAME_WIDTH * 0.45 * (logo.height / logo.width));
			ctx.globalAlpha = 1;
			dcanvas.toBlob(blob => {
				if (!blob) return res(null);
				if ($i == null) return res(null);
				const formData = new FormData();
				formData.append('file', blob);
				formData.append('name', `bubble-game-${Date.now()}.png`);
				formData.append('isSensitive', 'false');
				formData.append('i', $i.token);
				if (prefer.s.uploadFolder) {
					formData.append('folderId', prefer.s.uploadFolder);
				}
				window.fetch(apiUrl + '/drive/files/create', {
					method: 'POST',
					body: formData,
				})
					.then(response => response.json())
					.then(f => {
						res(f);
					});
			}, 'image/png');
			dcanvas.remove();
		});
	});
}
async function share() {
	const uploading = getGameImageDriveFile();
	os.promiseDialog(uploading);
	const file = await uploading;
	if (!file) return;
	os.post({
		initialText: `#BubbleGame (${props.gameMode})
SCORE: ${score.value.toLocaleString()}${getScoreUnit(props.gameMode)}`,
		initialFiles: [file],
		instant: true,
	});
}
function attachGameEvents() {
	game.addListener('changeScore', value => {
		score.value = value;
	});
	game.addListener('changeCombo', value => {
		if (value === 0) {
			comboPrev.value = combo.value;
		} else {
			comboPrev.value = value;
		}
		maxCombo.value = Math.max(maxCombo.value, value);
		combo.value = value;
	});
	game.addListener('changeStock', value => {
		currentPick.value = JSON.parse(JSON.stringify(value[0]));
		stock.value = JSON.parse(JSON.stringify(value.slice(1)));
	});
	game.addListener('changeHolding', value => {
		holdingStock.value = value;
		if (!props.mute) {
			sound.playUrl('/client-assets/drop-and-fusion/hold.mp3', {
				volume: 0.5 * sfxVolume.value,
				playbackRate: replayPlaybackRate.value,
			});
		}
	});
	game.addListener('dropped', (x) => {
		if (!props.mute) {
			const panV = x - game.PLAYAREA_MARGIN;
			const panW = game.GAME_WIDTH - game.PLAYAREA_MARGIN - game.PLAYAREA_MARGIN;
			const pan = ((panV / panW) - 0.5) * 2;
			if (props.gameMode === 'yen') {
				sound.playUrl('/client-assets/drop-and-fusion/drop_yen.mp3', {
					volume: sfxVolume.value,
					pan,
					playbackRate: replayPlaybackRate.value,
				});
			} else {
				sound.playUrl('/client-assets/drop-and-fusion/drop.mp3', {
					volume: sfxVolume.value,
					pan,
					playbackRate: replayPlaybackRate.value,
				});
			}
		}
		if (replaying.value) return;
		dropReady.value = false;
		window.setTimeout(() => {
			if (!isGameOver.value) {
				dropReady.value = true;
			}
		}, game.frameToMs(game.DROP_COOLTIME));
	});
	game.addListener('fusioned', (x, y, nextMono, scoreDelta) => {
		if (!canvasEl.value) return;
		const rect = canvasEl.value.getBoundingClientRect();
		const domX = rect.left + (x * viewScale);
		const domY = rect.top + (y * viewScale);
		const scoreUnit = getScoreUnit(props.gameMode);
		{
			const { dispose } = os.popup(MkRippleEffect, { x: domX, y: domY }, {
				end: () => dispose(),
			});
		}
		{
			const { dispose } = os.popup(MkPlusOneEffect, { x: domX, y: domY, value: scoreDelta + (scoreUnit === 'pt' ? '' : scoreUnit) }, {
				end: () => dispose(),
			});
		}
		if (nextMono) {
			const def = monoDefinitions.value.find(x => x.id === nextMono.id)!;
			if (!props.mute) {
				const panV = x - game.PLAYAREA_MARGIN;
				const panW = game.GAME_WIDTH - game.PLAYAREA_MARGIN - game.PLAYAREA_MARGIN;
				const pan = ((panV / panW) - 0.5) * 2;
				const pitch = def.sfxPitch;
				if (props.gameMode === 'yen') {
					sound.playUrl('/client-assets/drop-and-fusion/fusion_yen.mp3', {
						volume: 0.25 * sfxVolume.value,
						pan: pan,
						playbackRate: (pitch / 4) * replayPlaybackRate.value,
					});
				} else {
					sound.playUrl('/client-assets/drop-and-fusion/fusion.mp3', {
						volume: sfxVolume.value,
						pan: pan,
						playbackRate: pitch * replayPlaybackRate.value,
					});
				}
			}
		} else {
			if (!props.mute) {
				// TODO: 融合後のモノがない場合でも何らかの効果音を再生
			}
		}
	});
	const minCollisionEnergyForSound = 2.5;
	const maxCollisionEnergyForSound = 9;
	const soundPitchMax = 4;
	const soundPitchMin = 0.5;
	game.addListener('collision', (energy, bodyA, bodyB) => {
		if (!props.mute && (energy > minCollisionEnergyForSound)) {
			const volume = (Math.min(maxCollisionEnergyForSound, energy - minCollisionEnergyForSound) / maxCollisionEnergyForSound) / 4;
			const panV =
				bodyA.label === '_wall_' ? bodyB.position.x - game.PLAYAREA_MARGIN :
				bodyB.label === '_wall_' ? bodyA.position.x - game.PLAYAREA_MARGIN :
				((bodyA.position.x + bodyB.position.x) / 2) - game.PLAYAREA_MARGIN;
			const panW = game.GAME_WIDTH - game.PLAYAREA_MARGIN - game.PLAYAREA_MARGIN;
			const pan = ((panV / panW) - 0.5) * 2;
			const pitch = soundPitchMin + ((soundPitchMax - soundPitchMin) * (1 - (Math.min(10, energy) / 10)));
			if (props.gameMode === 'yen') {
				sound.playUrl('/client-assets/drop-and-fusion/collision_yen.mp3', {
					volume: volume * sfxVolume.value,
					pan: pan,
					playbackRate: Math.max(1, pitch) * replayPlaybackRate.value,
				});
			} else {
				sound.playUrl('/client-assets/drop-and-fusion/collision.mp3', {
					volume: volume * sfxVolume.value,
					pan: pan,
					playbackRate: pitch * replayPlaybackRate.value,
				});
			}
		}
	});
	game.addListener('monoAdded', (mono) => {
		if (replaying.value) return;
		// 実績関連
		if (mono.level === 10) {
			claimAchievement('bubbleGameExplodingHead');
			const monos = game.getActiveMonos();
			if (monos.filter(x => x.level === 10).length >= 2) {
				claimAchievement('bubbleGameDoubleExplodingHead');
			}
		}
	});
	game.addListener('gameOver', () => {
		if (!props.mute) {
			if (props.gameMode === 'yen') {
				sound.playUrl('/client-assets/drop-and-fusion/gameover_yen.mp3', {
					volume: 0.5 * sfxVolume.value,
				});
			} else {
				sound.playUrl('/client-assets/drop-and-fusion/gameover.mp3', {
					volume: sfxVolume.value,
				});
			}
		}
		if (replaying.value) {
			endReplay();
			return;
		}
		logs = game.getLogs();
		endedAtFrame = game.frame;
		currentPick.value = null;
		dropReady.value = false;
		isGameOver.value = true;
		misskeyApi('bubble-game/register', {
			seed,
			score: score.value,
			gameMode: props.gameMode,
			gameVersion: game.GAME_VERSION,
			logs: DropAndFusionGame.serializeLogs(logs),
		});
		if (props.gameMode === 'yen') {
			yenTotal.value = (yenTotal.value ?? 0) + score.value;
			misskeyApi('i/registry/set', {
				scope: ['dropAndFusionGame'],
				key: 'yenTotal',
				value: yenTotal.value,
			});
		}
		if (score.value > (highScore.value ?? 0)) {
			highScore.value = score.value;
			misskeyApi('i/registry/set', {
				scope: ['dropAndFusionGame'],
				key: 'highScore:' + props.gameMode,
				value: highScore.value,
			});
		}
	});
}
useInterval(() => {
	if (!canvasEl.value) return;
	const actualCanvasWidth = canvasEl.value.getBoundingClientRect().width;
	if (actualCanvasWidth === 0) return;
	viewScale = actualCanvasWidth / game.GAME_WIDTH;
	containerElRect = containerEl.value?.getBoundingClientRect() ?? null;
}, 1000, { immediate: false, afterMounted: true });
onMounted(async () => {
	try {
		highScore.value = await misskeyApi('i/registry/get', {
			scope: ['dropAndFusionGame'],
			key: 'highScore:' + props.gameMode,
		});
	} catch (err) {
		highScore.value = null;
	}
	if (props.gameMode === 'yen') {
		try {
			yenTotal.value = await misskeyApi('i/registry/get', {
				scope: ['dropAndFusionGame'],
				key: 'yenTotal',
			});
		} catch (err: any) {
			if (err.code === 'NO_SUCH_KEY') {
				// nop
			} else {
				os.alert({
					type: 'error',
					text: i18n.ts.cannotLoad,
				});
				return;
			}
		}
	}
	/*
	const getVerticesFromSvg = async (path: string) => {
		const svgDoc = await fetch(path)
			.then((response) => response.text())
			.then((svgString) => {
				const parser = new DOMParser();
				return parser.parseFromString(svgString, 'image/svg+xml');
			});
		const pathDatas = svgDoc.querySelectorAll('path');
		if (!pathDatas) return;
		const vertices = Array.from(pathDatas).map((pathData) => {
			return Matter.Svg.pathToVertices(pathData);
		});
		return vertices;
	};
	getVerticesFromSvg('/client-assets/drop-and-fusion/sweets_monos/verts/doughnut_color.svg').then((vertices) => {
		console.log('doughnut_color', vertices);
	});
	*/
	await start();
	const bgmBuffer = await sound.loadAudio('/client-assets/drop-and-fusion/bgm_1.mp3');
	if (!bgmBuffer) return;
	bgmNodes = sound.createSourceNode(bgmBuffer, {
		volume: props.mute ? 0 : bgmVolume.value,
	});
	if (!bgmNodes) return;
	bgmNodes.soundSource.loop = true;
	bgmNodes.soundSource.start();
});
onUnmounted(() => {
	dispose();
	bgmNodes?.soundSource.stop();
});
onDeactivated(() => {
	dispose();
	bgmNodes?.soundSource.stop();
});
definePage(() => ({
	title: i18n.ts.bubbleGame,
	icon: 'ti ti-apple',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createElementBlock("div", {
      class: "_spacer",
      style: "--MI_SPACER-w: 800px;"
    }, [ _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.root)
      }, [ (!gameLoaded.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.loadingScreen)
          }, [ _createElementVNode("div", null, [ _createTextVNode(_toDisplayString(_unref(i18n).ts.loading), 1 /* TEXT */), _createVNode(_component_MkEllipsis) ]) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\t\t" + "\n\t\t"), _withDirectives(_createElementVNode("div", { class: "_gaps_s" }, [ (readyGo.value === 'ready') ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.readyGo_bg)
            })) : _createCommentVNode("v-if", true), _createVNode(_Transition, {
            enterActiveClass: _ctx.$style.transition_zoom_enterActive,
            leaveActiveClass: _ctx.$style.transition_zoom_leaveActive,
            enterFromClass: _ctx.$style.transition_zoom_enterFrom,
            leaveToClass: _ctx.$style.transition_zoom_leaveTo,
            moveClass: _ctx.$style.transition_zoom_move,
            mode: "default"
          }, {
            default: _withCtx(() => [
              (readyGo.value === 'ready')
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.readyGo_ready)
                }, [
                  _createElementVNode("img", {
                    src: "/client-assets/drop-and-fusion/ready.png",
                    class: _normalizeClass(_ctx.$style.readyGo_img)
                  })
                ]))
                : (readyGo.value === 'go')
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 1,
                    class: _normalizeClass(_ctx.$style.readyGo_go)
                  }, [
                    _createElementVNode("img", {
                      src: "/client-assets/drop-and-fusion/go.png",
                      class: _normalizeClass(_ctx.$style.readyGo_img)
                    })
                  ]))
                : _createCommentVNode("v-if", true)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"]), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.header)
          }, [ _createElementVNode("div", {
              class: _normalizeClass(["_woodenFrame", [_ctx.$style.headerTitle]])
            }, [ _createElementVNode("div", { class: "_woodenFrameInner" }, [ _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.bubbleGame), 1 /* TEXT */), _createElementVNode("div", null, "- " + _toDisplayString(__props.gameMode.toUpperCase()) + " -", 1 /* TEXT */) ]) ]), _createElementVNode("div", { class: "_woodenFrame _woodenFrameH" }, [ _createElementVNode("div", { class: "_woodenFrameInner" }, [ _createVNode(MkButton, {
                  inline: "",
                  small: "",
                  onClick: hold
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._bubbleGame.hold), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }), (holdingStock.value) ? (_openBlock(), _createElementBlock("img", {
                    key: 0,
                    src: getTextureImageUrl(holdingStock.value.mono),
                    style: "width: 32px; margin-left: 8px; vertical-align: bottom;"
                  })) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", {
                class: _normalizeClass(["_woodenFrameInner", _ctx.$style.stock]),
                style: "text-align: center;"
              }, [ _createVNode(_TransitionGroup, {
                  enterActiveClass: _ctx.$style.transition_stock_enterActive,
                  leaveActiveClass: _ctx.$style.transition_stock_leaveActive,
                  enterFromClass: _ctx.$style.transition_stock_enterFrom,
                  leaveToClass: _ctx.$style.transition_stock_leaveTo,
                  moveClass: _ctx.$style.transition_stock_move
                }, {
                  default: _withCtx(() => [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(stock.value, (x) => {
                      return (_openBlock(), _createElementBlock("img", {
                        key: x.id,
                        src: getTextureImageUrl(x.mono),
                        style: "width: 32px; vertical-align: bottom;"
                      }, 8 /* PROPS */, ["src"]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"]) ]) ]) ]), _createElementVNode("div", {
            ref_key: "containerEl", ref: containerEl,
            class: _normalizeClass([_ctx.$style.gameContainer, { [_ctx.$style.gameOver]: isGameOver.value && !replaying.value }]),
            onContextmenu: _withModifiers(() => {}, ["stop","prevent"]),
            onClick: _withModifiers(onClick, ["stop","prevent"]),
            onTouchmove: _withModifiers(onTouchmove, ["stop","prevent"]),
            onTouchend: onTouchend,
            onMousemove: onMousemove
          }, [ (_unref(store).s.darkMode) ? (_openBlock(), _createElementBlock("img", {
                key: 0,
                src: "/client-assets/drop-and-fusion/frame-dark.svg",
                class: _normalizeClass(_ctx.$style.mainFrameImg)
              })) : (_openBlock(), _createElementBlock("img", {
                key: 1,
                src: "/client-assets/drop-and-fusion/frame-light.svg",
                class: _normalizeClass(_ctx.$style.mainFrameImg)
              })), _createElementVNode("canvas", {
              ref_key: "canvasEl", ref: canvasEl,
              class: _normalizeClass(_ctx.$style.canvas)
            }, null, 512 /* NEED_PATCH */), _createVNode(_Transition, {
              enterActiveClass: _ctx.$style.transition_combo_enterActive,
              leaveActiveClass: _ctx.$style.transition_combo_leaveActive,
              enterFromClass: _ctx.$style.transition_combo_enterFrom,
              leaveToClass: _ctx.$style.transition_combo_leaveTo,
              moveClass: _ctx.$style.transition_combo_move
            }, {
              default: _withCtx(() => [
                _withDirectives(_createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.combo),
                  style: _normalizeStyle({ fontSize: `${100 + ((comboPrev.value - 2) * 15)}%` })
                }, _toDisplayString(comboPrev.value) + " Chain!", 5 /* TEXT, STYLE */), [
                  [_vShow, combo.value > 1]
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"]), (!isGameOver.value && !replaying.value && readyGo.value !== 'ready') ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.dropperContainer),
                style: _normalizeStyle({ left: dropperX.value + 'px' })
              }, [ _createVNode(_Transition, {
                  enterActiveClass: _ctx.$style.transition_picked_enterActive,
                  leaveActiveClass: _ctx.$style.transition_picked_leaveActive,
                  enterFromClass: _ctx.$style.transition_picked_enterFrom,
                  leaveToClass: _ctx.$style.transition_picked_leaveTo,
                  moveClass: _ctx.$style.transition_picked_move,
                  mode: "out-in"
                }, {
                  default: _withCtx(() => [
                    (currentPick.value)
                      ? (_openBlock(), _createElementBlock("img", {
                        key: currentPick.value.id,
                        src: getTextureImageUrl(currentPick.value.mono),
                        class: _normalizeClass(_ctx.$style.currentMono),
                        style: _normalizeStyle({ marginBottom: -((currentPick.value?.mono.sizeY * _unref(viewScale)) / 2) + 'px', left: -((currentPick.value?.mono.sizeX * _unref(viewScale)) / 2) + 'px', width: `${currentPick.value?.mono.sizeX * _unref(viewScale)}px` })
                      }))
                      : _createCommentVNode("v-if", true)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"]), (dropReady.value && currentPick.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("img", {
                      src: "/client-assets/drop-and-fusion/drop-arrow.svg",
                      class: _normalizeClass(_ctx.$style.currentMonoArrow)
                    }), _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.dropGuide)
                    }) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true), (isGameOver.value && !replaying.value) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.gameOverLabel)
              }, [ _createElementVNode("div", { class: "_gaps_s" }, [ _hoisted_1, _createElementVNode("div", null, [ _createTextVNode(_toDisplayString(_unref(i18n).ts._bubbleGame._score.score) + ": ", 1 /* TEXT */), _createVNode(MkNumber, { value: score.value }, null, 8 /* PROPS */, ["value"]), _createTextVNode(_toDisplayString(getScoreUnit(__props.gameMode)), 1 /* TEXT */) ]), _createElementVNode("div", null, [ _createTextVNode(_toDisplayString(_unref(i18n).ts._bubbleGame._score.maxChain) + ": ", 1 /* TEXT */), _createVNode(MkNumber, { value: maxCombo.value }, null, 8 /* PROPS */, ["value"]) ]), (__props.gameMode === 'yen') ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _toDisplayString(_unref(i18n).ts._bubbleGame._score.scoreYen), _createTextVNode(":\n\t\t\t\t\t\t\t"), _createVNode(_component_I18n, {
                        src: _unref(i18n).ts._bubbleGame._score.yen,
                        tag: "b"
                      }, {
                        yen: _withCtx(() => [
                          _createVNode(MkNumber, { value: yenTotal.value ?? score.value }, null, 8 /* PROPS */, ["value"])
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["src"]) ])) : _createCommentVNode("v-if", true), (__props.gameMode === 'sweets') ? (_openBlock(), _createBlock(_component_I18n, {
                      key: 0,
                      src: _unref(i18n).ts._bubbleGame._score.scoreSweets,
                      tag: "div"
                    }, {
                      onigiriQtyWithUnit: _withCtx(() => [
                        _createVNode(_component_I18n, {
                          src: _unref(i18n).ts._bubbleGame._score.estimatedQty,
                          tag: "b"
                        }, {
                          qty: _withCtx(() => [
                            _createVNode(MkNumber, { value: score.value / 130 }, null, 8 /* PROPS */, ["value"])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["src"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["src"])) : _createCommentVNode("v-if", true) ]) ])) : _createCommentVNode("v-if", true), (replaying.value) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.replayIndicator)
              }, [ _createElementVNode("span", {
                  class: _normalizeClass(_ctx.$style.replayIndicatorText)
                }, [ _hoisted_2, _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.replaying), 1 /* TEXT */) ]) ])) : _createCommentVNode("v-if", true) ], 42 /* CLASS, PROPS, NEED_HYDRATION */, ["onContextmenu"]), (replaying.value) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_woodenFrame"
            }, [ _createElementVNode("div", { class: "_woodenFrameInner" }, [ _createElementVNode("div", { style: "background: #0004;" }, [ _createElementVNode("div", {
                    style: _normalizeStyle([{"height":"10px","background":"var(--MI_THEME-accent)","will-change":"width"}, { width: `${(currentFrame.value / _unref(endedAtFrame)) * 100}%` }])
                  }, null, 4 /* STYLE */) ]) ]), _createElementVNode("div", { class: "_woodenFrameInner" }, [ _createElementVNode("div", { class: "_buttonsCenter" }, [ _createVNode(MkButton, { onClick: endReplay }, {
                    default: _withCtx(() => [
                      _hoisted_3,
                      _createTextVNode(" "),
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.endReplay), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }), _createVNode(MkButton, {
                    primary: replayPlaybackRate.value === 4,
                    onClick: _cache[0] || (_cache[0] = ($event: any) => (replayPlaybackRate.value = replayPlaybackRate.value === 4 ? 1 : 4))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_4,
                      _createTextVNode(" x4")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["primary"]), _createVNode(MkButton, {
                    primary: replayPlaybackRate.value === 16,
                    onClick: _cache[1] || (_cache[1] = ($event: any) => (replayPlaybackRate.value = replayPlaybackRate.value === 16 ? 1 : 16))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_5,
                      _createTextVNode(" x16")
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["primary"]) ]) ]) ])) : _createCommentVNode("v-if", true), (isGameOver.value) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_woodenFrame"
            }, [ _createElementVNode("div", { class: "_woodenFrameInner" }, [ _createElementVNode("div", { class: "_buttonsCenter" }, [ _createVNode(MkButton, {
                    primary: "",
                    rounded: "",
                    onClick: backToTitle
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.backToTitle), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }), _createVNode(MkButton, {
                    primary: "",
                    rounded: "",
                    onClick: replay
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.showReplay), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }), _createVNode(MkButton, {
                    primary: "",
                    rounded: "",
                    onClick: share
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.share), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }), _createVNode(MkButton, {
                    rounded: "",
                    onClick: exportLog
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.copyReplayData), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }) ]) ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", { style: "display: flex;" }, [ _createElementVNode("div", {
              class: "_woodenFrame",
              style: "flex: 1; margin-right: 10px;"
            }, [ _createElementVNode("div", { class: "_woodenFrameInner" }, [ _createElementVNode("div", null, [ _createTextVNode(_toDisplayString(_unref(i18n).ts._bubbleGame._score.score) + ": ", 1 /* TEXT */), _createVNode(MkNumber, { value: score.value }, null, 8 /* PROPS */, ["value"]), _createTextVNode(_toDisplayString(getScoreUnit(__props.gameMode)), 1 /* TEXT */) ]), _createElementVNode("div", null, [ _createTextVNode(_toDisplayString(_unref(i18n).ts._bubbleGame._score.highScore) + ": ", 1 /* TEXT */), (highScore.value) ? (_openBlock(), _createElementBlock("b", { key: 0 }, [ _createVNode(MkNumber, { value: highScore.value }, null, 8 /* PROPS */, ["value"]), _toDisplayString(getScoreUnit(__props.gameMode)) ])) : (_openBlock(), _createElementBlock("b", { key: 1 }, "-")) ]), (__props.gameMode === 'yen') ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _toDisplayString(_unref(i18n).ts._bubbleGame._score.scoreYen), _createTextVNode(":\n\t\t\t\t\t\t\t"), _createVNode(_component_I18n, {
                      src: _unref(i18n).ts._bubbleGame._score.yen,
                      tag: "b"
                    }, {
                      yen: _withCtx(() => [
                        _createVNode(MkNumber, { value: yenTotal.value ?? score.value }, null, 8 /* PROPS */, ["value"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["src"]) ])) : _createCommentVNode("v-if", true) ]) ]), _createElementVNode("div", {
              class: "_woodenFrame",
              style: "margin-left: auto;"
            }, [ _createElementVNode("div", {
                class: "_woodenFrameInner",
                style: "text-align: center;"
              }, [ _createElementVNode("div", {
                  onClick: _cache[2] || (_cache[2] = ($event: any) => (showConfig.value = !showConfig.value))
                }, [ _hoisted_6 ]) ]) ]) ]), (showConfig.value) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_woodenFrame"
            }, [ _createElementVNode("div", { class: "_woodenFrameInner" }, [ _createElementVNode("div", { class: "_gaps" }, [ _createVNode(MkRange, {
                    min: 0,
                    max: 1,
                    step: 0.01,
                    textConverter: (v) => `${Math.floor(v * 100)}%`,
                    continuousUpdate: true,
                    onDragEnded: _cache[3] || (_cache[3] = (v) => updateSettings('bgmVolume', v)),
                    modelValue: bgmVolume.value,
                    "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((bgmVolume).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode("BGM " + _toDisplayString(_unref(i18n).ts.volume), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "continuousUpdate", "modelValue"]), _createVNode(MkRange, {
                    min: 0,
                    max: 1,
                    step: 0.01,
                    textConverter: (v) => `${Math.floor(v * 100)}%`,
                    continuousUpdate: true,
                    onDragEnded: _cache[5] || (_cache[5] = (v) => updateSettings('sfxVolume', v)),
                    modelValue: sfxVolume.value,
                    "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((sfxVolume).value = $event))
                  }, {
                    label: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.sfx) + " " + _toDisplayString(_unref(i18n).ts.volume), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["min", "max", "step", "textConverter", "continuousUpdate", "modelValue"]) ]) ]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", { class: "_woodenFrame" }, [ _createElementVNode("div", { class: "_woodenFrameInner" }, [ _hoisted_7, _createElementVNode("div", null, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(game).monoDefinitions.sort((a, b) => a.level - b.level), (mono, i) => {
                  return (_openBlock(), _createElementBlock("div", {
                    key: mono.id,
                    style: "display: inline-block;"
                  }, [
                    _createElementVNode("img", {
                      src: getTextureImageUrl(mono),
                      style: "width: 32px; vertical-align: bottom;"
                    }, null, 8 /* PROPS */, ["src"]),
                    (i < _unref(game).monoDefinitions.length - 1)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        style: "display: inline-block; margin-left: 4px; vertical-align: bottom;"
                      }, [
                        _hoisted_8
                      ]))
                      : _createCommentVNode("v-if", true)
                  ]))
                }), 128 /* KEYED_FRAGMENT */)) ]) ]) ]), _createElementVNode("div", { class: "_woodenFrame" }, [ _createElementVNode("div", { class: "_woodenFrameInner" }, [ (!isGameOver.value && !replaying.value) ? (_openBlock(), _createBlock(MkButton, {
                  key: 0,
                  full: "",
                  danger: "",
                  onClick: surrender
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.surrender), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })) : (_openBlock(), _createBlock(MkButton, {
                  key: 1,
                  full: "",
                  onClick: restart
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.gameRetry), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })) ]) ]) ], 512 /* NEED_PATCH */), [ [_vShow, gameLoaded.value] ]) ]) ]))
}
}

})
