import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("img", { src: "/client-assets/drop-and-fusion/logo.png", style: "display: block; max-width: 100%; max-height: 200px; margin: auto;" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-music" })
const _hoisted_3 = { style: "margin-left: auto;" }
const _hoisted_4 = { style: "font-weight: bold;" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("b", null, "Credit")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("div", null, "Ai-chan illustration: @poteriri@misskey.io")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("div", null, "BGM: @ys@misskey.design")
import { computed, ref, watch } from 'vue'
import * as Misskey from 'misskey-js'
import XGame from './drop-and-fusion.game.vue'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import MkSelect from '@/components/MkSelect.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import { misskeyApiGet } from '@/utility/misskey-api.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'drop-and-fusion',
  setup(__props) {

const {
	model: gameMode,
	def: gameModeDef,
} = useMkSelect({
	items: [
		{ label: 'NORMAL', value: 'normal' },
		{ label: 'SQUARE', value: 'square' },
		{ label: 'YEN', value: 'yen' },
		{ label: 'SWEETS', value: 'sweets' },
		//{ label: 'SPACE', value: 'space' },
	],
	initialValue: 'normal',
});
const gameStarted = ref(false);
const mute = ref(false);
const ranking = ref<Misskey.entities.BubbleGameRankingResponse | null>(null);
watch(gameMode, async () => {
	ranking.value = await misskeyApiGet('bubble-game/ranking', { gameMode: gameMode.value });
}, { immediate: true });
function getScoreUnit(gameMode: string) {
	return gameMode === 'normal' ? 'pt' :
		gameMode === 'square' ? 'pt' :
		gameMode === 'yen' ? '円' :
		gameMode === 'sweets' ? 'kcal' :
		gameMode === 'space' ? 'pt' :
		'' as never;
}
async function start() {
	gameStarted.value = true;
}
function onGameEnd() {
	gameStarted.value = false;
}
definePage(() => ({
	title: i18n.ts.bubbleGame,
	icon: 'ti ti-device-gamepad',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")

  return (_openBlock(), _createBlock(_Transition, {
      enterActiveClass: _ctx.$style.transition_zoom_enterActive,
      leaveActiveClass: _ctx.$style.transition_zoom_leaveActive,
      enterFromClass: _ctx.$style.transition_zoom_enterFrom,
      leaveToClass: _ctx.$style.transition_zoom_leaveTo,
      moveClass: _ctx.$style.transition_zoom_move,
      mode: "out-in"
    }, {
      default: _withCtx(() => [
        (!gameStarted.value)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_spacer",
            style: "--MI_SPACER-w: 800px;"
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.root)
            }, [
              _createElementVNode("div", { class: "_gaps" }, [
                _createElementVNode("div", {
                  class: "_woodenFrame",
                  style: "text-align: center;"
                }, [
                  _createElementVNode("div", { class: "_woodenFrameInner" }, [
                    _hoisted_1
                  ])
                ]),
                _createElementVNode("div", {
                  class: "_woodenFrame",
                  style: "text-align: center;"
                }, [
                  _createElementVNode("div", { class: "_woodenFrameInner" }, [
                    _createElementVNode("div", {
                      class: "_gaps",
                      style: "padding: 16px;"
                    }, [
                      _createVNode(MkSelect, {
                        items: _unref(gameModeDef),
                        modelValue: _unref(gameMode),
                        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((gameMode).value = $event))
                      }, null, 8 /* PROPS */, ["items", "modelValue"]),
                      _createVNode(MkButton, {
                        primary: "",
                        gradate: "",
                        large: "",
                        rounded: "",
                        inline: "",
                        onClick: start
                      }, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.start), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ])
                  ]),
                  _createElementVNode("div", { class: "_woodenFrameInner" }, [
                    _createElementVNode("div", {
                      class: "_gaps",
                      style: "padding: 16px;"
                    }, [
                      _createElementVNode("div", { style: "font-size: 90%;" }, [
                        _hoisted_2,
                        _createTextVNode(" " + _toDisplayString(_unref(i18n).ts.soundWillBePlayed), 1 /* TEXT */)
                      ]),
                      _createVNode(MkSwitch, {
                        modelValue: mute.value,
                        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((mute).value = $event))
                      }, {
                        label: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.mute), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }, 8 /* PROPS */, ["modelValue"])
                    ])
                  ])
                ]),
                _createElementVNode("div", { class: "_woodenFrame" }, [
                  _createElementVNode("div", { class: "_woodenFrameInner" }, [
                    _createElementVNode("div", {
                      class: "_gaps_s",
                      style: "padding: 16px;"
                    }, [
                      _createElementVNode("div", null, [
                        _createElementVNode("b", null, _toDisplayString(_unref(i18n).tsx.lastNDays({ n: 7 })) + " " + _toDisplayString(_unref(i18n).ts.ranking), 1 /* TEXT */),
                        _createTextVNode(" (" + _toDisplayString(_unref(gameMode).toUpperCase()) + ")", 1 /* TEXT */)
                      ]),
                      (ranking.value)
                        ? (_openBlock(), _createElementBlock("div", {
                          key: 0,
                          class: "_gaps_s"
                        }, [
                          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(ranking.value, (r) => {
                            return (_openBlock(), _createElementBlock("div", {
                              key: r.id,
                              class: _normalizeClass(_ctx.$style.rankingRecord)
                            }, [
                              (r.user)
                                ? (_openBlock(), _createBlock(_component_MkAvatar, {
                                  key: 0,
                                  link: true,
                                  style: "width: 24px; height: 24px; margin-right: 4px;",
                                  user: r.user
                                }, null, 8 /* PROPS */, ["link", "user"]))
                                : _createCommentVNode("v-if", true),
                              (r.user)
                                ? (_openBlock(), _createBlock(_component_MkUserName, {
                                  key: 0,
                                  user: r.user,
                                  nowrap: true
                                }, null, 8 /* PROPS */, ["user", "nowrap"]))
                                : _createCommentVNode("v-if", true),
                              _createElementVNode("b", _hoisted_3, _toDisplayString(r.score.toLocaleString()) + " " + _toDisplayString(getScoreUnit(_unref(gameMode))), 1 /* TEXT */)
                            ]))
                          }), 128 /* KEYED_FRAGMENT */))
                        ]))
                        : (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts.loading), 1 /* TEXT */))
                    ])
                  ])
                ]),
                _createElementVNode("div", { class: "_woodenFrame" }, [
                  _createElementVNode("div", {
                    class: "_woodenFrameInner",
                    style: "padding: 16px;"
                  }, [
                    _createElementVNode("div", _hoisted_4, _toDisplayString(_unref(i18n).ts._bubbleGame.howToPlay), 1 /* TEXT */),
                    _createElementVNode("ol", null, [
                      _createElementVNode("li", null, _toDisplayString(_unref(i18n).ts._bubbleGame._howToPlay.section1), 1 /* TEXT */),
                      _createElementVNode("li", null, _toDisplayString(_unref(i18n).ts._bubbleGame._howToPlay.section2), 1 /* TEXT */),
                      _createElementVNode("li", null, _toDisplayString(_unref(i18n).ts._bubbleGame._howToPlay.section3), 1 /* TEXT */)
                    ])
                  ])
                ]),
                _createElementVNode("div", { class: "_woodenFrame" }, [
                  _createElementVNode("div", { class: "_woodenFrameInner" }, [
                    _createElementVNode("div", {
                      class: "_gaps_s",
                      style: "padding: 16px;"
                    }, [
                      _createElementVNode("div", null, [
                        _hoisted_5
                      ]),
                      _createElementVNode("div", null, [
                        _hoisted_6,
                        _hoisted_7
                      ])
                    ])
                  ])
                ])
              ])
            ])
          ]))
          : (_openBlock(), _createBlock(XGame, {
            key: 1,
            gameMode: _unref(gameMode),
            mute: mute.value,
            onEnd: onGameEnd
          }, null, 8 /* PROPS */, ["gameMode", "mute"]))
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass"]))
}
}

})
