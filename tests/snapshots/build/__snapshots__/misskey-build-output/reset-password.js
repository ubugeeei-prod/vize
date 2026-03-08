import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
import { defineAsyncComponent, onMounted, ref, computed } from 'vue'
import MkInput from '@/components/MkInput.vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { mainRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'reset-password',
  props: {
    token: { type: String, required: false }
  },
  setup(__props: any) {

const props = __props
const password = ref('');
async function save() {
	if (props.token == null) return;
	await os.apiWithDialog('reset-password', {
		token: props.token,
		password: password.value,
	});
	mainRouter.push('/');
}
onMounted(async () => {
	if (props.token == null) {
		const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkForgotPassword.vue').then(x => x.default), {}, {
			closed: () => dispose(),
		});
		mainRouter.push('/');
	}
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.resetPassword,
	icon: 'ti ti-lock',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        (__props.token)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "_spacer",
            style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
          }, [
            _createElementVNode("div", { class: "_gaps_m" }, [
              _createVNode(MkInput, {
                type: "password",
                modelValue: password.value,
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((password).value = $event))
              }, {
                prefix: _withCtx(() => [
                  _hoisted_1
                ]),
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.newPassword), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"]),
              _createVNode(MkButton, {
                primary: "",
                onClick: save
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
