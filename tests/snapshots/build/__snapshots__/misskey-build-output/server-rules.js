import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-checkbox" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-menu" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { ref } from 'vue'
import * as os from '@/os.js'
import { fetchInstance, instance } from '@/instance.js'
import { i18n } from '@/i18n.js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkDraggable from '@/components/MkDraggable.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'server-rules',
  setup(__props) {

const serverRules = ref<{ text: string; id: string; }[]>(instance.serverRules.map(text => ({ text, id: Math.random().toString() })));
async function save() {
	await os.apiWithDialog('admin/update-meta', {
		serverRules: serverRules.value.map(r => r.text),
	});
	fetchInstance(true);
}
function add(): void {
	serverRules.value.push({ text: '', id: Math.random().toString() });
}
function remove(id: string): void {
	serverRules.value = serverRules.value.filter(r => r.id !== id);
}

return (_ctx: any,_cache: any) => {
  const _component_SearchIcon = _resolveComponent("SearchIcon")
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchText = _resolveComponent("SearchText")
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      markerId: "serverRules",
      keywords: ['rules']
    }, {
      default: _withCtx(() => [
        _createVNode(MkFolder, null, {
          icon: _withCtx(() => [
            _createVNode(_component_SearchIcon, null, {
              default: _withCtx(() => [
                _hoisted_1
              ]),
              _: 1 /* STABLE */
            })
          ]),
          label: _withCtx(() => [
            _createVNode(_component_SearchLabel, null, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.serverRules), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", { class: "_gaps_m" }, [
              _createElementVNode("div", null, [
                _createVNode(_component_SearchText, null, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._serverRules.description), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ]),
              _createVNode(MkDraggable, {
                direction: "vertical",
                withGaps: "",
                manualDragStart: "",
                modelValue: serverRules.value,
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((serverRules).value = $event))
              }, {
                default: _withCtx(({ item, index, dragStart }) => [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.item)
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(_ctx.$style.itemHeader)
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(_ctx.$style.itemNumber)
                      }, _toDisplayString(index + 1), 1 /* TEXT */),
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.itemHandle),
                        draggable: true,
                        onDragstart: _cache[1] || (_cache[1] = _withModifiers((...args) => (_ctx.dragStart && _ctx.dragStart(...args)), ["stop"]))
                      }, [
                        _hoisted_2
                      ], 40 /* PROPS, NEED_HYDRATION */, ["draggable"]),
                      _createElementVNode("button", {
                        class: _normalizeClass(["_button", _ctx.$style.itemRemove]),
                        onClick: _cache[2] || (_cache[2] = ($event: any) => (remove(_ctx.item.id)))
                      }, [
                        _hoisted_3
                      ])
                    ]),
                    _createVNode(MkInput, {
                      modelValue: item.text,
                      "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => (serverRules.value[_ctx.index].text = $event))
                    }, null, 8 /* PROPS */, ["modelValue"])
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"]),
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.commands)
              }, [
                _createVNode(MkButton, {
                  rounded: "",
                  onClick: add
                }, {
                  default: _withCtx(() => [
                    _hoisted_4,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }),
                _createVNode(MkButton, {
                  primary: "",
                  rounded: "",
                  onClick: save
                }, {
                  default: _withCtx(() => [
                    _hoisted_5,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ])
          ]),
          _: 1 /* STABLE */
        })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["keywords"]))
}
}

})
