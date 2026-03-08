import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", null, "&middot;")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", null, "&middot;")
import type { BuildInfo } from '#shared/types'

export default /*@__PURE__*/_defineComponent({
  __name: 'BuildEnvironment',
  props: {
    footer: { type: Boolean, required: false, default: false },
    buildInfo: { type: null, required: false }
  },
  setup(__props: any) {

const appConfig = useAppConfig()
const buildInfo = computed(() => __props.buildInfo || appConfig.buildInfo)
const buildTime = computed(() => new Date(buildInfo.value.time))

return (_ctx: any,_cache: any) => {
  const _component_DateTime = _resolveComponent("DateTime")
  const _component_i18n_t = _resolveComponent("i18n-t")
  const _component_LinkBase = _resolveComponent("LinkBase")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["font-mono text-xs text-fg-muted flex items-center gap-2 motion-safe:animate-fade-in motion-safe:animate-fill-both", __props.footer ? 'my-1 justify-center sm:justify-start' : 'mb-8 justify-center']),
      style: "animation-delay: 0.05s"
    }, [ _createVNode(_component_i18n_t, {
        keypath: "built_at",
        scope: "global"
      }, {
        default: _withCtx(() => [
          _createVNode(_component_DateTime, {
            datetime: buildTime.value,
            year: "numeric",
            month: "short",
            day: "numeric"
          }, null, 8 /* PROPS */, ["datetime"])
        ]),
        _: 1 /* STABLE */
      }), _hoisted_1, (buildInfo.value.env === 'release') ? (_openBlock(), _createBlock(_component_LinkBase, {
          key: 0,
          to: `https://github.com/npmx-dev/npmx.dev/tag/v${buildInfo.value.version}`
        }, {
          default: _withCtx(() => [
            _createTextVNode("\n      v"),
            _createTextVNode(_toDisplayString(buildInfo.value.version), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["to"])) : (_openBlock(), _createElementBlock("span", {
          key: 1,
          class: "tracking-wider"
        }, _toDisplayString(buildInfo.value.env), 1 /* TEXT */)), (buildInfo.value.commit && buildInfo.value.branch !== 'release') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_2, _createVNode(_component_LinkBase, { to: `https://github.com/npmx-dev/npmx.dev/commit/${buildInfo.value.commit}` }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(buildInfo.value.shortCommit), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"]) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ], 2 /* CLASS */))
}
}

})
