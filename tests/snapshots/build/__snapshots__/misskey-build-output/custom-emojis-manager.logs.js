import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, ref, toRefs } from 'vue'
import { i18n } from '@/i18n.js'
import MkGrid from '@/components/grid/MkGrid.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import { copyGridDataToClipboard } from '@/components/grid/grid-utils.js'
import type { RequestLogItem } from '@/pages/admin/custom-emojis-manager.impl.js'
import type { GridSetting } from '@/components/grid/grid.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'custom-emojis-manager.logs',
  props: {
    logs: { type: Array, required: true }
  },
  setup(__props: any) {

const props = __props
function setupGrid(): GridSetting {
	return {
		row: {
			showNumber: false,
			selectable: false,
			contextMenuFactory: (row, context) => {
				return [
					{
						type: 'button',
						text: i18n.ts._customEmojisManager._gridCommon.copySelectionRows,
						icon: 'ti ti-copy',
						action: () => copyGridDataToClipboard(logs, context),
					},
				];
			},
		},
		cols: [
			{ bindTo: 'failed', title: 'failed', type: 'boolean', editable: false, width: 50 },
			{ bindTo: 'url', icon: 'ti-icons', type: 'image', editable: false, width: 'auto' },
			{ bindTo: 'name', title: 'name', type: 'text', editable: false, width: 140 },
			{ bindTo: 'error', title: 'log', type: 'text', editable: false, width: 'auto' },
		],
		cells: {
			contextMenuFactory: (col, row, value, context) => {
				return [
					{
						type: 'button',
						text: i18n.ts._customEmojisManager._gridCommon.copySelectionRanges,
						icon: 'ti ti-copy',
						action: () => copyGridDataToClipboard(logs, context),
					},
				];
			},
		},
	};
}
const { logs } = toRefs(props);
const showingSuccessLogs = ref<boolean>(false);
const filteredLogs = computed(() => {
	const forceShowing = showingSuccessLogs.value;
	return logs.value.filter((log) => forceShowing || log.failed);
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ (_unref(logs).length > 0) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          style: "display:flex; flex-direction: column; overflow-y: scroll; gap: 16px;"
        }, [ _createVNode(MkSwitch, {
            modelValue: showingSuccessLogs.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((showingSuccessLogs).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._customEmojisManager._logs.showSuccessLogSwitch), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createElementVNode("div", null, [ (filteredLogs.value.length > 0) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createVNode(MkGrid, {
                  data: filteredLogs.value,
                  settings: setupGrid()
                }, null, 8 /* PROPS */, ["data", "settings"]) ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts._customEmojisManager._logs.failureLogNothing), 1 /* TEXT */)) ]) ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts._customEmojisManager._logs.logNothing), 1 /* TEXT */)) ]))
}
}

})
