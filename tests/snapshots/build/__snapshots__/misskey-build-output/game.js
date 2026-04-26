import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent } from "vue";
import { watch, ref, onMounted, shallowRef, onUnmounted } from "vue";
import GameSetting from "./game.setting.vue";
import GameBoard from "./game.board.vue";
import { misskeyApi } from "@/utility/misskey-api.js";
import { definePage } from "@/page.js";
import { useStream } from "@/stream.js";
import { $i } from "@/i.js";
import { useRouter } from "@/router.js";
import * as os from "@/os.js";
import { url } from "@@/js/config.js";
import { i18n } from "@/i18n.js";
import { useInterval } from "@@/js/use-interval.js";
export default {
  __name: "game",
  props: { gameId: {
    type: String,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const router = useRouter();
    const game = shallowRef(null);
    const connection = shallowRef(null);
    const shareWhenStart = ref(false);
    watch(() => props.gameId, () => {
      fetchGame();
    });
    function start(_game) {
      if (game.value?.isStarted) return;
      if (shareWhenStart.value) {
        misskeyApi("notes/create", {
          text: `${i18n.ts._reversi.iStartedAGame}\n${url}/reversi/g/${props.gameId}`,
          visibility: "home"
        });
      }
      game.value = _game;
    }
    async function fetchGame() {
      const _game = await misskeyApi("reversi/show-game", { gameId: props.gameId });
      game.value = _game;
      shareWhenStart.value = false;
      if (connection.value) {
        connection.value.dispose();
      }
      if (!game.value.isEnded) {
        connection.value = useStream().useChannel("reversiGame", { gameId: game.value.id });
        connection.value.on("started", (x) => {
          start(x.game);
        });
        connection.value.on("canceled", (x) => {
          connection.value?.dispose();
          if (x.userId !== $i?.id) {
            os.alert({
              type: "warning",
              text: i18n.ts._reversi.gameCanceled
            });
            router.push("/reversi");
          }
        });
      }
    }
    // 通信を取りこぼした場合の救済
    useInterval(async () => {
      if (game.value == null) return;
      if (game.value.isStarted) return;
      const _game = await misskeyApi("reversi/show-game", { gameId: props.gameId });
      if (_game.isStarted) {
        start(_game);
      } else {
        game.value = _game;
      }
    }, 1e3 * 10, {
      immediate: false,
      afterMounted: true
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
      title: "Reversi",
      icon: "ti ti-device-gamepad"
    }));
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      return game.value == null || !game.value.isEnded && connection.value == null ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createVNode(_component_MkLoading)])) : !game.value.isStarted ? (_openBlock(), _createBlock(GameSetting, {
        key: 1,
        game: game.value,
        connection: connection.value,
        shareWhenStart: shareWhenStart.value,
        "onUpdate:shareWhenStart": _cache[0] || (_cache[0] = ($event) => shareWhenStart.value = $event)
      }, null, 8, [
        "game",
        "connection",
        "shareWhenStart"
      ])) : (_openBlock(), _createBlock(GameBoard, {
        key: 2,
        game: game.value,
        connection: connection.value
      }, null, 8, ["game", "connection"]));
    };
  }
};
