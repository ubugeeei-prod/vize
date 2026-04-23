import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-checkbox" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-menu" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-x" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { ref } from "vue";
import * as os from "@/os.js";
import { fetchInstance, instance } from "@/instance.js";
import { i18n } from "@/i18n.js";
import MkButton from "@/components/MkButton.vue";
import MkInput from "@/components/MkInput.vue";
import MkFolder from "@/components/MkFolder.vue";
import MkDraggable from "@/components/MkDraggable.vue";
export default {
  __name: "server-rules",
  setup(__props) {
    const serverRules = ref(instance.serverRules.map((text) => ({
      text,
      id: Math.random().toString()
    })));
    async function save() {
      await os.apiWithDialog("admin/update-meta", { serverRules: serverRules.value.map((r) => r.text) });
      fetchInstance(true);
    }
    function add() {
      serverRules.value.push({
        text: "",
        id: Math.random().toString()
      });
    }
    function remove(id) {
      serverRules.value = serverRules.value.filter((r) => r.id !== id);
    }
    return (_ctx, _cache) => {
      const _component_SearchIcon = _resolveComponent("SearchIcon");
      const _component_SearchLabel = _resolveComponent("SearchLabel");
      const _component_SearchText = _resolveComponent("SearchText");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      return _openBlock(), _createBlock(_component_SearchMarker, {
        markerId: "serverRules",
        keywords: ["rules"]
      }, {
        default: _withCtx(() => [_createVNode(MkFolder, null, {
          icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
            default: _withCtx(() => [_hoisted_1]),
            _: 1
          })]),
          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.serverRules),
              1
              /* TEXT */
            )]),
            _: 1
          })]),
          default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
            _createElementVNode("div", null, [_createVNode(_component_SearchText, null, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._serverRules.description),
                1
                /* TEXT */
              )]),
              _: 1
            })]),
            _createVNode(MkDraggable, {
              direction: "vertical",
              withGaps: "",
              manualDragStart: "",
              modelValue: serverRules.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => serverRules.value = $event)
            }, {
              default: _withCtx(({ item, index, dragStart }) => [_createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.item) },
                [_createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.itemHeader) },
                  [
                    _createElementVNode(
                      "div",
                      { class: _normalizeClass(_ctx.$style.itemNumber) },
                      _toDisplayString(index + 1),
                      3
                      /* TEXT, CLASS */
                    ),
                    _createElementVNode("span", {
                      class: _normalizeClass(_ctx.$style.itemHandle),
                      draggable: true,
                      onDragstart: _withModifiers(dragStart, ["stop"])
                    }, [_hoisted_2], 42, ["draggable", "onDragstart"]),
                    _createElementVNode("button", {
                      class: _normalizeClass(["_button", _ctx.$style.itemRemove]),
                      onClick: ($event) => remove(item.id)
                    }, [_hoisted_3], 10, ["onClick"])
                  ],
                  2
                  /* CLASS */
                ), _createVNode(MkInput, {
                  modelValue: item.text,
                  "onUpdate:modelValue": ($event) => serverRules.value[index].text = $event
                }, null, 8, ["modelValue", "onUpdate:modelValue"])],
                2
                /* CLASS */
              )]),
              _: 1
            }, 8, ["modelValue"]),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.commands) },
              [_createVNode(MkButton, {
                rounded: "",
                onClick: add
              }, {
                default: _withCtx(() => [
                  _hoisted_4,
                  _createTextVNode(" "),
                  _createTextVNode(
                    _toDisplayString(_unref(i18n).ts.add),
                    1
                    /* TEXT */
                  )
                ]),
                _: 1
              }), _createVNode(MkButton, {
                primary: "",
                rounded: "",
                onClick: save
              }, {
                default: _withCtx(() => [
                  _hoisted_5,
                  _createTextVNode(" "),
                  _createTextVNode(
                    _toDisplayString(_unref(i18n).ts.save),
                    1
                    /* TEXT */
                  )
                ]),
                _: 1
              })],
              2
              /* CLASS */
            )
          ])]),
          _: 1
        })]),
        _: 1
      }, 8, ["keywords"]);
    };
  }
};
