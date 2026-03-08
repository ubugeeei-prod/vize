import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-apps", style: "margin-right: 8px;" })
import { ref } from 'vue'
import XColumn from './column.vue'
import type { Column } from '@/deck.js'
import type { Widget } from '@/components/MkWidgets.vue'
import { addColumnWidget, removeColumnWidget, setColumnWidgets, updateColumnWidget } from '@/deck.js'
import XWidgets from '@/components/MkWidgets.vue'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'widgets-column',
  props: {
    column: { type: null, required: true },
    isStacked: { type: Boolean, required: true }
  },
  setup(__props: any) {

const props = __props
const edit = ref(false);
function addWidget(widget: Widget) {
	addColumnWidget(props.column.id, widget);
}
function removeWidget(widget: Widget) {
	removeColumnWidget(props.column.id, widget);
}
function updateWidget(widget: { id: Widget['id']; data: Widget['data']; }) {
	updateColumnWidget(props.column.id, widget.id, widget.data);
}
function updateWidgets(widgets: Widget[]) {
	setColumnWidgets(props.column.id, widgets);
}
function func() {
	edit.value = !edit.value;
}
const menu = [{
	icon: 'ti ti-pencil',
	text: i18n.ts.editWidgets,
	action: func,
}];

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XColumn, {
      menu: menu,
      naked: true,
      column: __props.column,
      isStacked: __props.isStacked
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createTextVNode(_toDisplayString(__props.column.name || _unref(i18n).ts._deck._columns[props.column.type]), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.root)
        }, [
          (!(__props.column.widgets && __props.column.widgets.length > 0) && !edit.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass(_ctx.$style.intro)
            }, _toDisplayString(_unref(i18n).ts._deck.widgetsIntroduction), 1 /* TEXT */))
            : _createCommentVNode("v-if", true),
          _createVNode(XWidgets, {
            edit: edit.value,
            widgets: __props.column.widgets ?? [],
            onAddWidget: addWidget,
            onRemoveWidget: removeWidget,
            onUpdateWidget: updateWidget,
            onUpdateWidgets: updateWidgets,
            onExit: _cache[0] || (_cache[0] = ($event: any) => (edit.value = false))
          }, null, 8 /* PROPS */, ["edit", "widgets"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["menu", "naked", "column", "isStacked"]))
}
}

})
