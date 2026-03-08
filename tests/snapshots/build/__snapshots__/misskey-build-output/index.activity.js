import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chart-line" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
import { ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkContainer from '@/components/MkContainer.vue'
import MkChart from '@/components/MkChart.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'index.activity',
  props: {
    user: { type: null, required: true },
    limit: { type: Number, required: false, default: 50 }
  },
  setup(__props: any) {

const props = __props
const chartSrc = ref<'per-user-notes' | 'per-user-pv'>('per-user-notes');
function showMenu(ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.notes,
		active: chartSrc.value === 'per-user-notes',
		action: () => {
			chartSrc.value = 'per-user-notes';
		},
	}, {
		text: i18n.ts.numberOfProfileView,
		active: chartSrc.value === 'per-user-pv',
		action: () => {
			chartSrc.value = 'per-user-pv';
		},
	}, /*, {
		text: i18n.ts.following,
		action: () => {
			chartSrc = 'per-user-following';
		}
	}, {
		text: i18n.ts.followers,
		action: () => {
			chartSrc = 'per-user-followers';
		}
	}*/], ev.currentTarget ?? ev.target);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkContainer, null, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.activity), 1 /* TEXT */)
      ]),
      func: _withCtx(({ buttonStyleClass }) => [
        _createElementVNode("button", {
          class: _normalizeClass(["_button", buttonStyleClass]),
          onClick: showMenu
        }, [
          _hoisted_2
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { style: "padding: 8px;" }, [
          _createVNode(MkChart, {
            src: chartSrc.value,
            args: { user: __props.user, withoutAll: true },
            span: "day",
            limit: __props.limit,
            bar: true,
            stacked: true,
            detailed: false,
            aspectRatio: 5
          }, null, 8 /* PROPS */, ["src", "args", "limit", "bar", "stacked", "detailed", "aspectRatio"])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
