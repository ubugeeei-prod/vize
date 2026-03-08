import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-bell", style: "margin-right: 8px;" })
import { defineAsyncComponent, useTemplateRef } from 'vue'
import XColumn from './column.vue'
import type { Column } from '@/deck.js'
import { updateColumn } from '@/deck.js'
import MkStreamingNotificationsTimeline from '@/components/MkStreamingNotificationsTimeline.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'notifications-column',
  props: {
    column: { type: null, required: true },
    isStacked: { type: Boolean, required: true }
  },
  setup(__props: any) {

const props = __props
const notificationsComponent = useTemplateRef('notificationsComponent');
async function func() {
	const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkNotificationSelectWindow.vue').then(x => x.default), {
		excludeTypes: props.column.excludeTypes,
	}, {
		done: async (res) => {
			const { excludeTypes } = res;
			updateColumn(props.column.id, {
				excludeTypes: excludeTypes,
			});
		},
		closed: () => dispose(),
	});
}
const menu = [{
	icon: 'ti ti-pencil',
	text: i18n.ts.notificationSetting,
	action: func,
}];

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XColumn, {
      column: __props.column,
      isStacked: __props.isStacked,
      menu: menu,
      refresher: async () => { await _unref(notificationsComponent)?.reload() }
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createTextVNode(_toDisplayString(__props.column.name || _unref(i18n).ts._deck._columns.notifications), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createVNode(MkStreamingNotificationsTimeline, {
          ref_key: "notificationsComponent", ref: notificationsComponent,
          excludeTypes: props.column.excludeTypes
        }, null, 8 /* PROPS */, ["excludeTypes"])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["column", "isStacked", "menu", "refresher"]))
}
}

})
