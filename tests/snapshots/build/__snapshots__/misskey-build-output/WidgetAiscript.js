import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vModelText as _vModelText } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-terminal-2" })
import { ref } from 'vue'
import { Interpreter, Parser, utils } from '@syuilo/aiscript'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import type { Value } from '@syuilo/aiscript/interpreter/value.js'
import * as os from '@/os.js'
import MkContainer from '@/components/MkContainer.vue'
import { aiScriptReadline, createAiScriptEnv } from '@/aiscript/api.js'
import { $i } from '@/i.js'
import { i18n } from '@/i18n.js'
import { genId } from '@/utility/id.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'aiscript';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetAiscript',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
	script: {
		type: 'string',
		label: i18n.ts.script,
		multiline: true,
		default: '(1 + 1)',
		hidden: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const parser = new Parser();
const logs = ref<{
	id: string;
	text: string;
	print: boolean;
}[]>([]);
const run = async () => {
	logs.value = [];
	const aiscript = new Interpreter(createAiScriptEnv({
		storageKey: 'widget',
		token: $i?.token,
	}), {
		in: aiScriptReadline,
		out: (value) => {
			logs.value.push({
				id: genId(),
				text: value.type === 'str' ? value.value : utils.valToString(value),
				print: true,
			});
		},
		log: (type, params) => {
			switch (type) {
				case 'end': logs.value.push({
					id: genId(),
					text: utils.valToString(params.val as Value, true),
					print: false,
				}); break;
				default: break;
			}
		},
	});

	let ast;
	try {
		ast = parser.parse(widgetProps.script);
	} catch (err) {
		os.alert({
			type: 'error',
			text: 'Syntax error :(',
		});
		return;
	}
	try {
		await aiscript.exec(ast);
	} catch (err) {
		os.alert({
			type: 'error',
			text: err instanceof Error ? err.message : String(err),
		});
	}
};
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      "data-cy-mkw-aiscript": "",
      class: "mkw-aiscript"
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets.aiscript), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { class: "uylguesu _monospace" }, [
          _withDirectives(_createElementVNode("textarea", {
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((_unref(widgetProps).script) = $event)),
            placeholder: "(1 + 1)"
          }, null, 512 /* NEED_PATCH */), [
            [_vModelText, _unref(widgetProps).script]
          ]),
          _createElementVNode("button", {
            class: "_buttonPrimary",
            onClick: run
          }, "RUN"),
          _createElementVNode("div", { class: "logs" }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(logs.value, (log) => {
              return (_openBlock(), _createElementBlock("div", {
                key: log.id,
                class: _normalizeClass(["log", { print: log.print }])
              }, _toDisplayString(log.text), 3 /* TEXT, CLASS */))
            }), 128 /* KEYED_FRAGMENT */))
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader"]))
}
}

})
