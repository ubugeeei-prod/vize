import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-play" })
const _hoisted_2 = { class: "" }
import { onDeactivated, onUnmounted, ref, watch, computed } from 'vue'
import { Interpreter, Parser, utils } from '@syuilo/aiscript'
import type { Ref } from 'vue'
import type { AsUiComponent } from '@/aiscript/ui.js'
import type { AsUiRoot } from '@/aiscript/ui.js'
import type { Value } from '@syuilo/aiscript/interpreter/value.js'
import MkContainer from '@/components/MkContainer.vue'
import MkButton from '@/components/MkButton.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkCodeEditor from '@/components/MkCodeEditor.vue'
import { aiScriptReadline, createAiScriptEnv } from '@/aiscript/api.js'
import * as os from '@/os.js'
import { $i } from '@/i.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { registerAsUiLib } from '@/aiscript/ui.js'
import MkAsUi from '@/components/MkAsUi.vue'
import { miLocalStorage } from '@/local-storage.js'
import { claimAchievement } from '@/utility/achievements.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'scratchpad',
  setup(__props) {

const parser = new Parser();
let aiscript: Interpreter;
const code = ref('');
const logs = ref<{
	id: number;
	text: string;
	print: boolean;
}[]>([]);
const root = ref<AsUiRoot | undefined>();
const components = ref<Ref<AsUiComponent>[]>([]);
const uiKey = ref(0);
const uiInspectorOpenedComponents = ref(new WeakMap<AsUiComponent | Ref<AsUiComponent>, boolean>);
const saved = miLocalStorage.getItem('scratchpad');
if (saved) {
	code.value = saved;
}
watch(code, () => {
	miLocalStorage.setItem('scratchpad', code.value);
});
function stringifyUiProps(uiProps: AsUiComponent) {
	return JSON.stringify(
		{ ...uiProps, type: undefined, id: undefined },
		(k, v) => typeof v === 'function' ? '<function>' : v,
		2,
	);
}
async function run() {
	if (aiscript) aiscript.abort();
	root.value = undefined;
	components.value = [];
	uiKey.value++;
	logs.value = [];
	aiscript = new Interpreter(({
		...createAiScriptEnv({
			storageKey: 'widget',
			token: $i?.token,
		}),
		...registerAsUiLib(components.value, (_root) => {
			root.value = _root.value;
		}),
	}), {
		in: aiScriptReadline,
		out: (value) => {
			if (value.type === 'str' && value.value.toLowerCase().replace(',', '').includes('hello world')) {
				claimAchievement('outputHelloWorldOnScratchpad');
			}
			logs.value.push({
				id: Math.random(),
				text: value.type === 'str' ? value.value : utils.valToString(value),
				print: true,
			});
		},
		err: (err) => {
			os.alert({
				type: 'error',
				title: 'AiScript Error',
				text: err.toString(),
			});
		},
		log: (type, params) => {
			switch (type) {
				case 'end': logs.value.push({
					id: Math.random(),
					text: utils.valToString(params.val as Value, true),
					print: false,
				}); break;
				default: break;
			}
		},
	});
	let ast;
	try {
		ast = parser.parse(code.value);
	} catch (err: any) {
		os.alert({
			type: 'error',
			title: 'Syntax Error',
			text: err.toString(),
		});
		return;
	}
	try {
		await aiscript.exec(ast);
	} catch (err: any) {
		// AiScript runtime errors should be processed by error callback function
		// so errors caught here are AiScript's internal errors.
		os.alert({
			type: 'error',
			title: 'Internal Error',
			text: err.toString(),
		});
	}
}
onDeactivated(() => {
	if (aiscript) aiscript.abort();
});
onUnmounted(() => {
	if (aiscript) aiscript.abort();
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
const showns = computed(() => {
	if (root.value == null) return new Set<string>();
	const result = new Set<string>();
	(function addChildrenToResult(c: AsUiComponent) {
		result.add(c.id);
		const children = c.children;
		if (children) {
			const childComponents = components.value.filter(v => children.includes(v.value.id));
			for (const child of childComponents) {
				addChildrenToResult(child.value);
			}
		}
	})(root.value);
	return result;
});
definePage(() => ({
	title: i18n.ts.scratchpad,
	icon: 'ti ti-terminal-2',
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, null, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _createElementVNode("div", { class: "_gaps" }, [
            _createElementVNode("div", { class: "_gaps_s" }, [
              _createElementVNode("div", {
                class: _normalizeClass(["_panel", _ctx.$style.editor])
              }, [
                _createVNode(MkCodeEditor, {
                  lang: "aiscript",
                  modelValue: code.value,
                  "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((code).value = $event))
                }, null, 8 /* PROPS */, ["modelValue"])
              ]),
              _createVNode(MkButton, {
                primary: "",
                onClick: _cache[1] || (_cache[1] = ($event: any) => (run()))
              }, {
                default: _withCtx(() => [
                  _hoisted_1
                ]),
                _: 1 /* STABLE */
              })
            ]),
            (root.value && components.value.length > 1)
              ? (_openBlock(), _createBlock(MkContainer, {
                key: uiKey.value,
                foldable: true
              }, {
                header: _withCtx(() => [
                  _createTextVNode("UI")
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.ui)
                  }, [
                    _createVNode(MkAsUi, {
                      component: root.value,
                      components: components.value,
                      size: "small"
                    }, null, 8 /* PROPS */, ["component", "components"])
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["foldable"]))
              : _createCommentVNode("v-if", true),
            _createVNode(MkContainer, {
              foldable: true,
              class: ""
            }, {
              header: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.output), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.logs)
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(logs.value, (log) => {
                    return (_openBlock(), _createElementBlock("div", {
                      key: log.id,
                      class: _normalizeClass(["log", { print: log.print }])
                    }, _toDisplayString(log.text), 3 /* TEXT, CLASS */))
                  }), 128 /* KEYED_FRAGMENT */))
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["foldable"]),
            _createVNode(MkContainer, {
              foldable: true,
              expanded: false
            }, {
              header: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.uiInspector), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.uiInspector)
                }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(components.value, (c) => {
                    return (_openBlock(), _createElementBlock("div", {
                      key: c.value.id,
                      class: _normalizeClass({ [_ctx.$style.uiInspectorUnShown]: !showns.value.has(c.value.id) })
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.uiInspectorType)
                      }, _toDisplayString(c.value.type), 1 /* TEXT */),
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.uiInspectorId)
                      }, _toDisplayString(c.value.id), 1 /* TEXT */),
                      _createElementVNode("button", {
                        class: _normalizeClass(_ctx.$style.uiInspectorPropsToggle),
                        onClick: _cache[2] || (_cache[2] = () => uiInspectorOpenedComponents.value.set(c, !uiInspectorOpenedComponents.value.get(c)))
                      }, [
                        (uiInspectorOpenedComponents.value.get(c))
                          ? (_openBlock(), _createElementBlock("i", {
                            key: 0,
                            class: "ti ti-chevron-up icon"
                          }))
                          : (_openBlock(), _createElementBlock("i", {
                            key: 1,
                            class: "ti ti-chevron-down icon"
                          }))
                      ]),
                      (uiInspectorOpenedComponents.value.get(c))
                        ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                          _createVNode(MkTextarea, {
                            modelValue: stringifyUiProps(c.value),
                            code: "",
                            readonly: ""
                          }, null, 8 /* PROPS */, ["modelValue"])
                        ]))
                        : _createCommentVNode("v-if", true)
                    ], 2 /* CLASS */))
                  }), 128 /* KEYED_FRAGMENT */)),
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.uiInspectorDescription)
                  }, _toDisplayString(_unref(i18n).ts.uiInspectorDescription), 1 /* TEXT */)
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["foldable", "expanded"]),
            _createElementVNode("div", _hoisted_2, _toDisplayString(_unref(i18n).ts.scratchpadDescription), 1 /* TEXT */)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
