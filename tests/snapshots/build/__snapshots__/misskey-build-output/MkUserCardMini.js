import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = { class: "_monospace" }
import * as Misskey from 'misskey-js'
import { onMounted, ref } from 'vue'
import MkMiniChart from '@/components/MkMiniChart.vue'
import { misskeyApiGet } from '@/utility/misskey-api.js'
import { acct } from '@/filters/user.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUserCardMini',
  props: {
    user: { type: null, required: true },
    withChart: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const props = __props
const chartValues = ref<number[] | null>(null);
onMounted(() => {
	if (props.withChart) {
		misskeyApiGet('charts/user/notes', { userId: props.user.id, limit: 16 + 1, span: 'day' }).then(res => {
			// 今日のぶんの値はまだ途中の値であり、それも含めると大抵の場合前日よりも下降しているようなグラフになってしまうため今日は弾く
			res.inc.splice(0, 1);
			chartValues.value = res.inc;
		});
	}
});

return (_ctx: any,_cache: any) => {
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_MkUserName = _resolveComponent("MkUserName")
  const _directive_adaptive_bg = _resolveDirective("adaptive-bg")

  return _withDirectives((_openBlock(), _createElementBlock("div", {
      class: _normalizeClass([_ctx.$style.root])
    }, [ _createVNode(_component_MkAvatar, {
        class: _normalizeClass(_ctx.$style.avatar),
        user: __props.user,
        indicator: ""
      }, null, 8 /* PROPS */, ["user"]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.body)
      }, [ _createElementVNode("span", {
          class: _normalizeClass(_ctx.$style.name)
        }, [ _createVNode(_component_MkUserName, { user: __props.user }, null, 8 /* PROPS */, ["user"]) ]), _createElementVNode("span", {
          class: _normalizeClass(_ctx.$style.sub)
        }, [ _renderSlot(_ctx.$slots, "sub", {}, () => [ _createElementVNode("span", _hoisted_1, "@" + _toDisplayString(_unref(acct)(__props.user)), 1 /* TEXT */) ]) ]) ]), (chartValues.value) ? (_openBlock(), _createBlock(MkMiniChart, {
          key: 0,
          class: _normalizeClass(_ctx.$style.chart),
          src: chartValues.value
        }, null, 8 /* PROPS */, ["src"])) : _createCommentVNode("v-if", true) ])), [ [_directive_adaptive_bg] ])
}
}

})
