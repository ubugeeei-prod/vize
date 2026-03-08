import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cookie", style: "font-size: 70%;" })
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useInterval } from '@@/js/use-interval.js'
import MkPlusOneEffect from '@/components/MkPlusOneEffect.vue'
import * as os from '@/os.js'
import * as game from '@/utility/clicker-game.js'
import number from '@/filters/number.js'
import { claimAchievement } from '@/utility/achievements.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkClickerGame',
  setup(__props) {

const saveData = game.saveData;
const cookies = computed(() => saveData.value?.cookies);
const cps = ref(0);
const prevCookies = ref(0);
function onClick(ev: PointerEvent) {
	const x = ev.clientX;
	const y = ev.clientY;
	const { dispose } = os.popup(MkPlusOneEffect, { x, y }, {
		end: () => dispose(),
	});
	saveData.value!.cookies++;
	saveData.value!.totalCookies++;
	saveData.value!.totalHandmadeCookies++;
	saveData.value!.clicked++;
	if (cookies.value === 1) {
		claimAchievement('cookieClicked');
	}
}
useInterval(() => {
	const diff = saveData.value!.cookies - prevCookies.value;
	cps.value = diff;
	prevCookies.value = saveData.value!.cookies;
}, 1000, {
	immediate: false,
	afterMounted: true,
});
useInterval(game.save, 1000 * 5, {
	immediate: false,
	afterMounted: true,
});
onMounted(async () => {
	await game.load();
	prevCookies.value = saveData.value!.cookies;
});
onUnmounted(() => {
	game.save();
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _directive_click_anime = _resolveDirective("click-anime")

  return (_openBlock(), _createElementBlock("div", null, [ (game.ready) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: _normalizeClass(_ctx.$style.game)
        }, [ _createElementVNode("div", {
            class: _normalizeClass(["", _ctx.$style.cps])
          }, _toDisplayString(number(cps.value)) + "cps", 1 /* TEXT */), _createElementVNode("div", {
            class: _normalizeClass(["", _ctx.$style.count]),
            "data-testid": "count"
          }, [ _hoisted_1, _createTextVNode(" " + _toDisplayString(number(cookies.value)), 1 /* TEXT */) ]), _createElementVNode("button", {
            class: "_button",
            onClick: onClick
          }, [ _createElementVNode("img", {
              src: "/client-assets/cookie.png",
              class: _normalizeClass(_ctx.$style.img)
            }) ]) ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, [ _createVNode(_component_MkLoading) ])) ]))
}
}

})
