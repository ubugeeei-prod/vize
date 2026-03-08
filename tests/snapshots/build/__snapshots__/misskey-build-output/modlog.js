import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, ref, markRaw, onMounted } from 'vue'
import * as Misskey from 'misskey-js'
import XModLog from './modlog.ModLog.vue'
import MkSelect from '@/components/MkSelect.vue'
import MkInput from '@/components/MkInput.vue'
import MkTl from '@/components/MkTl.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { prefer } from '@/preferences.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import MkPullToRefresh from '@/components/MkPullToRefresh.vue'
import MkButton from '@/components/MkButton.vue'
import MkPaginationControl from '@/components/MkPaginationControl.vue'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'modlog',
  setup(__props) {

const {
	model: type,
	def: typeDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: null },
		...Misskey.moderationLogTypes.map(t => ({ label: i18n.ts._moderationLogTypes[t] ?? t, value: t })),
	],
	initialValue: null,
});
const moderatorId = ref('');
const paginator = markRaw(new Paginator('admin/show-moderation-logs', {
	limit: 20,
	canFetchDetection: 'limit',
	canSearch: true,
	computedParams: computed(() => ({
		type: type.value,
		userId: moderatorId.value === '' ? null : moderatorId.value,
	})),
}));
paginator.init();
const timeline = computed(() => {
	return paginator.items.value.map(x => ({
		id: x.id,
		timestamp: new Date(x.createdAt).getTime(),
		data: x as Misskey.entities.ModerationLog,
	}));
});
function fetchMore() {
	if (paginator.order.value === 'oldest') {
		paginator.fetchNewer();
	} else {
		paginator.fetchOlder();
	}
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.moderationLogs,
	icon: 'ti ti-list-search',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 900px;"
        }, [
          _createElementVNode("div", { class: "_gaps" }, [
            _createVNode(MkPaginationControl, {
              paginator: _unref(paginator),
              canFilter: ""
            }, {
              default: _withCtx(() => [
                _createVNode(MkSelect, {
                  items: _unref(typeDef),
                  style: "margin: 0; flex: 1;",
                  modelValue: _unref(type),
                  "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((type).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.type), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["items", "modelValue"]),
                _createVNode(MkInput, {
                  style: "margin: 0; flex: 1;",
                  modelValue: moderatorId.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((moderatorId).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.moderator) + "(ID)", 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["modelValue"])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["paginator"]),
            _createVNode(_resolveDynamicComponent(_unref(prefer).s.enablePullToRefresh ? MkPullToRefresh : 'div'), { refresher: () => _unref(paginator).reload() }, {
              default: _withCtx(() => [
                (_unref(paginator).fetching.value)
                  ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
                  : (_unref(paginator).error.value)
                    ? (_openBlock(), _createBlock(_component_MkError, {
                      key: 1,
                      onRetry: _cache[2] || (_cache[2] = ($event: any) => (_unref(paginator).init()))
                    }))
                  : (_openBlock(), _createBlock(MkTl, {
                    key: 2,
                    events: timeline.value,
                    groupBy: "d"
                  }, {
                    left: _withCtx(({ event }) => [
                      _createElementVNode("div", null, [
                        _createVNode(_component_MkAvatar, {
                          user: event.user,
                          style: "width: 26px; height: 26px;"
                        }, null, 8 /* PROPS */, ["user"])
                      ])
                    ]),
                    right: _withCtx(({ event, timestamp, delta }) => [
                      _createElementVNode("div", { style: "margin: 4px 0;" }, [
                        _createVNode(XModLog, {
                          key: event.id,
                          log: event
                        }, null, 8 /* PROPS */, ["log"])
                      ])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["events"]))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["refresher"]),
            _createVNode(MkButton, {
              primary: "",
              rounded: "",
              style: "margin: 0 auto;",
              onClick: fetchMore
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.loadMore), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
