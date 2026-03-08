import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-reload" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { ref, computed } from 'vue'
import XQueue from './federation-job-queue.chart.vue'
import type { Ref } from 'vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkButton from '@/components/MkButton.vue'

export type ApQueueDomain = 'deliver' | 'inbox';

export default /*@__PURE__*/_defineComponent({
  __name: 'federation-job-queue',
  setup(__props) {

const tab: Ref<ApQueueDomain> = ref('deliver');
function clear() {
	os.confirm({
		type: 'warning',
		title: i18n.ts.clearQueueConfirmTitle,
		text: i18n.ts.clearQueueConfirmText,
	}).then(({ canceled }) => {
		if (canceled) return;
		os.apiWithDialog('admin/queue/clear', { queue: tab.value, state: '*' });
	});
}
function promoteAllQueues() {
	os.confirm({
		type: 'warning',
		title: i18n.ts.retryAllQueuesConfirmTitle,
		text: i18n.ts.retryAllQueuesConfirmText,
	}).then(({ canceled }) => {
		if (canceled) return;
		os.apiWithDialog('admin/queue/promote-jobs', { queue: tab.value });
	});
}
const headerActions = computed(() => []);
const headerTabs = computed(() => [{
	key: 'deliver',
	title: 'Deliver',
}, {
	key: 'inbox',
	title: 'Inbox',
}]);
definePage(() => ({
	title: i18n.ts.federationJobs,
	icon: 'ti ti-clock-play',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value,
      tab: tab.value,
      "onUpdate:tab": _cache[0] || (_cache[0] = ($event: any) => ((tab).value = $event))
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          (tab.value === 'deliver')
            ? (_openBlock(), _createBlock(XQueue, {
              key: 0,
              domain: "deliver"
            }))
            : (tab.value === 'inbox')
              ? (_openBlock(), _createBlock(XQueue, {
                key: 1,
                domain: "inbox"
              }))
            : _createCommentVNode("v-if", true),
          _hoisted_1,
          _createElementVNode("div", { class: "_buttons" }, [
            _createVNode(MkButton, { onClick: promoteAllQueues }, {
              default: _withCtx(() => [
                _hoisted_2,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.retryAllQueuesNow), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkButton, {
              danger: "",
              onClick: clear
            }, {
              default: _withCtx(() => [
                _hoisted_3,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.clearQueue), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs", "tab"]))
}
}

})
