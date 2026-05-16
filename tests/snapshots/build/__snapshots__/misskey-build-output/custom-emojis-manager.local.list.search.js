import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", {
  class: "ti ti-search",
  style: "margin-right: 0.5em;"
});
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-pencil" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-arrows-sort" });
import { computed, ref, watch } from "vue";
import MkWindow from "@/components/MkWindow.vue";
import MkInput from "@/components/MkInput.vue";
import MkSelect from "@/components/MkSelect.vue";
import MkButton from "@/components/MkButton.vue";
import MkFolder from "@/components/MkFolder.vue";
import MkSortOrderEditor from "@/components/MkSortOrderEditor.vue";
import { gridSortOrderKeys } from "./custom-emojis-manager.impl.js";
import { i18n } from "@/i18n.js";
import * as os from "@/os.js";
export default {
  __name: "custom-emojis-manager.local.list.search",
  props: { query: {
    type: null,
    required: true
  } },
  emits: [
    "closed",
    "queryUpdated",
    "sortOrderUpdated",
    "search"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const model = ref(props.query);
    const queryRolesText = computed(() => model.value.roles.map((it) => it.name).join(","));
    watch(model, () => {
      emit("queryUpdated", model.value);
    }, { deep: true });
    const sortOrders = ref([]);
    function onSortOrderUpdate(orders) {
      sortOrders.value = orders;
      emit("sortOrderUpdated", orders);
    }
    function onSearchRequest() {
      emit("search");
    }
    function onQueryResetButtonClicked() {
      model.value.name = "";
      model.value.category = "";
      model.value.aliases = "";
      model.value.type = "";
      model.value.license = "";
      model.value.sensitive = null;
      model.value.localOnly = null;
      model.value.updatedAtFrom = "";
      model.value.updatedAtTo = "";
      sortOrders.value = [];
    }
    async function onQueryRolesEditClicked() {
      const result = await os.selectRole({
        initialRoleIds: model.value.roles.map((it) => it.id),
        title: i18n.ts._customEmojisManager._local._list.dialogSelectRoleTitle,
        publicOnly: true
      });
      if (result.canceled) {
        return;
      }
      model.value.roles = result.result;
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkWindow, {
        ref: "uiWindow",
        initialWidth: 400,
        initialHeight: 500,
        canResize: true,
        onClosed: _cache[0] || (_cache[0] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [
          _hoisted_1,
          _createTextVNode(" "),
          _createTextVNode(
            _toDisplayString(_unref(i18n).ts.search),
            1
            /* TEXT */
          )
        ]),
        default: _withCtx(() => [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.root) },
          [_createElementVNode("div", { class: "_spacer" }, [_createElementVNode("div", { class: "_gaps" }, [_createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(MkInput, {
              type: "search",
              autocapitalize: "off",
              modelValue: model.value.name,
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => model.value.name = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("name")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkInput, {
              type: "search",
              autocapitalize: "off",
              modelValue: model.value.category,
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => model.value.category = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("category")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkInput, {
              type: "search",
              autocapitalize: "off",
              modelValue: model.value.aliases,
              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => model.value.aliases = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("aliases")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkInput, {
              type: "search",
              autocapitalize: "off",
              modelValue: model.value.type,
              "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => model.value.type = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("type")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkInput, {
              type: "search",
              autocapitalize: "off",
              modelValue: model.value.license,
              "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => model.value.license = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("license")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkSelect, {
              items: [
                {
                  label: "-",
                  value: null
                },
                {
                  label: "true",
                  value: "true"
                },
                {
                  label: "false",
                  value: "false"
                }
              ],
              modelValue: model.value.sensitive,
              "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event) => model.value.sensitive = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("sensitive")]),
              _: 1
            }, 8, ["items", "modelValue"]),
            _createVNode(MkSelect, {
              items: [
                {
                  label: "-",
                  value: null
                },
                {
                  label: "true",
                  value: "true"
                },
                {
                  label: "false",
                  value: "false"
                }
              ],
              modelValue: model.value.localOnly,
              "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event) => model.value.localOnly = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("localOnly")]),
              _: 1
            }, 8, ["items", "modelValue"]),
            _createVNode(MkInput, {
              type: "date",
              autocapitalize: "off",
              modelValue: model.value.updatedAtFrom,
              "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event) => model.value.updatedAtFrom = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("updatedAt(from)")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkInput, {
              type: "date",
              autocapitalize: "off",
              modelValue: model.value.updatedAtTo,
              "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event) => model.value.updatedAtTo = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("updatedAt(to)")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkInput, {
              type: "text",
              readonly: "",
              autocapitalize: "off",
              onClick: onQueryRolesEditClicked,
              modelValue: queryRolesText.value,
              "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event) => queryRolesText.value = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("role")]),
              suffix: _withCtx(() => [_hoisted_2]),
              _: 1
            }, 8, ["modelValue"])
          ]), _createVNode(MkFolder, {
            spacerMax: 8,
            spacerMin: 8
          }, {
            icon: _withCtx(() => [_hoisted_3]),
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.sortOrder),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [_createVNode(MkSortOrderEditor, {
              baseOrderKeyNames: _unref(gridSortOrderKeys),
              currentOrders: sortOrders.value,
              onUpdate: onSortOrderUpdate
            }, null, 8, ["baseOrderKeyNames", "currentOrders"])]),
            _: 1
          }, 8, ["spacerMax", "spacerMin"])])]), _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.footerActions) },
            [_createVNode(MkButton, {
              primary: "",
              onClick: onSearchRequest
            }, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.search),
                1
                /* TEXT */
              )]),
              _: 1
            }), _createVNode(MkButton, { onClick: onQueryResetButtonClicked }, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.reset),
                1
                /* TEXT */
              )]),
              _: 1
            })],
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        )]),
        _: 1
      }, 8, [
        "initialWidth",
        "initialHeight",
        "canResize"
      ]);
    };
  }
};
