import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
import type { DefaultStoredWidget, Widget } from '@/components/MkWidgets.vue'
import XWidgets from '@/components/MkWidgets.vue'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'

import { computed, ref } from 'vue';
const editMode = ref(false);

export default /*@__PURE__*/_defineComponent({
  __name: 'widgets',
  props: {
    place: { type: String, required: false, default: null }
  },
  setup(__props: any) {

const props = __props
const widgets = computed(() => {
	if (props.place === null) return prefer.r.widgets.value;
	if (props.place === 'left') return prefer.r.widgets.value.filter(w => w.place === 'left');
	return prefer.r.widgets.value.filter(w => w.place !== 'left');
});
function addWidget(widget: Widget) {
	prefer.commit('widgets', [{
		...widget,
		place: props.place,
	}, ...prefer.s.widgets]);
}
function removeWidget(widget: Widget) {
	prefer.commit('widgets', prefer.s.widgets.filter(w => w.id !== widget.id));
}
function updateWidget(widget: { id: Widget['id']; data: Widget['data']; }) {
	prefer.commit('widgets', prefer.s.widgets.map(w => w.id === widget.id ? {
		...w,
		data: widget.data,
		place: props.place,
	} : w));
}
function updateWidgets(thisWidgets: Widget[]) {
	if (props.place === null) {
		prefer.commit('widgets', thisWidgets as DefaultStoredWidget[]);
		return;
	}
	if (props.place === 'left') {
		prefer.commit('widgets', [
			...thisWidgets.map(w => ({ ...w, place: 'left' })),
			...prefer.s.widgets.filter(w => w.place !== 'left' && !thisWidgets.some(t => w.id === t.id)),
		]);
		return;
	}
	prefer.commit('widgets', [
		...prefer.s.widgets.filter(w => w.place === 'left' && !thisWidgets.some(t => w.id === t.id)),
		...thisWidgets.map(w => ({ ...w, place: 'right' })),
	]);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(XWidgets, {
        edit: _ctx.editMode,
        widgets: widgets.value,
        onAddWidget: addWidget,
        onRemoveWidget: removeWidget,
        onUpdateWidget: updateWidget,
        onUpdateWidgets: updateWidgets,
        onExit: _cache[0] || (_cache[0] = ($event: any) => (_ctx.editMode = false))
      }, null, 8 /* PROPS */, ["edit", "widgets"]), (_ctx.editMode) ? (_openBlock(), _createElementBlock("button", {
          key: 0,
          class: "_textButton",
          style: "font-size: 0.9em;",
          onClick: _cache[1] || (_cache[1] = ($event: any) => (_ctx.editMode = false))
        }, [ _hoisted_1, _createTextVNode(), _toDisplayString(_unref(i18n).ts.editWidgetsExit) ])) : (_openBlock(), _createElementBlock("button", {
          key: 1,
          "data-cy-widget-edit": "",
          class: _normalizeClass(["_textButton", _ctx.$style.edit]),
          style: "font-size: 0.9em; margin-top: 16px;",
          onClick: _cache[2] || (_cache[2] = ($event: any) => (_ctx.editMode = true))
        }, [ _hoisted_2, _createTextVNode(), _toDisplayString(_unref(i18n).ts.editWidgets) ])) ]))
}
}

})
