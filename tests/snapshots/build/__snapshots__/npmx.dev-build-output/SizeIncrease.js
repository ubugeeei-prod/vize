import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:trending-up w-4 h-4", "aria-hidden": "true" })
import type { InstallSizeDiff } from '~/composables/useInstallSizeDiff'

export default /*@__PURE__*/_defineComponent({
  __name: 'SizeIncrease',
  props: {
    diff: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const bytesFormatter = useBytesFormatter()
const numberFormatter = useNumberFormatter()
const sizePercent = computed(() => Math.round(props.diff.sizeRatio * 100))

return (_ctx: any,_cache: any) => {
  const _component_i18n_t = _resolveComponent("i18n-t")

  return (_openBlock(), _createElementBlock("div", { class: "border border-amber-600/40 bg-amber-500/10 rounded-lg px-3 py-2 text-base text-amber-800 dark:text-amber-400" }, [ _createElementVNode("h2", { class: "font-medium mb-1 flex items-center gap-2" }, [ _hoisted_1, _createTextVNode("\n      " + _toDisplayString(__props.diff.sizeThresholdExceeded && __props.diff.depThresholdExceeded ? _ctx.$t('package.size_increase.title_both', { version: __props.diff.comparisonVersion }) : __props.diff.sizeThresholdExceeded ? _ctx.$t('package.size_increase.title_size', { version: __props.diff.comparisonVersion }) : _ctx.$t('package.size_increase.title_deps', { version: __props.diff.comparisonVersion })), 1 /* TEXT */) ]), _createElementVNode("p", { class: "text-sm m-0 mt-1" }, [ (__props.diff.sizeThresholdExceeded) ? (_openBlock(), _createBlock(_component_i18n_t, {
            key: 0,
            keypath: "package.size_increase.size",
            scope: "global"
          }, {
            percent: _withCtx(() => [
              _createElementVNode("strong", null, _toDisplayString(sizePercent.value) + "%", 1 /* TEXT */)
            ]),
            size: _withCtx(() => [
              _createElementVNode("strong", null, _toDisplayString(_unref(bytesFormatter).format(__props.diff.sizeIncrease)), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })) : _createCommentVNode("v-if", true), (__props.diff.sizeThresholdExceeded && __props.diff.depThresholdExceeded) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createTextVNode(" · ") ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), (__props.diff.depThresholdExceeded) ? (_openBlock(), _createBlock(_component_i18n_t, {
            key: 0,
            keypath: "package.size_increase.deps",
            scope: "global"
          }, {
            count: _withCtx(() => [
              _createElementVNode("strong", null, "+" + _toDisplayString(_unref(numberFormatter).format(__props.diff.depDiff)), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })) : _createCommentVNode("v-if", true) ]) ]))
}
}

})
