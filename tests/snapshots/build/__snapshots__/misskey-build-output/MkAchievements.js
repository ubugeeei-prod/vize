import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import * as Misskey from 'misskey-js'
import { onMounted, ref, computed } from 'vue'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { ACHIEVEMENT_TYPES, ACHIEVEMENT_BADGES, claimAchievement } from '@/utility/achievements.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAchievements',
  props: {
    user: { type: null, required: true },
    withLocked: { type: Boolean, required: false, default: true },
    withDescription: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const props = __props
const achievements = ref<Misskey.entities.UsersAchievementsResponse | null>(null);
const lockedAchievements = computed(() => ACHIEVEMENT_TYPES.filter(x => !(achievements.value ?? []).some(a => a.name === x)));
function _fetch_() {
	misskeyApi('users/achievements', { userId: props.user.id }).then(res => {
		achievements.value = [];
		for (const t of ACHIEVEMENT_TYPES) {
			const a = res.find(x => x.name === t);
			if (a) achievements.value.push(a);
		}
		//achievements = res.sort((a, b) => b.unlockedAt - a.unlockedAt);
	});
}
function clickHere() {
	claimAchievement('clickedClickHere');
	_fetch_();
}
onMounted(() => {
	_fetch_();
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", null, [ (achievements.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.root)
        }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(achievements.value, (achievement) => {
            return (_openBlock(), _createElementBlock("div", {
              key: achievement.name,
              class: _normalizeClass(["_panel", _ctx.$style.achievement])
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.icon)
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass([_ctx.$style.iconFrame, {
  						[_ctx.$style.iconFrame_bronze]: _unref(ACHIEVEMENT_BADGES)[achievement.name].frame === 'bronze',
  						[_ctx.$style.iconFrame_silver]: _unref(ACHIEVEMENT_BADGES)[achievement.name].frame === 'silver',
  						[_ctx.$style.iconFrame_gold]: _unref(ACHIEVEMENT_BADGES)[achievement.name].frame === 'gold',
  						[_ctx.$style.iconFrame_platinum]: _unref(ACHIEVEMENT_BADGES)[achievement.name].frame === 'platinum',
  					}])
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass([_ctx.$style.iconInner]),
                    style: _normalizeStyle({ background: _unref(ACHIEVEMENT_BADGES)[achievement.name].bg ?? '' })
                  }, [
                    _createElementVNode("img", {
                      class: _normalizeClass(_ctx.$style.iconImg),
                      src: _unref(ACHIEVEMENT_BADGES)[achievement.name].img
                    }, null, 8 /* PROPS */, ["src"])
                  ], 4 /* STYLE */)
                ], 2 /* CLASS */)
              ]),
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.body)
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.header)
                }, [
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.title)
                  }, _toDisplayString(_unref(i18n).ts._achievements._types[`_${achievement.name}`].title), 1 /* TEXT */),
                  _createElementVNode("span", {
                    class: _normalizeClass(_ctx.$style.time)
                  }, [
                    _createElementVNode("time", null, _toDisplayString(new Date(achievement.unlockedAt).getFullYear()) + "/" + _toDisplayString(new Date(achievement.unlockedAt).getMonth() + 1) + "/" + _toDisplayString(new Date(achievement.unlockedAt).getDate()), 1 /* TEXT */)
                  ])
                ]),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.description)
                }, _toDisplayString(__props.withDescription ? _unref(i18n).ts._achievements._types[`_${achievement.name}`].description : '???'), 1 /* TEXT */),
                ('flavor' in _unref(i18n).ts._achievements._types[`_${achievement.name}`] && __props.withDescription)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.flavor)
                  }, _toDisplayString(_unref(i18n).ts._achievements._types[`_${achievement.name}`].flavor), 1 /* TEXT */))
                  : _createCommentVNode("v-if", true)
              ])
            ]))
          }), 128 /* KEYED_FRAGMENT */)), (__props.withLocked) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(lockedAchievements.value, (achievement) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: achievement,
                  class: _normalizeClass(["_panel", [_ctx.$style.achievement, _ctx.$style.locked]]),
                  onClick: _cache[0] || (_cache[0] = achievement === 'clickedClickHere' ? clickHere() : () => {})
                }, [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.icon)
                  }),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.body)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.header)
                    }, [
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.title)
                      }, "???")
                    ]),
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.description)
                    }, "???")
                  ])
                ]))
              }), 128 /* KEYED_FRAGMENT */)) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, [ _createVNode(_component_MkLoading) ])) ]))
}
}

})
