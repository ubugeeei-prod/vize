import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", { class: "label" }, "Users")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", { class: "label" }, "Notes")
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-planet" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("div", { class: "label" }, "Instances")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-icons" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("div", { class: "label" }, "Custom emojis")
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-access-point" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("div", { class: "label" }, "Online")
import { onMounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { misskeyApi, misskeyApiGet } from '@/utility/misskey-api.js'
import MkNumberDiff from '@/components/MkNumberDiff.vue'
import MkNumber from '@/components/MkNumber.vue'
import { i18n } from '@/i18n.js'
import { customEmojis } from '@/custom-emojis.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'overview.stats',
  setup(__props) {

const stats = ref<Misskey.entities.StatsResponse | null>(null);
const usersComparedToThePrevDay = ref<number | null>(null);
const notesComparedToThePrevDay = ref<number | null>(null);
const onlineUsersCount = ref(0);
const fetching = ref(true);
onMounted(async () => {
	const [_stats, _onlineUsersCount] = await Promise.all([
		misskeyApi('stats', {}),
		misskeyApiGet('get-online-users-count').then(res => res.count),
	]);
	stats.value = _stats;
	onlineUsersCount.value = _onlineUsersCount;
	misskeyApiGet('charts/users', { limit: 2, span: 'day' }).then(chart => {
		usersComparedToThePrevDay.value = _stats.originalUsersCount - chart.local.total[1];
	});
	misskeyApiGet('charts/notes', { limit: 2, span: 'day' }).then(chart => {
		notesComparedToThePrevDay.value = _stats.originalNotesCount - chart.local.total[1];
	});
	fetching.value = false;
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkError = _resolveComponent("MkError")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(_Transition, {
        name: _unref(prefer).s.animation ? '_transition_zoom' : '',
        mode: "out-in"
      }, {
        default: _withCtx(() => [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (stats.value != null)
              ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                class: _normalizeClass(_ctx.$style.root)
              }, [
                _createElementVNode("div", { class: "item _panel users" }, [
                  _createElementVNode("div", { class: "icon" }, [
                    _hoisted_1
                  ]),
                  _createElementVNode("div", { class: "body" }, [
                    _createElementVNode("div", { class: "value" }, [
                      _createVNode(MkNumber, {
                        value: stats.value.originalUsersCount,
                        style: "margin-right: 0.5em;"
                      }, null, 8 /* PROPS */, ["value"]),
                      (usersComparedToThePrevDay.value != null)
                        ? (_openBlock(), _createBlock(MkNumberDiff, {
                          key: 0,
                          class: "diff",
                          value: usersComparedToThePrevDay.value
                        }, null, 8 /* PROPS */, ["value"]))
                        : _createCommentVNode("v-if", true)
                    ]),
                    _hoisted_2
                  ])
                ]),
                _createElementVNode("div", { class: "item _panel notes" }, [
                  _createElementVNode("div", { class: "icon" }, [
                    _hoisted_3
                  ]),
                  _createElementVNode("div", { class: "body" }, [
                    _createElementVNode("div", { class: "value" }, [
                      _createVNode(MkNumber, {
                        value: stats.value.originalNotesCount,
                        style: "margin-right: 0.5em;"
                      }, null, 8 /* PROPS */, ["value"]),
                      (notesComparedToThePrevDay.value != null)
                        ? (_openBlock(), _createBlock(MkNumberDiff, {
                          key: 0,
                          class: "diff",
                          value: notesComparedToThePrevDay.value
                        }, null, 8 /* PROPS */, ["value"]))
                        : _createCommentVNode("v-if", true)
                    ]),
                    _hoisted_4
                  ])
                ]),
                _createElementVNode("div", { class: "item _panel instances" }, [
                  _createElementVNode("div", { class: "icon" }, [
                    _hoisted_5
                  ]),
                  _createElementVNode("div", { class: "body" }, [
                    _createElementVNode("div", { class: "value" }, [
                      _createVNode(MkNumber, {
                        value: stats.value.instances,
                        style: "margin-right: 0.5em;"
                      }, null, 8 /* PROPS */, ["value"])
                    ]),
                    _hoisted_6
                  ])
                ]),
                _createElementVNode("div", { class: "item _panel emojis" }, [
                  _createElementVNode("div", { class: "icon" }, [
                    _hoisted_7
                  ]),
                  _createElementVNode("div", { class: "body" }, [
                    _createElementVNode("div", { class: "value" }, [
                      _createVNode(MkNumber, {
                        value: _unref(customEmojis).length,
                        style: "margin-right: 0.5em;"
                      }, null, 8 /* PROPS */, ["value"])
                    ]),
                    _hoisted_8
                  ])
                ]),
                _createElementVNode("div", { class: "item _panel online" }, [
                  _createElementVNode("div", { class: "icon" }, [
                    _hoisted_9
                  ]),
                  _createElementVNode("div", { class: "body" }, [
                    _createElementVNode("div", { class: "value" }, [
                      _createVNode(MkNumber, {
                        value: onlineUsersCount.value,
                        style: "margin-right: 0.5em;"
                      }, null, 8 /* PROPS */, ["value"])
                    ]),
                    _hoisted_10
                  ])
                ])
              ]))
            : (_openBlock(), _createBlock(_component_MkError, { key: 2 }))
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["name"]) ]))
}
}

})
