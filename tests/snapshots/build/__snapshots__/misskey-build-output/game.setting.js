import { useModel as _useModel } from "vue";
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = { style: "font-size: 1.5em; text-align: center;" };
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-dice" });
import { computed, watch, ref, onUnmounted } from "vue";
import * as Reversi from "misskey-reversi";
import { i18n } from "@/i18n.js";
import { $i } from "@/i.js";
import { deepClone } from "@/utility/clone.js";
import MkButton from "@/components/MkButton.vue";
import MkRadios from "@/components/MkRadios.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkFolder from "@/components/MkFolder.vue";
import * as os from "@/os.js";
import { useRouter } from "@/router.js";
export default {
  __name: "game.setting",
  props: {
    game: {
      type: null,
      required: true
    },
    connection: {
      type: null,
      required: true
    },
    "shareWhenStart": { default: false }
  },
  emits: ["update:shareWhenStart"],
  setup(__props) {
    const props = __props;
    const shareWhenStart = _useModel(__props, "shareWhenStart");
    const router = useRouter();
    const mapCategories = Array.from(new Set(Object.values(Reversi.maps).map((x) => x.category)));
    const game = ref(deepClone(props.game));
    const gameTurnOptionsDef = [
      {
        value: 5,
        label: "5" + i18n.ts._time.second
      },
      {
        value: 10,
        label: "10" + i18n.ts._time.second
      },
      {
        value: 30,
        label: "30" + i18n.ts._time.second
      },
      {
        value: 60,
        label: "60" + i18n.ts._time.second
      },
      {
        value: 90,
        label: "90" + i18n.ts._time.second
      },
      {
        value: 120,
        label: "120" + i18n.ts._time.second
      },
      {
        value: 180,
        label: "180" + i18n.ts._time.second
      },
      {
        value: 3600,
        label: "3600" + i18n.ts._time.second
      }
    ];
    const mapName = computed(() => {
      if (game.value.map == null) return "Random";
      const found = Object.values(Reversi.maps).find((x) => x.data.join("") === game.value.map.join(""));
      return found ? found.name : "-Custom-";
    });
    const isReady = computed(() => {
      if (game.value.user1Id === $i?.id && game.value.user1Ready) return true;
      if (game.value.user2Id === $i?.id && game.value.user2Ready) return true;
      return false;
    });
    const isOpReady = computed(() => {
      if (game.value.user1Id !== $i?.id && game.value.user1Ready) return true;
      if (game.value.user2Id !== $i?.id && game.value.user2Ready) return true;
      return false;
    });
    const opponentHasSettingsChanged = ref(false);
    watch(() => game.value.bw, () => {
      updateSettings("bw");
    });
    watch(() => game.value.timeLimitForEachTurn, () => {
      updateSettings("timeLimitForEachTurn");
    });
    function chooseMap(ev) {
      const menu = [];
      for (const c of mapCategories) {
        const maps = Object.values(Reversi.maps).filter((x) => x.category === c);
        if (maps.length === 0) continue;
        if (c != null) {
          menu.push({
            type: "label",
            text: c
          });
        }
        for (const m of maps) {
          menu.push({
            text: m.name,
            action: () => {
              game.value.map = m.data;
              updateSettings("map");
            }
          });
        }
      }
      os.popupMenu(menu, ev.currentTarget ?? ev.target);
    }
    async function cancel() {
      const { canceled } = await os.confirm({
        type: "warning",
        text: i18n.ts.areYouSure
      });
      if (canceled) return;
      props.connection.send("cancel", {});
      router.push("/reversi");
    }
    function ready() {
      props.connection.send("ready", true);
      opponentHasSettingsChanged.value = false;
    }
    function unready() {
      props.connection.send("ready", false);
    }
    function onChangeReadyStates(states) {
      game.value.user1Ready = states.user1;
      game.value.user2Ready = states.user2;
    }
    function updateSettings(key) {
      props.connection.send("updateSettings", {
        key,
        value: game.value[key]
      });
    }
    function onUpdateSettings({ userId, key, value }) {
      if (userId === $i?.id) return;
      if (game.value[key] === value) return;
      game.value[key] = value;
      if (isReady.value) {
        opponentHasSettingsChanged.value = true;
        unready();
      }
    }
    function onMapCellClick(pos, pixel) {
      const x = pos % game.value.map[0].length;
      const y = Math.floor(pos / game.value.map[0].length);
      const newPixel = pixel === " " ? "-" : pixel === "-" ? "b" : pixel === "b" ? "w" : " ";
      const line = game.value.map[y].split("");
      line[x] = newPixel;
      game.value.map[y] = line.join("");
      updateSettings("map");
    }
    props.connection.on("changeReadyStates", onChangeReadyStates);
    props.connection.on("updateSettings", onUpdateSettings);
    onUnmounted(() => {
      props.connection.off("changeReadyStates", onChangeReadyStates);
      props.connection.off("updateSettings", onUpdateSettings);
    });
    return (_ctx, _cache) => {
      const _component_MkUserName = _resolveComponent("MkUserName");
      const _component_I18n = _resolveComponent("I18n");
      const _component_MkEllipsis = _resolveComponent("MkEllipsis");
      const _component_MkStickyContainer = _resolveComponent("MkStickyContainer");
      return _openBlock(), _createBlock(_component_MkStickyContainer, null, {
        footer: _withCtx(() => [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.footer) },
          [_createElementVNode("div", {
            class: "_spacer",
            style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 16px;"
          }, [_createElementVNode("div", {
            style: "text-align: center;",
            class: "_gaps_s"
          }, [
            opponentHasSettingsChanged.value ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                style: "color: var(--MI_THEME-warn);"
              },
              _toDisplayString(_unref(i18n).ts._reversi.opponentHasSettingsChanged),
              1
              /* TEXT */
            )) : _createCommentVNode("v-if", true),
            _createElementVNode("div", null, [
              isReady.value && isOpReady.value ? (_openBlock(), _createElementBlock(
                _Fragment,
                { key: 0 },
                [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._reversi.thisGameIsStartedSoon),
                  1
                  /* TEXT */
                ), _createVNode(_component_MkEllipsis)],
                64
                /* STABLE_FRAGMENT */
              )) : _createCommentVNode("v-if", true),
              isReady.value && !isOpReady.value ? (_openBlock(), _createElementBlock(
                _Fragment,
                { key: 0 },
                [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._reversi.waitingForOther),
                  1
                  /* TEXT */
                ), _createVNode(_component_MkEllipsis)],
                64
                /* STABLE_FRAGMENT */
              )) : _createCommentVNode("v-if", true),
              !isReady.value && isOpReady.value ? (_openBlock(), _createElementBlock(
                _Fragment,
                { key: 0 },
                [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._reversi.waitingForMe),
                  1
                  /* TEXT */
                )],
                64
                /* STABLE_FRAGMENT */
              )) : _createCommentVNode("v-if", true),
              !isReady.value && !isOpReady.value ? (_openBlock(), _createElementBlock(
                _Fragment,
                { key: 0 },
                [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._reversi.waitingBoth),
                  1
                  /* TEXT */
                ), _createVNode(_component_MkEllipsis)],
                64
                /* STABLE_FRAGMENT */
              )) : _createCommentVNode("v-if", true)
            ]),
            _createElementVNode("div", { class: "_buttonsCenter" }, [
              _createVNode(MkButton, {
                rounded: "",
                danger: "",
                onClick: cancel
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.cancel),
                  1
                  /* TEXT */
                )]),
                _: 1
              }),
              !isReady.value ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                rounded: "",
                primary: "",
                onClick: ready
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._reversi.ready),
                  1
                  /* TEXT */
                )]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              isReady.value ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                rounded: "",
                onClick: unready
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._reversi.cancelReady),
                  1
                  /* TEXT */
                )]),
                _: 1
              })) : _createCommentVNode("v-if", true)
            ]),
            _createElementVNode("div", null, [_createVNode(MkSwitch, {
              modelValue: shareWhenStart.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => shareWhenStart.value = $event)
            }, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._reversi.shareToTlTheGameWhenStart),
                1
                /* TEXT */
              )]),
              _: 1
            }, 8, ["modelValue"])])
          ])])],
          2
          /* CLASS */
        )]),
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 600px;"
        }, [_createElementVNode("div", { style: "text-align: center;" }, [
          _createElementVNode("b", null, [_createVNode(_component_MkUserName, { user: game.value.user1 }, null, 8, ["user"])]),
          _createTextVNode(" vs "),
          _createElementVNode("b", null, [_createVNode(_component_MkUserName, { user: game.value.user2 }, null, 8, ["user"])])
        ]), _createElementVNode(
          "div",
          { class: _normalizeClass({ [_ctx.$style.disallow]: isReady.value }) },
          [_createElementVNode(
            "div",
            { class: _normalizeClass(["_gaps", { [_ctx.$style.disallowInner]: isReady.value }]) },
            [_createElementVNode(
              "div",
              _hoisted_1,
              _toDisplayString(_unref(i18n).ts._reversi.gameSettings),
              1
              /* TEXT */
            ), game.value.noIrregularRules ? (_openBlock(), _createElementBlock(
              "div",
              { key: 0 },
              _toDisplayString(_unref(i18n).ts._reversi.disallowIrregularRules),
              1
              /* TEXT */
            )) : (_openBlock(), _createElementBlock(
              _Fragment,
              { key: 1 },
              [
                _createElementVNode("div", { class: "_panel" }, [_createElementVNode("div", { style: "display: flex; align-items: center; padding: 16px; border-bottom: solid 1px var(--MI_THEME-divider);" }, [_createElementVNode(
                  "div",
                  null,
                  _toDisplayString(mapName.value),
                  1
                  /* TEXT */
                ), _createVNode(MkButton, {
                  style: "margin-left: auto;",
                  onClick: chooseMap
                }, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._reversi.chooseBoard),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]), _createElementVNode("div", { style: "padding: 16px;" }, [game.value.map == null ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_hoisted_2])) : (_openBlock(), _createElementBlock(
                  "div",
                  {
                    key: 1,
                    class: _normalizeClass(_ctx.$style.board),
                    style: _normalizeStyle({
                      "grid-template-rows": `repeat(${game.value.map.length}, 1fr)`,
                      "grid-template-columns": `repeat(${game.value.map[0].length}, 1fr)`
                    })
                  },
                  [(_openBlock(true), _createElementBlock(
                    _Fragment,
                    null,
                    _renderList(game.value.map.join(""), (x, i) => {
                      return _openBlock(), _createElementBlock("div", {
                        class: _normalizeClass([_ctx.$style.boardCell, { [_ctx.$style.boardCellNone]: x == " " }]),
                        onClick: ($event) => onMapCellClick(i, x)
                      }, [x === "b" || x === "w" ? (_openBlock(), _createElementBlock(
                        "i",
                        {
                          key: 0,
                          style: "pointer-events: none; user-select: none;",
                          class: _normalizeClass(x === "b" ? "ti ti-circle-filled" : "ti ti-circle")
                        },
                        null,
                        2
                        /* CLASS */
                      )) : _createCommentVNode("v-if", true)], 10, ["onClick"]);
                    }),
                    256
                    /* UNKEYED_FRAGMENT */
                  ))],
                  6
                  /* CLASS, STYLE */
                ))])]),
                _createVNode(MkFolder, { defaultOpen: true }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._reversi.blackOrWhite),
                    1
                    /* TEXT */
                  )]),
                  default: _withCtx(() => [_createVNode(MkRadios, {
                    options: [
                      {
                        value: "random",
                        label: _unref(i18n).ts.random
                      },
                      {
                        value: "1",
                        slotId: "user1"
                      },
                      {
                        value: "2",
                        slotId: "user2"
                      }
                    ],
                    modelValue: game.value.bw,
                    "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => game.value.bw = $event)
                  }, {
                    "option-user1": _withCtx(() => [_createVNode(_component_I18n, {
                      src: _unref(i18n).ts._reversi.blackIs,
                      tag: "span"
                    }, {
                      name: _withCtx(() => [_createElementVNode("b", null, [_createVNode(_component_MkUserName, { user: game.value.user1 }, null, 8, ["user"])])]),
                      _: 1
                    }, 8, ["src"])]),
                    "option-user2": _withCtx(() => [_createVNode(_component_I18n, {
                      src: _unref(i18n).ts._reversi.blackIs,
                      tag: "span"
                    }, {
                      name: _withCtx(() => [_createElementVNode("b", null, [_createVNode(_component_MkUserName, { user: game.value.user2 }, null, 8, ["user"])])]),
                      _: 1
                    }, 8, ["src"])]),
                    _: 1
                  }, 8, ["options", "modelValue"])]),
                  _: 1
                }, 8, ["defaultOpen"]),
                _createVNode(MkFolder, { defaultOpen: true }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._reversi.timeLimitForEachTurn),
                    1
                    /* TEXT */
                  )]),
                  suffix: _withCtx(() => [_createTextVNode(
                    _toDisplayString(game.value.timeLimitForEachTurn) + _toDisplayString(_unref(i18n).ts._time.second),
                    1
                    /* TEXT */
                  )]),
                  default: _withCtx(() => [_createVNode(MkRadios, {
                    options: _unref(gameTurnOptionsDef),
                    modelValue: game.value.timeLimitForEachTurn,
                    "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => game.value.timeLimitForEachTurn = $event)
                  }, null, 8, ["options", "modelValue"])]),
                  _: 1
                }, 8, ["defaultOpen"]),
                _createVNode(MkFolder, { defaultOpen: true }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._reversi.rules),
                    1
                    /* TEXT */
                  )]),
                  default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [
                    _createVNode(MkSwitch, {
                      "onUpdate:modelValue": [($event) => updateSettings("isLlotheo"), ($event) => game.value.isLlotheo = $event],
                      modelValue: game.value.isLlotheo
                    }, {
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._reversi.isLlotheo),
                        1
                        /* TEXT */
                      )]),
                      _: 1
                    }, 8, ["modelValue"]),
                    _createVNode(MkSwitch, {
                      "onUpdate:modelValue": [($event) => updateSettings("loopedBoard"), ($event) => game.value.loopedBoard = $event],
                      modelValue: game.value.loopedBoard
                    }, {
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._reversi.loopedMap),
                        1
                        /* TEXT */
                      )]),
                      _: 1
                    }, 8, ["modelValue"]),
                    _createVNode(MkSwitch, {
                      "onUpdate:modelValue": [($event) => updateSettings("canPutEverywhere"), ($event) => game.value.canPutEverywhere = $event],
                      modelValue: game.value.canPutEverywhere
                    }, {
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._reversi.canPutEverywhere),
                        1
                        /* TEXT */
                      )]),
                      _: 1
                    }, 8, ["modelValue"])
                  ])]),
                  _: 1
                }, 8, ["defaultOpen"])
              ],
              64
              /* STABLE_FRAGMENT */
            ))],
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        )])]),
        _: 1
      });
    };
  }
};
