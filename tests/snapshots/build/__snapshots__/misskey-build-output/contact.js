import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-report-search" })
import { ref } from 'vue'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { definePage } from '@/page.js'
import { getUserEnvironment } from '@/utility/get-user-environment.js'
import type { UserEnvironment } from '@/utility/get-user-environment.js'
import MkKeyValue from '@/components/MkKeyValue.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkLink from '@/components/MkLink.vue'
import MkCode from '@/components/MkCode.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'contact',
  setup(__props) {

const userEnv = ref<UserEnvironment | null>(null);
async function onOpened() {
	if (userEnv.value == null) {
		userEnv.value = await getUserEnvironment();
	}
}
definePage(() => ({
	title: i18n.ts.inquiry,
	icon: 'ti ti-help-circle',
}));

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 20px;"
        }, [
          _createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(MkKeyValue, { copy: _unref(instance).maintainerName }, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.administrator), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                (_unref(instance).maintainerName)
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    _toDisplayString(_unref(instance).maintainerName)
                  ], 64 /* STABLE_FRAGMENT */))
                  : (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    style: "opacity: 0.7;"
                  }, "(" + _toDisplayString(_unref(i18n).ts.none) + ")", 1 /* TEXT */))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["copy"]),
            _createVNode(MkKeyValue, { copy: _unref(instance).maintainerEmail }, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.contact), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                (_unref(instance).maintainerEmail)
                  ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
                    _toDisplayString(_unref(instance).maintainerEmail)
                  ], 64 /* STABLE_FRAGMENT */))
                  : (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    style: "opacity: 0.7;"
                  }, "(" + _toDisplayString(_unref(i18n).ts.none) + ")", 1 /* TEXT */))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["copy"]),
            _createVNode(MkKeyValue, { copy: _unref(instance).inquiryUrl }, {
              key: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.inquiry), 1 /* TEXT */)
              ]),
              value: _withCtx(() => [
                (_unref(instance).inquiryUrl)
                  ? (_openBlock(), _createBlock(MkLink, {
                    key: 0,
                    url: _unref(instance).inquiryUrl,
                    target: "_blank"
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(instance).inquiryUrl), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["url"]))
                  : (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    style: "opacity: 0.7;"
                  }, "(" + _toDisplayString(_unref(i18n).ts.none) + ")", 1 /* TEXT */))
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["copy"]),
            _createVNode(MkFolder, { onOpened: onOpened }, {
              icon: _withCtx(() => [
                _hoisted_1
              ]),
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.deviceInfo), 1 /* TEXT */)
              ]),
              caption: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.deviceInfoDescription), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                (userEnv.value == null)
                  ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
                  : (_openBlock(), _createBlock(MkCode, {
                    key: 1,
                    lang: "json",
                    code: JSON.stringify(userEnv.value, null, 2),
                    style: "max-height: 300px; overflow: auto;"
                  }, null, 8 /* PROPS */, ["code"]))
              ]),
              _: 1 /* STABLE */
            })
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
