import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-settings" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-x" });
import { computed } from "vue";
import { isLink } from "@@/js/is-link.js";
import { genId } from "@/utility/id.js";
import MkSelect from "@/components/MkSelect.vue";
import MkButton from "@/components/MkButton.vue";
import MkDraggable from "@/components/MkDraggable.vue";
import { widgets as widgetDefs, federationWidgets } from "@/widgets/index.js";
import * as os from "@/os.js";
import { i18n } from "@/i18n.js";
import { instance } from "@/instance.js";
import { useMkSelect } from "@/composables/use-mkselect.js";
export {};
export default {
  __name: "MkWidgets",
  props: {
    widgets: {
      type: Array,
      required: true
    },
    edit: {
      type: Boolean,
      required: true
    }
  },
  emits: ["data"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const _widgetDefs = computed(() => {
      if (instance.federation === "none") {
        return widgetDefs.filter((x) => !federationWidgets.includes(x));
      } else {
        return widgetDefs;
      }
    });
    const _widgets = computed(() => props.widgets.filter((x) => _widgetDefs.value.includes(x.name)));
    const widgetRefs = {};
    function configWidget(id) {
      widgetRefs[id].configure();
    }
    const { model: widgetAdderSelected, def: widgetAdderSelectedDef } = useMkSelect({
      items: computed(() => [{
        label: i18n.ts.none,
        value: null
      }, ..._widgetDefs.value.map((x) => ({
        label: i18n.ts._widgets[x],
        value: x
      }))]),
      initialValue: null
    });
    function addWidget() {
      if (widgetAdderSelected.value == null) return;
      emit("addWidget", {
        name: widgetAdderSelected.value,
        id: genId(),
        data: {}
      });
      widgetAdderSelected.value = null;
    }
    function removeWidget(widget) {
      emit("removeWidget", widget);
    }
    function updateWidget(id, data) {
      emit("updateWidget", {
        id,
        data
      });
    }
    function onContextmenu(widget, ev) {
      const element = ev.target;
      if (element && isLink(element)) return;
      if (element && ([
        "INPUT",
        "TEXTAREA",
        "IMG",
        "VIDEO",
        "CANVAS"
      ].includes(element.tagName) || element.attributes.getNamedItem("contenteditable") != null)) return;
      if (window.getSelection()?.toString() !== "") return;
      os.contextMenu([{
        type: "label",
        text: i18n.ts._widgets[widget.name]
      }, {
        icon: "ti ti-settings",
        text: i18n.ts.settings,
        action: () => {
          configWidget(widget.id);
        }
      }], ev);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(["_gaps_s", _ctx.$style.root]) },
        [__props.edit ? (_openBlock(), _createElementBlock(
          _Fragment,
          { key: 0 },
          [_createElementVNode(
            "header",
            { class: _normalizeClass(_ctx.$style.editHeader) },
            [
              _createVNode(MkSelect, {
                items: _unref(widgetAdderSelectedDef),
                style: "margin-bottom: var(--MI-margin)",
                "data-cy-widget-select": "",
                modelValue: _unref(widgetAdderSelected),
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => widgetAdderSelected.value = $event)
              }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.selectWidget),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["items", "modelValue"]),
              _createVNode(MkButton, {
                inline: "",
                primary: "",
                "data-cy-widget-add": "",
                onClick: addWidget
              }, {
                default: _withCtx(() => [
                  _hoisted_1,
                  _createTextVNode(" "),
                  _createTextVNode(
                    _toDisplayString(_unref(i18n).ts.add),
                    1
                    /* TEXT */
                  )
                ]),
                _: 1
              }),
              _createVNode(MkButton, {
                inline: "",
                onClick: _cache[1] || (_cache[1] = ($event) => emit("exit"))
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.close),
                  1
                  /* TEXT */
                )]),
                _: 1
              })
            ],
            2
            /* CLASS */
          ), _createVNode(MkDraggable, {
            modelValue: props.widgets,
            direction: "vertical",
            withGaps: "",
            group: "MkWidgets",
            "onUpdate:modelValue": _cache[2] || (_cache[2] = (v) => emit("updateWidgets", v))
          }, {
            default: _withCtx(({ item }) => [_createElementVNode(
              "div",
              {
                class: _normalizeClass([_ctx.$style.widget, _ctx.$style.customizeContainer]),
                "data-cy-customize-container": ""
              },
              [
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.customizeContainerConfig]),
                  onClick: _withModifiers(($event) => configWidget(item.id), ["prevent", "stop"])
                }, [_hoisted_2], 10, ["onClick"]),
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.customizeContainerRemove]),
                  "data-cy-customize-container-remove": "",
                  onClick: _withModifiers(($event) => removeWidget(item), ["prevent", "stop"])
                }, [_hoisted_3], 10, ["onClick"]),
                _createVNode(_resolveDynamicComponent(`widget-${item.name}`), {
                  ref: (el) => _unref(widgetRefs)[item.id] = el,
                  class: _normalizeClass(_ctx.$style.customizeContainerHandleWidget),
                  widget: item,
                  onUpdateProps: ($event) => updateWidget(item.id, $event)
                }, null, 522, ["widget", "onUpdateProps"])
              ],
              2
              /* CLASS */
            )]),
            _: 1
          }, 8, ["modelValue"])],
          64
          /* STABLE_FRAGMENT */
        )) : (_openBlock(), _createElementBlock(
          _Fragment,
          { key: _ctx.widget.id },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(_widgets.value, (widget) => {
              return _openBlock(), _createBlock(_resolveDynamicComponent(`widget-${widget.name}`), {
                ref: (el) => _unref(widgetRefs)[widget.id] = el,
                class: _normalizeClass(_ctx.$style.widget),
                widget,
                onUpdateProps: ($event) => updateWidget(widget.id, $event),
                onContextmenu: _withModifiers(($event) => onContextmenu(widget, $event), ["stop"])
              }, null, 522, [
                "widget",
                "onUpdateProps",
                "onContextmenu"
              ]);
            }),
            256
            /* UNKEYED_FRAGMENT */
          ))],
          64
          /* STABLE_FRAGMENT */
        ))],
        2
        /* CLASS */
      );
    };
  }
};
