import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-exclamation-circle", style: "margin-right: 0.5em;" })
import { ref, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import MkWindow from '@/components/MkWindow.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkAbuseReportWindow',
  props: {
    user: { type: null, required: true },
    initialComment: { type: String, required: false }
  },
  emits: ["closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const uiWindow = useTemplateRef('uiWindow');
const comment = ref(props.initialComment ?? '');
function send() {
	os.apiWithDialog('users/report-abuse', {
		userId: props.user.id,
		comment: comment.value,
	}, undefined).then(res => {
		os.alert({
			type: 'success',
			text: i18n.ts.abuseReported,
		});
		uiWindow.value?.close();
		emit('closed');
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_I18n = _resolveComponent("I18n")

  return (_openBlock(), _createBlock(MkWindow, {
      ref_key: "uiWindow", ref: uiWindow,
      initialWidth: 400,
      initialHeight: 500,
      canResize: true,
      onClosed: _cache[0] || (_cache[0] = ($event: any) => (emit('closed')))
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createVNode(_component_I18n, {
          src: _unref(i18n).ts.reportAbuseOf,
          tag: "span"
        }, {
          name: _withCtx(() => [
            _createElementVNode("b", null, [
              _createVNode(_component_MkAcct, { user: __props.user }, null, 8 /* PROPS */, ["user"])
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["src"])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px;"
        }, [
          _createElementVNode("div", {
            class: _normalizeClass(["_gaps_m", _ctx.$style.root])
          }, [
            _createElementVNode("div", { class: "" }, [
              _createVNode(MkTextarea, {
                modelValue: comment.value,
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((comment).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.details), 1 /* TEXT */)
                ]),
                caption: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.fillAbuseReportDescription), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _createElementVNode("div", { class: "" }, [
              _createVNode(MkButton, {
                primary: "",
                full: "",
                disabled: comment.value.length === 0,
                onClick: send
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.send), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["disabled"])
            ])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["initialWidth", "initialHeight", "canResize"]))
}
}

})
