import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { onMounted, ref, watch } from 'vue'
import { Interpreter, Parser } from '@syuilo/aiscript'
import { useWidgetPropsManager } from './widget.js'
import type { Ref } from 'vue'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import type { AsUiComponent, AsUiRoot } from '@/aiscript/ui.js'
import * as os from '@/os.js'
import { aiScriptReadline, createAiScriptEnv } from '@/aiscript/api.js'
import { $i } from '@/i.js'
import { i18n } from '@/i18n.js'
import MkAsUi from '@/components/MkAsUi.vue'
import MkContainer from '@/components/MkContainer.vue'
import { registerAsUiLib } from '@/aiscript/ui.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'aiscriptApp';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetAiscriptApp',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	script: {
		type: 'string',
		label: i18n.ts.script,
		multiline: true,
		manualSave: true,
		default: '',
	},
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const parser = new Parser();
const root = ref<AsUiRoot>();
const components = ref<Ref<AsUiComponent>[]>([]);
const isSyntaxError = ref(false);
async function run() {
	isSyntaxError.value = false;
	const aiscript = new Interpreter({
		...createAiScriptEnv({
			storageKey: 'widget',
			token: $i?.token,
		}),
		...registerAsUiLib(components.value, (_root) => {
			root.value = _root.value;
		}),
	}, {
		in: aiScriptReadline,
		out: (value) => {
			// nop
		},
		log: (type, params) => {
			// nop
		},
	});
	let ast;
	try {
		ast = parser.parse(widgetProps.script);
	} catch (err) {
		isSyntaxError.value = true;
		return;
	}
	try {
		await aiscript.exec(ast);
	} catch (err) {
		os.alert({
			type: 'error',
			title: 'AiScript Error',
			text: err instanceof Error ? err.message : String(err),
		});
	}
}
watch(() => widgetProps.script, () => {
	run();
});
onMounted(() => {
	run();
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      class: "mkw-aiscriptApp"
    }, {
      header: _withCtx(() => [
        _createTextVNode("App")
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          (isSyntaxError.value)
            ? (_openBlock(), _createElementBlock("div", { key: 0 }, "Syntax error :("))
            : (root.value)
              ? (_openBlock(), _createBlock(MkAsUi, {
                key: 1,
                component: root.value,
                components: components.value,
                size: "small"
              }, null, 8 /* PROPS */, ["component", "components"]))
            : _createCommentVNode("v-if", true)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader"]))
}
}

})
