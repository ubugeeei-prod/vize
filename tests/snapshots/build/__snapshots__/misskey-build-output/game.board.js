import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("span", null, " vs ");
const _hoisted_2 = { style: "margin-left: 1em; opacity: 0.7;" };
const _hoisted_3 = { style: "display: inline-block; font-weight: bold; animation: global-tada 1s linear infinite both;" };
const _hoisted_4 = { style: "margin-left: 1em; opacity: 0.7;" };
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevrons-left" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-left" });
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-right" });
const _hoisted_8 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevrons-right" });
const _hoisted_9 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-player-play" });
const _hoisted_10 = { style: "margin-right: 8px;" };
const _hoisted_11 = /* @__PURE__ */ _createElementVNode("div", null, " vs ");
const _hoisted_12 = { style: "margin-right: 8px;" };
const _hoisted_13 = /* @__PURE__ */ _createElementVNode("img", {
  src: "/client-assets/reversi/logo.png",
  style: "display: block; max-width: 100%; width: 200px; margin: auto;"
});
import { computed, onActivated, onDeactivated, onMounted, onUnmounted, ref, shallowRef, triggerRef, watch } from "vue";
import * as Reversi from "misskey-reversi";
import { useInterval } from "@@/js/use-interval.js";
import { url } from "@@/js/config.js";
import MkButton from "@/components/MkButton.vue";
import MkFolder from "@/components/MkFolder.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import { deepClone } from "@/utility/clone.js";
import { $i } from "@/i.js";
import { i18n } from "@/i18n.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { userPage } from "@/filters/user.js";
import * as sound from "@/utility/sound.js";
import * as os from "@/os.js";
import { confetti } from "@/utility/confetti.js";
import { genId } from "@/utility/id.js";
const TIMER_INTERVAL_SEC = 3;
export default {
  __name: "game.board",
  props: {
    game: {
      type: null,
      required: true
    },
    connection: {
      type: null,
      required: false
    }
  },
  setup(__props) {
    const props = __props;
    const showBoardLabels = ref(false);
    const useAvatarAsStone = ref(true);
    const autoplaying = ref(false);
    // eslint-disable-next-line vue/no-setup-props-reactivity-loss
    const game = ref(deepClone(props.game));
    const logPos = ref(game.value.logs.length);
    const engine = shallowRef(Reversi.Serializer.restoreGame({
      map: game.value.map,
      isLlotheo: game.value.isLlotheo,
      canPutEverywhere: game.value.canPutEverywhere,
      loopedBoard: game.value.loopedBoard,
      logs: game.value.logs
    }));
    const iAmPlayer = computed(() => {
      return game.value.user1Id === $i?.id || game.value.user2Id === $i?.id;
    });
    const myColor = computed(() => {
      if (!iAmPlayer.value) return null;
      if (game.value.user1Id === $i?.id && game.value.black === 1) return true;
      if (game.value.user2Id === $i?.id && game.value.black === 2) return true;
      return false;
    });
    const opColor = computed(() => {
      if (!iAmPlayer.value) return null;
      return !myColor.value;
    });
    const blackUser = computed(() => {
      return game.value.black === 1 ? game.value.user1 : game.value.user2;
    });
    const whiteUser = computed(() => {
      return game.value.black === 1 ? game.value.user2 : game.value.user1;
    });
    const turnUser = computed(() => {
      if (engine.value.turn === true) {
        return game.value.black === 1 ? game.value.user1 : game.value.user2;
      } else if (engine.value.turn === false) {
        return game.value.black === 1 ? game.value.user2 : game.value.user1;
      } else {
        return null;
      }
    });
    const isMyTurn = computed(() => {
      if (!iAmPlayer.value) return false;
      const u = turnUser.value;
      if (u == null) return false;
      return u.id === $i?.id;
    });
    const cellsStyle = computed(() => {
      return {
        "grid-template-rows": `repeat(${game.value.map.length}, 1fr)`,
        "grid-template-columns": `repeat(${game.value.map[0].length}, 1fr)`
      };
    });
    watch(logPos, (v) => {
      if (!game.value.isEnded) return;
      engine.value = Reversi.Serializer.restoreGame({
        map: game.value.map,
        isLlotheo: game.value.isLlotheo,
        canPutEverywhere: game.value.canPutEverywhere,
        loopedBoard: game.value.loopedBoard,
        logs: game.value.logs.slice(0, v)
      });
    });
    if (game.value.isStarted && !game.value.isEnded) {
      useInterval(() => {
        if (game.value.isEnded) return;
        const crc32 = engine.value.calcCrc32();
        if (_DEV_) console.log("crc32", crc32);
        misskeyApi("reversi/verify", {
          gameId: game.value.id,
          crc32: crc32.toString()
        }).then((res) => {
          if (res.desynced) {
            if (_DEV_) console.log("resynced");
            restoreGame(res.game);
          }
        });
      }, 1e4, {
        immediate: false,
        afterMounted: true
      });
    }
    const appliedOps = [];
    function putStone(pos) {
      if (game.value.isEnded) return;
      if (!iAmPlayer.value) return;
      if (!isMyTurn.value) return;
      if (!engine.value.canPut(myColor.value, pos)) return;
      engine.value.putStone(pos);
      triggerRef(engine);
      sound.playUrl("/client-assets/reversi/put.mp3", {
        volume: 1,
        playbackRate: 1
      });
      const id = genId();
      props.connection.send("putStone", {
        pos,
        id
      });
      appliedOps.push(id);
      myTurnTimerRmain.value = game.value.timeLimitForEachTurn;
      opTurnTimerRmain.value = game.value.timeLimitForEachTurn;
      checkEnd();
    }
    const myTurnTimerRmain = ref(game.value.timeLimitForEachTurn);
    const opTurnTimerRmain = ref(game.value.timeLimitForEachTurn);
    if (!props.game.isEnded) {
      useInterval(() => {
        if (myTurnTimerRmain.value > 0) {
          myTurnTimerRmain.value = Math.max(0, myTurnTimerRmain.value - TIMER_INTERVAL_SEC);
        }
        if (opTurnTimerRmain.value > 0) {
          opTurnTimerRmain.value = Math.max(0, opTurnTimerRmain.value - TIMER_INTERVAL_SEC);
        }
        if (iAmPlayer.value) {
          if (isMyTurn.value && myTurnTimerRmain.value === 0 || !isMyTurn.value && opTurnTimerRmain.value === 0) {
            props.connection.send("claimTimeIsUp", {});
          }
        }
      }, TIMER_INTERVAL_SEC * 1e3, {
        immediate: false,
        afterMounted: true
      });
    }
    async function onStreamLog(log) {
      game.value.logs = Reversi.Serializer.serializeLogs([...Reversi.Serializer.deserializeLogs(game.value.logs), log]);
      logPos.value++;
      if (log.id == null || !appliedOps.includes(log.id)) {
        switch (log.operation) {
          case "put": {
            sound.playUrl("/client-assets/reversi/put.mp3", {
              volume: 1,
              playbackRate: 1
            });
            if (log.player !== engine.value.turn) {
              const _game = await misskeyApi("reversi/show-game", { gameId: props.game.id });
              restoreGame(_game);
              return;
            }
            engine.value.putStone(log.pos);
            triggerRef(engine);
            myTurnTimerRmain.value = game.value.timeLimitForEachTurn;
            opTurnTimerRmain.value = game.value.timeLimitForEachTurn;
            checkEnd();
            break;
          }
          default: break;
        }
      }
    }
    function onStreamEnded(x) {
      game.value = deepClone(x.game);
      if (game.value.winnerId === $i?.id) {
        confetti({ duration: 1e3 * 3 });
        sound.playUrl("/client-assets/reversi/win.mp3", {
          volume: 1,
          playbackRate: 1
        });
      } else {
        sound.playUrl("/client-assets/reversi/lose.mp3", {
          volume: 1,
          playbackRate: 1
        });
      }
    }
    function checkEnd() {
      game.value.isEnded = engine.value.isEnded;
      if (game.value.isEnded) {
        if (engine.value.winner === true) {
          game.value.winnerId = game.value.black === 1 ? game.value.user1Id : game.value.user2Id;
          game.value.winner = game.value.black === 1 ? game.value.user1 : game.value.user2;
        } else if (engine.value.winner === false) {
          game.value.winnerId = game.value.black === 1 ? game.value.user2Id : game.value.user1Id;
          game.value.winner = game.value.black === 1 ? game.value.user2 : game.value.user1;
        } else {
          game.value.winnerId = null;
          game.value.winner = null;
        }
      }
    }
    function restoreGame(_game) {
      game.value = deepClone(_game);
      engine.value = Reversi.Serializer.restoreGame({
        map: game.value.map,
        isLlotheo: game.value.isLlotheo,
        canPutEverywhere: game.value.canPutEverywhere,
        loopedBoard: game.value.loopedBoard,
        logs: game.value.logs
      });
      logPos.value = game.value.logs.length;
      checkEnd();
    }
    async function surrender() {
      const { canceled } = await os.confirm({
        type: "warning",
        text: i18n.ts.areYouSure
      });
      if (canceled) return;
      misskeyApi("reversi/surrender", { gameId: game.value.id });
    }
    function autoplay() {
      autoplaying.value = true;
      logPos.value = 0;
      const logs = Reversi.Serializer.deserializeLogs(game.value.logs);
      window.setTimeout(() => {
        logPos.value = 1;
        let i = 1;
        let previousLog = logs[0];
        const tick = () => {
          const log = logs[i];
          const time = log.time - previousLog.time;
          window.setTimeout(() => {
            i++;
            logPos.value++;
            previousLog = log;
            if (i < logs.length) {
              tick();
            } else {
              autoplaying.value = false;
            }
          }, time);
        };
        tick();
      }, 1e3);
    }
    function share() {
      os.post({
        initialText: `#MisskeyReversi\n${url}/reversi/g/${game.value.id}`,
        instant: true
      });
    }
    onMounted(() => {
      if (props.connection != null) {
        props.connection.on("log", onStreamLog);
        props.connection.on("ended", onStreamEnded);
      }
    });
    onActivated(() => {
      if (props.connection != null) {
        props.connection.on("log", onStreamLog);
        props.connection.on("ended", onStreamEnded);
      }
    });
    onDeactivated(() => {
      if (props.connection != null) {
        props.connection.off("log", onStreamLog);
        props.connection.off("ended", onStreamEnded);
      }
    });
    onUnmounted(() => {
      if (props.connection != null) {
        props.connection.off("log", onStreamLog);
        props.connection.off("ended", onStreamEnded);
      }
    });
    return (_ctx, _cache) => {
      const _component_MkAvatar = _resolveComponent("MkAvatar");
      const _component_Mfm = _resolveComponent("Mfm");
      const _component_MkEllipsis = _resolveComponent("MkEllipsis");
      const _component_MkUserName = _resolveComponent("MkUserName");
      const _component_MkA = _resolveComponent("MkA");
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock("div", {
        class: "_spacer",
        style: "--MI_SPACER-w: 500px;"
      }, [_createElementVNode(
        "div",
        { class: _normalizeClass(["_gaps", _ctx.$style.root]) },
        [
          _createElementVNode("div", { style: "display: flex; align-items: center; justify-content: center; gap: 10px;" }, [
            _createElementVNode(
              "span",
              null,
              "(" + _toDisplayString(_unref(i18n).ts._reversi.black) + ")",
              1
              /* TEXT */
            ),
            _createVNode(_component_MkAvatar, {
              style: "width: 32px; height: 32px;",
              user: blackUser.value,
              showIndicator: true
            }, null, 8, ["user", "showIndicator"]),
            _hoisted_1,
            _createVNode(_component_MkAvatar, {
              style: "width: 32px; height: 32px;",
              user: whiteUser.value,
              showIndicator: true
            }, null, 8, ["user", "showIndicator"]),
            _createElementVNode(
              "span",
              null,
              "(" + _toDisplayString(_unref(i18n).ts._reversi.white) + ")",
              1
              /* TEXT */
            )
          ]),
          _createElementVNode("div", { style: "overflow: clip; line-height: 28px;" }, [
            !iAmPlayer.value && !game.value.isEnded && turnUser.value ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createVNode(_component_Mfm, {
              key: "turn:" + turnUser.value.id,
              text: _unref(i18n).tsx._reversi.turnOf({ name: turnUser.value.name ?? turnUser.value.username }),
              plain: true,
              customEmojis: turnUser.value.emojis
            }, null, 8, [
              "text",
              "plain",
              "customEmojis"
            ]), _createVNode(_component_MkEllipsis)])) : _createCommentVNode("v-if", true),
            logPos.value !== game.value.logs.length && turnUser.value ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createVNode(_component_Mfm, {
              key: "past-turn-of:" + turnUser.value.id,
              text: _unref(i18n).tsx._reversi.pastTurnOf({ name: turnUser.value.name ?? turnUser.value.username }),
              plain: true,
              customEmojis: turnUser.value.emojis
            }, null, 8, [
              "text",
              "plain",
              "customEmojis"
            ])])) : _createCommentVNode("v-if", true),
            iAmPlayer.value && !game.value.isEnded && !isMyTurn.value ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts._reversi.opponentTurn),
                1
                /* TEXT */
              ),
              _createVNode(_component_MkEllipsis),
              _createElementVNode(
                "span",
                _hoisted_2,
                "(" + _toDisplayString(_unref(i18n).tsx.remainingN({ n: opTurnTimerRmain.value })) + ")",
                1
                /* TEXT */
              )
            ])) : _createCommentVNode("v-if", true),
            iAmPlayer.value && !game.value.isEnded && isMyTurn.value ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createElementVNode(
              "span",
              _hoisted_3,
              _toDisplayString(_unref(i18n).ts._reversi.myTurn),
              1
              /* TEXT */
            ), _createElementVNode(
              "span",
              _hoisted_4,
              "(" + _toDisplayString(_unref(i18n).tsx.remainingN({ n: myTurnTimerRmain.value })) + ")",
              1
              /* TEXT */
            )])) : _createCommentVNode("v-if", true),
            game.value.isEnded && logPos.value == game.value.logs.length ? (_openBlock(), _createElementBlock("div", { key: 0 }, [game.value.winner ? (_openBlock(), _createElementBlock(
              _Fragment,
              { key: 0 },
              [
                _createVNode(_component_Mfm, {
                  key: "won",
                  text: _unref(i18n).tsx._reversi.won({ name: game.value.winner.name ?? game.value.winner.username }),
                  plain: true,
                  customEmojis: game.value.winner.emojis
                }, null, 8, [
                  "text",
                  "plain",
                  "customEmojis"
                ]),
                game.value.surrenderedUserId != null ? (_openBlock(), _createElementBlock(
                  "span",
                  { key: 0 },
                  " (" + _toDisplayString(_unref(i18n).ts._reversi.surrendered) + ")",
                  1
                  /* TEXT */
                )) : _createCommentVNode("v-if", true),
                game.value.timeoutUserId != null ? (_openBlock(), _createElementBlock(
                  "span",
                  { key: 0 },
                  " (" + _toDisplayString(_unref(i18n).ts._reversi.timeout) + ")",
                  1
                  /* TEXT */
                )) : _createCommentVNode("v-if", true)
              ],
              64
              /* STABLE_FRAGMENT */
            )) : (_openBlock(), _createElementBlock(
              _Fragment,
              { key: 1 },
              [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._reversi.drawn),
                1
                /* TEXT */
              )],
              64
              /* STABLE_FRAGMENT */
            ))])) : _createCommentVNode("v-if", true)
          ]),
          _createElementVNode("div", { class: "_woodenFrame" }, [_createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.boardInner) },
            [
              showBoardLabels.value ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.labelsX)
                },
                [(_openBlock(true), _createElementBlock(
                  _Fragment,
                  null,
                  _renderList(game.value.map[0].length, (i) => {
                    return _openBlock(), _createElementBlock(
                      "span",
                      {
                        key: i,
                        class: _normalizeClass(_ctx.$style.labelsXLabel)
                      },
                      _toDisplayString(String.fromCharCode(64 + i)),
                      3
                      /* TEXT, CLASS */
                    );
                  }),
                  128
                  /* KEYED_FRAGMENT */
                ))],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true),
              _createElementVNode("div", { style: "display: flex;" }, [
                showBoardLabels.value ? (_openBlock(), _createElementBlock(
                  "div",
                  {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.labelsY)
                  },
                  [(_openBlock(true), _createElementBlock(
                    _Fragment,
                    null,
                    _renderList(game.value.map.length, (i) => {
                      return _openBlock(), _createElementBlock(
                        "div",
                        {
                          key: i,
                          class: _normalizeClass(_ctx.$style.labelsYLabel)
                        },
                        _toDisplayString(i),
                        3
                        /* TEXT, CLASS */
                      );
                    }),
                    128
                    /* KEYED_FRAGMENT */
                  ))],
                  2
                  /* CLASS */
                )) : _createCommentVNode("v-if", true),
                _createElementVNode(
                  "div",
                  {
                    class: _normalizeClass(_ctx.$style.boardCells),
                    style: _normalizeStyle(cellsStyle.value)
                  },
                  [(_openBlock(true), _createElementBlock(
                    _Fragment,
                    null,
                    _renderList(engine.value.board, (stone, i) => {
                      return _withDirectives((_openBlock(), _createElementBlock("div", {
                        key: i,
                        class: _normalizeClass([_ctx.$style.boardCell, {
                          [_ctx.$style.boardCell_empty]: stone == null,
                          [_ctx.$style.boardCell_none]: engine.value.map[i] === "null",
                          [_ctx.$style.boardCell_isEnded]: game.value.isEnded,
                          [_ctx.$style.boardCell_myTurn]: !game.value.isEnded && isMyTurn.value,
                          [_ctx.$style.boardCell_can]: turnUser.value ? engine.value.canPut(turnUser.value.id === blackUser.value.id, i) : null,
                          [_ctx.$style.boardCell_prev]: engine.value.prevPos === i
                        }]),
                        onClick: ($event) => putStone(i)
                      }, [_createVNode(_Transition, {
                        enterActiveClass: _ctx.$style.transition_flip_enterActive,
                        leaveActiveClass: _ctx.$style.transition_flip_leaveActive,
                        enterFromClass: _ctx.$style.transition_flip_enterFrom,
                        leaveToClass: _ctx.$style.transition_flip_leaveTo,
                        mode: "default"
                      }, {
                        default: _withCtx(() => [useAvatarAsStone.value ? (_openBlock(), _createElementBlock(
                          _Fragment,
                          { key: 0 },
                          [stone === true ? (_openBlock(), _createElementBlock("img", {
                            key: 0,
                            class: _normalizeClass(_ctx.$style.boardCellStone),
                            src: blackUser.value.avatarUrl ?? undefined
                          }, null, 10, ["src"])) : stone === false ? (_openBlock(), _createElementBlock("img", {
                            key: 1,
                            class: _normalizeClass(_ctx.$style.boardCellStone),
                            src: whiteUser.value.avatarUrl ?? undefined
                          }, null, 10, ["src"])) : _createCommentVNode("v-if", true)],
                          64
                          /* STABLE_FRAGMENT */
                        )) : (_openBlock(), _createElementBlock(
                          _Fragment,
                          { key: 1 },
                          [stone === true ? (_openBlock(), _createElementBlock(
                            "img",
                            {
                              key: 0,
                              class: _normalizeClass(_ctx.$style.boardCellStone),
                              src: "/client-assets/reversi/stone_b.png"
                            },
                            null,
                            2
                            /* CLASS */
                          )) : stone === false ? (_openBlock(), _createElementBlock(
                            "img",
                            {
                              key: 1,
                              class: _normalizeClass(_ctx.$style.boardCellStone),
                              src: "/client-assets/reversi/stone_w.png"
                            },
                            null,
                            2
                            /* CLASS */
                          )) : _createCommentVNode("v-if", true)],
                          64
                          /* STABLE_FRAGMENT */
                        ))]),
                        _: 2
                      }, 1032, [
                        "enterActiveClass",
                        "leaveActiveClass",
                        "enterFromClass",
                        "leaveToClass"
                      ])], 10, ["onClick"])), [[_directive_tooltip, `${String.fromCharCode(65 + engine.value.posToXy(i)[0])}${engine.value.posToXy(i)[1] + 1}`]]);
                    }),
                    128
                    /* KEYED_FRAGMENT */
                  ))],
                  6
                  /* CLASS, STYLE */
                ),
                showBoardLabels.value ? (_openBlock(), _createElementBlock(
                  "div",
                  {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.labelsY)
                  },
                  [(_openBlock(true), _createElementBlock(
                    _Fragment,
                    null,
                    _renderList(game.value.map.length, (i) => {
                      return _openBlock(), _createElementBlock(
                        "div",
                        {
                          key: i,
                          class: _normalizeClass(_ctx.$style.labelsYLabel)
                        },
                        _toDisplayString(i),
                        3
                        /* TEXT, CLASS */
                      );
                    }),
                    128
                    /* KEYED_FRAGMENT */
                  ))],
                  2
                  /* CLASS */
                )) : _createCommentVNode("v-if", true)
              ]),
              showBoardLabels.value ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.labelsX)
                },
                [(_openBlock(true), _createElementBlock(
                  _Fragment,
                  null,
                  _renderList(game.value.map[0].length, (i) => {
                    return _openBlock(), _createElementBlock(
                      "span",
                      {
                        key: i,
                        class: _normalizeClass(_ctx.$style.labelsXLabel)
                      },
                      _toDisplayString(String.fromCharCode(64 + i)),
                      3
                      /* TEXT, CLASS */
                    );
                  }),
                  128
                  /* KEYED_FRAGMENT */
                ))],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true)
            ],
            2
            /* CLASS */
          )]),
          game.value.isEnded ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_panel _gaps_s",
            style: "padding: 16px;"
          }, [
            _createElementVNode(
              "div",
              null,
              _toDisplayString(logPos.value) + " / " + _toDisplayString(game.value.logs.length),
              1
              /* TEXT */
            ),
            !autoplaying.value ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_buttonsCenter"
            }, [
              _createVNode(MkButton, {
                disabled: logPos.value === 0,
                onClick: _cache[0] || (_cache[0] = ($event) => logPos.value = 0)
              }, {
                default: _withCtx(() => [_hoisted_5]),
                _: 1
              }, 8, ["disabled"]),
              _createVNode(MkButton, {
                disabled: logPos.value === 0,
                onClick: _cache[1] || (_cache[1] = ($event) => logPos.value--)
              }, {
                default: _withCtx(() => [_hoisted_6]),
                _: 1
              }, 8, ["disabled"]),
              _createVNode(MkButton, {
                disabled: logPos.value === game.value.logs.length,
                onClick: _cache[2] || (_cache[2] = ($event) => logPos.value++)
              }, {
                default: _withCtx(() => [_hoisted_7]),
                _: 1
              }, 8, ["disabled"]),
              _createVNode(MkButton, {
                disabled: logPos.value === game.value.logs.length,
                onClick: _cache[3] || (_cache[3] = ($event) => logPos.value = game.value.logs.length)
              }, {
                default: _withCtx(() => [_hoisted_8]),
                _: 1
              }, 8, ["disabled"])
            ])) : _createCommentVNode("v-if", true),
            _createVNode(MkButton, {
              style: "margin: auto;",
              disabled: autoplaying.value,
              onClick: _cache[4] || (_cache[4] = ($event) => autoplay())
            }, {
              default: _withCtx(() => [_hoisted_9]),
              _: 1
            }, 8, ["disabled"])
          ])) : _createCommentVNode("v-if", true),
          _createElementVNode("div", {
            class: "_panel",
            style: "padding: 16px;"
          }, [
            _createElementVNode("div", null, [_createElementVNode(
              "b",
              null,
              _toDisplayString(_unref(i18n).tsx._reversi.turnCount({ count: logPos.value })),
              1
              /* TEXT */
            ), _createTextVNode(
              " " + _toDisplayString(_unref(i18n).ts._reversi.black) + ":" + _toDisplayString(engine.value.blackCount) + " " + _toDisplayString(_unref(i18n).ts._reversi.white) + ":" + _toDisplayString(engine.value.whiteCount) + " " + _toDisplayString(_unref(i18n).ts._reversi.total) + ":" + _toDisplayString(engine.value.blackCount + engine.value.whiteCount),
              1
              /* TEXT */
            )]),
            _createElementVNode("div", null, [
              _createElementVNode("div", { style: "display: flex; align-items: center;" }, [
                _createElementVNode(
                  "span",
                  _hoisted_10,
                  "(" + _toDisplayString(_unref(i18n).ts._reversi.black) + ")",
                  1
                  /* TEXT */
                ),
                _createVNode(_component_MkAvatar, {
                  style: "width: 32px; height: 32px; margin-right: 8px;",
                  user: blackUser.value,
                  showIndicator: true
                }, null, 8, ["user", "showIndicator"]),
                _createVNode(_component_MkA, { to: _unref(userPage)(blackUser.value) }, {
                  default: _withCtx(() => [_createVNode(_component_MkUserName, { user: blackUser.value }, null, 8, ["user"])]),
                  _: 1
                }, 8, ["to"])
              ]),
              _hoisted_11,
              _createElementVNode("div", { style: "display: flex; align-items: center;" }, [
                _createElementVNode(
                  "span",
                  _hoisted_12,
                  "(" + _toDisplayString(_unref(i18n).ts._reversi.white) + ")",
                  1
                  /* TEXT */
                ),
                _createVNode(_component_MkAvatar, {
                  style: "width: 32px; height: 32px; margin-right: 8px;",
                  user: whiteUser.value,
                  showIndicator: true
                }, null, 8, ["user", "showIndicator"]),
                _createVNode(_component_MkA, { to: _unref(userPage)(whiteUser.value) }, {
                  default: _withCtx(() => [_createVNode(_component_MkUserName, { user: whiteUser.value }, null, 8, ["user"])]),
                  _: 1
                }, 8, ["to"])
              ])
            ]),
            _createElementVNode("div", null, [
              game.value.isLlotheo ? (_openBlock(), _createElementBlock(
                "p",
                { key: 0 },
                _toDisplayString(_unref(i18n).ts._reversi.isLlotheo),
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true),
              game.value.loopedBoard ? (_openBlock(), _createElementBlock(
                "p",
                { key: 0 },
                _toDisplayString(_unref(i18n).ts._reversi.loopedMap),
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true),
              game.value.canPutEverywhere ? (_openBlock(), _createElementBlock(
                "p",
                { key: 0 },
                _toDisplayString(_unref(i18n).ts._reversi.canPutEverywhere),
                1
                /* TEXT */
              )) : _createCommentVNode("v-if", true)
            ])
          ]),
          _createVNode(MkFolder, null, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.options),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [_createElementVNode("div", {
              class: "_gaps_s",
              style: "text-align: left;"
            }, [_createVNode(MkSwitch, {
              modelValue: showBoardLabels.value,
              "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => showBoardLabels.value = $event)
            }, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._reversi.showBoardLabels),
                1
                /* TEXT */
              )]),
              _: 1
            }, 8, ["modelValue"]), _createVNode(MkSwitch, {
              modelValue: useAvatarAsStone.value,
              "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event) => useAvatarAsStone.value = $event)
            }, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._reversi.useAvatarAsStone),
                1
                /* TEXT */
              )]),
              _: 1
            }, 8, ["modelValue"])])]),
            _: 1
          }),
          _createElementVNode("div", { class: "_buttonsCenter" }, [!game.value.isEnded && iAmPlayer.value ? (_openBlock(), _createBlock(MkButton, {
            key: 0,
            danger: "",
            onClick: surrender
          }, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._reversi.surrender),
              1
              /* TEXT */
            )]),
            _: 1
          })) : _createCommentVNode("v-if", true), _createVNode(MkButton, { onClick: share }, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.share),
              1
              /* TEXT */
            )]),
            _: 1
          })]),
          game.value.isEnded ? (_openBlock(), _createBlock(_component_MkA, {
            key: 0,
            to: `/reversi`
          }, {
            default: _withCtx(() => [_hoisted_13]),
            _: 1
          }, 8, ["to"])) : _createCommentVNode("v-if", true)
        ],
        2
        /* CLASS */
      )]);
    };
  }
};
