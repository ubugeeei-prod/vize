import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent } from "vue"

import { computed, watch, ref, onMounted, shallowRef, onUnmounted } from 'vue'
import * as Misskey from 'misskey-js'
import GameSetting from './game.setting.vue'
import GameBoard from './game.board.vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { definePage } from '@/page.js'
import { useStream } from '@/stream.js'
import { $i } from '@/i.js'
import { useRouter } from '@/router.js'
import * as os from '@/os.js'
import { url } from '@@/js/config.js'
import { i18n } from '@/i18n.js'
import { useInterval } from '@@/js/use-interval.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'game',
  props: {
    gameId: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const game = shallowRef<Misskey.entities.ReversiGameDetailed | null>(null);
const connection = shallowRef<Misskey.IChannelConnection<Misskey.Channels['reversiGame']> | null>(null);
const shareWhenStart = ref(false);
watch(() => props.gameId, () => {
	fetchGame();
});
function start(_game: Misskey.entities.ReversiGameDetailed) {
	if (game.value?.isStarted) return;
	if (shareWhenStart.value) {
		misskeyApi('notes/create', {
			text: `${i18n.ts._reversi.iStartedAGame}\n${url}/reversi/g/${props.gameId}`,
			visibility: 'home',
		});
	}
	game.value = _game;
}
async function fetchGame() {
	const _game = await misskeyApi('reversi/show-game', {
		gameId: props.gameId,
	});
	game.value = _game;
	shareWhenStart.value = false;
	if (connection.value) {
		connection.value.dispose();
	}
	if (!game.value.isEnded) {
		connection.value = useStream().useChannel('reversiGame', {
			gameId: game.value.id,
		});
		connection.value.on('started', x => {
			start(x.game);
		});
		connection.value.on('canceled', x => {
			connection.value?.dispose();
			if (x.userId !== $i?.id) {
				os.alert({
					type: 'warning',
					text: i18n.ts._reversi.gameCanceled,
				});
				router.push('/reversi');
			}
		});
	}
}
// 通信を取りこぼした場合の救済
useInterval(async () => {
	if (game.value == null) return;
	if (game.value.isStarted) return;
	const _game = await misskeyApi('reversi/show-game', {
		gameId: props.gameId,
	});
	if (_game.isStarted) {
		start(_game);
	} else {
		game.value = _game;
	}
}, 1000 * 10, {
	immediate: false,
	afterMounted: true,
});
onMounted(() => {
	fetchGame();
});
onUnmounted(() => {
	if (connection.value) {
		connection.value.dispose();
	}
});
definePage(() => ({
	title: 'Reversi',
	icon: 'ti ti-device-gamepad',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (game.value == null || (!game.value.isEnded && connection.value == null))
      ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createVNode(_component_MkLoading) ]))
      : (!game.value.isStarted)
        ? (_openBlock(), _createBlock(GameSetting, {
          key: 1,
          game: game.value,
          connection: connection.value,
          shareWhenStart: shareWhenStart.value,
          "onUpdate:shareWhenStart": _cache[0] || (_cache[0] = ($event: any) => ((shareWhenStart).value = $event))
        }, null, 8 /* PROPS */, ["game", "connection", "shareWhenStart"]))
      : (_openBlock(), _createBlock(GameBoard, {
        key: 2,
        game: game.value,
        connection: connection.value
      }, null, 8 /* PROPS */, ["game", "connection"]))
}
}

})
