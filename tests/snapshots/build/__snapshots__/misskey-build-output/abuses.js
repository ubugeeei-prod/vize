import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { computed, ref, markRaw } from 'vue'
import MkSelect from '@/components/MkSelect.vue'
import MkPagination from '@/components/MkPagination.vue'
import XAbuseReport from '@/components/MkAbuseReport.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { useMkSelect } from '@/composables/use-mkselect.js'
import MkButton from '@/components/MkButton.vue'
import { store } from '@/store.js'
import { Paginator } from '@/utility/paginator.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'abuses',
  setup(__props) {

const {
	model: state,
	def: stateDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: 'all' },
		{ label: i18n.ts.unresolved, value: 'unresolved' },
		{ label: i18n.ts.resolved, value: 'resolved' },
	],
	initialValue: 'unresolved',
});
const {
	model: reporterOrigin,
	def: reporterOriginDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: 'combined' },
		{ label: i18n.ts.local, value: 'local' },
		{ label: i18n.ts.remote, value: 'remote' },
	],
	initialValue: 'combined',
});
const {
	model: targetUserOrigin,
	def: targetUserOriginDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.all, value: 'combined' },
		{ label: i18n.ts.local, value: 'local' },
		{ label: i18n.ts.remote, value: 'remote' },
	],
	initialValue: 'combined',
});
const searchUsername = ref('');
const searchHost = ref('');
const paginator = markRaw(new Paginator('admin/abuse-user-reports', {
	limit: 10,
	computedParams: computed(() => ({
		state: state.value,
		reporterOrigin: reporterOrigin.value,
		targetUserOrigin: targetUserOrigin.value,
	})),
}));
function resolved(reportId: string) {
	paginator.removeItem(reportId);
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.abuseReports,
	icon: 'ti ti-exclamation-circle',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkTip = _resolveComponent("MkTip")
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
          _createElementVNode("div", {
            class: _normalizeClass(["_gaps", _ctx.$style.root])
          }, [
            _createElementVNode("div", {
              class: _normalizeClass(["_gaps", _ctx.$style.subMenus])
            }, [
              _createVNode(MkButton, {
                link: "",
                to: "/admin/abuse-report-notification-recipient",
                primary: ""
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.notificationSetting), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _createVNode(_component_MkTip, { k: "abuses" }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._abuseUserReport.resolveTutorial), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }),
            _createElementVNode("div", {
              class: _normalizeClass(["_gaps", _ctx.$style.inputs])
            }, [
              _createVNode(MkSelect, {
                items: _unref(stateDef),
                style: "margin: 0; flex: 1;",
                modelValue: _unref(state),
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((state).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.state), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"]),
              _createVNode(MkSelect, {
                items: _unref(targetUserOriginDef),
                style: "margin: 0; flex: 1;",
                modelValue: _unref(targetUserOrigin),
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((targetUserOrigin).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.reporteeOrigin), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"]),
              _createVNode(MkSelect, {
                items: _unref(reporterOriginDef),
                style: "margin: 0; flex: 1;",
                modelValue: _unref(reporterOrigin),
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((reporterOrigin).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.reporterOrigin), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"])
            ]),
            _createTextVNode("\n\n\t\t\t" + "\n\n\t\t\t"),
            _createVNode(MkPagination, { paginator: _unref(paginator) }, {
              default: _withCtx(({items}) => [
                _createElementVNode("div", { class: "_gaps" }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (report) => {
                    return (_openBlock(), _createBlock(XAbuseReport, {
                      key: report.id,
                      report: report,
                      onResolved: resolved
                    }, null, 8 /* PROPS */, ["report"]))
                  }), 128 /* KEYED_FRAGMENT */))
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["paginator"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
