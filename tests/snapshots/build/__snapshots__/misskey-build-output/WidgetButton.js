import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
import { Interpreter, Parser } from "@syuilo/aiscript";
import { useWidgetPropsManager } from "./widget.js";
import * as os from "@/os.js";
import { aiScriptReadline, createAiScriptEnv } from "@/aiscript/api.js";
import { $i } from "@/i.js";
import MkButton from "@/components/MkButton.vue";
import { i18n } from "@/i18n.js";
const name = "button";
export default {
  __name: "WidgetButton",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = {
      label: {
        type: "string",
        label: i18n.ts.label,
        default: "BUTTON"
      },
      colored: {
        type: "boolean",
        label: i18n.ts._widgetOptions._button.colored,
        default: true
      },
      script: {
        type: "string",
        label: i18n.ts.script,
        multiline: true,
        default: "Mk:dialog(\"hello\", \"world\")"
      }
    };
    const { widgetProps, configure } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    const parser = new Parser();
    async function run() {
      const aiscript = new Interpreter(createAiScriptEnv({
        storageKey: "widget",
        token: $i?.token
      }), {
        in: aiScriptReadline,
        out: (value) => {
          // nop
        },
        log: (type, params) => {
          // nop
        }
      });
      let ast;
      try {
        ast = parser.parse(widgetProps.script);
      } catch (err) {
        os.alert({
          type: "error",
          text: "Syntax error :("
        });
        return;
      }
      try {
        await aiscript.exec(ast);
      } catch (err) {
        os.alert({
          type: "error",
          text: err instanceof Error ? err.message : String(err)
        });
      }
    }
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", {
        "data-cy-mkw-button": "",
        class: "mkw-button"
      }, [_createVNode(MkButton, {
        primary: _unref(widgetProps).colored,
        full: "",
        onClick: run
      }, {
        default: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(widgetProps).label),
          1
          /* TEXT */
        )]),
        _: 1
      }, 8, ["primary"])]);
    };
  }
};
