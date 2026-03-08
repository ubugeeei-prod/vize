import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref, withModifiers as _withModifiers } from "vue"

import { computed, defineAsyncComponent, nextTick, onMounted, onUnmounted, ref, useTemplateRef, toRefs, watch } from 'vue'
import type { Size } from '@/components/grid/grid.js'
import type { CellValue, GridCell } from '@/components/grid/cell.js'
import type { GridRowSetting } from '@/components/grid/row.js'
import { GridEventEmitter } from '@/components/grid/grid.js'
import { useTooltip } from '@/composables/use-tooltip.js'
import * as os from '@/os.js'
import { equalCellAddress, getCellAddress } from '@/components/grid/grid-utils.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkDataCell',
  props: {
    cell: { type: null, required: true },
    rowSetting: { type: null, required: true },
    bus: { type: null, required: true }
  },
  emits: ["operation:beginEdit", "operation:endEdit", "change:value", "change:contentSize"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { cell, bus } = toRefs(props);
const rootEl = useTemplateRef('rootEl');
const contentAreaEl = useTemplateRef('contentAreaEl');
const inputAreaEl = useTemplateRef('inputAreaEl');
/** 値が編集中かどうか */
const editing = ref<boolean>(false);
/** 編集中の値. {@link beginEditing}と{@link endEditing}内、および各inputタグやそのコールバックからの操作のみを想定する */
const editingValue = ref<CellValue>(undefined);
const cellWidth = computed(() => cell.value.column.width);
const cellType = computed(() => cell.value.column.setting.type);
const needsContentCentering = computed(() => {
	switch (cellType.value) {
		case 'boolean':
			return true;
		default:
			return false;
	}
});
watch(() => [cell.value.value], () => {
	// 中身がセットされた直後はサイズが分からないので、次のタイミングで更新する
	nextTick(emitContentSizeChanged);
}, { immediate: true });
watch(() => cell.value.selected, () => {
	if (cell.value.selected) {
		requestFocus();
	}
});
function onCellDoubleClick(ev: MouseEvent) {
	switch (ev.type) {
		case 'dblclick': {
			beginEditing(ev.target as HTMLElement);
			break;
		}
	}
}
function onOutsideMouseDown(ev: MouseEvent) {
	const isOutside = ev.target instanceof Node && !rootEl.value?.contains(ev.target);
	if (isOutside || !equalCellAddress(cell.value.address, getCellAddress(ev.target as HTMLElement))) {
		endEditing(true, false);
	}
}
function onCellKeyDown(ev: KeyboardEvent) {
	if (!editing.value) {
		ev.preventDefault();
		switch (ev.code) {
			case 'NumpadEnter':
			case 'Enter':
			case 'F2': {
				beginEditing(ev.target as HTMLElement);
				break;
			}
		}
	} else {
		switch (ev.code) {
			case 'Escape': {
				endEditing(false, true);
				break;
			}
			case 'NumpadEnter':
			case 'Enter': {
				if (!ev.isComposing) {
					endEditing(true, true);
				}
			}
		}
	}
}
function onInputText(ev: InputEvent) {
	editingValue.value = (ev.target as HTMLInputElement).value;
}
function onForceRefreshContentSize() {
	emitContentSizeChanged();
}
function registerOutsideMouseDown() {
	unregisterOutsideMouseDown();
	addEventListener('mousedown', onOutsideMouseDown);
}
function unregisterOutsideMouseDown() {
	removeEventListener('mousedown', onOutsideMouseDown);
}
async function beginEditing(target: HTMLElement) {
	if (editing.value || !cell.value.selected || !cell.value.column.setting.editable) {
		return;
	}
	if (cell.value.column.setting.customValueEditor) {
		emit('operation:beginEdit', cell.value);
		const newValue = await cell.value.column.setting.customValueEditor(
			cell.value.row,
			cell.value.column,
			cell.value.value,
			target,
		);
		emit('operation:endEdit', cell.value);
		if (newValue !== cell.value.value) {
			emitValueChange(newValue);
		}
		requestFocus();
	} else {
		switch (cellType.value) {
			case 'number':
			case 'date':
			case 'text': {
				editingValue.value = cell.value.value;
				editing.value = true;
				registerOutsideMouseDown();
				emit('operation:beginEdit', cell.value);
				await nextTick(() => {
					// inputの展開後にフォーカスを当てたい
					if (inputAreaEl.value) {
						(inputAreaEl.value.querySelector('*') as HTMLElement).focus();
					}
				});
				break;
			}
			case 'boolean': {
				// とくに特殊なUIは設けず、トグルするだけ
				emitValueChange(!cell.value.value);
				break;
			}
		}
	}
}
function endEditing(applyValue: boolean, requireFocus: boolean) {
	if (!editing.value) {
		return;
	}
	const newValue = editingValue.value;
	editingValue.value = undefined;
	emit('operation:endEdit', cell.value);
	unregisterOutsideMouseDown();
	if (applyValue && newValue !== cell.value.value) {
		emitValueChange(newValue);
	}
	editing.value = false;
	if (requireFocus) {
		requestFocus();
	}
}
function requestFocus() {
	nextTick(() => {
		rootEl.value?.focus();
	});
}
function emitValueChange(newValue: CellValue) {
	const _cell = cell.value;
	emit('change:value', _cell, newValue);
}
function emitContentSizeChanged() {
	emit('change:contentSize', cell.value, {
		width: contentAreaEl.value?.clientWidth ?? 0,
		height: contentAreaEl.value?.clientHeight ?? 0,
	});
}
useTooltip(rootEl, (showing) => {
	if (cell.value.violation.valid) {
		return;
	}
	const content = cell.value.violation.violations.filter(it => !it.valid).map(it => it.result.message).join('\n');
	const result = os.popup(defineAsyncComponent(() => import('@/components/grid/MkCellTooltip.vue')), {
		showing,
		content,
		anchorElement: rootEl.value!,
	}, {
		closed: () => {
			result.dispose();
		},
	});
});
onMounted(() => {
	bus.value.on('forceRefreshContentSize', onForceRefreshContentSize);
});
onUnmounted(() => {
	bus.value.off('forceRefreshContentSize', onForceRefreshContentSize);
});

return (_ctx: any,_cache: any) => {
  return (_unref(cell).row.using)
      ? (_openBlock(), _createElementBlock("div", {
        key: 0,
        ref: "rootEl",
        class: _normalizeClass(["mk_grid_td", _ctx.$style.cell]),
        style: _normalizeStyle({ maxWidth: cellWidth.value, minWidth: cellWidth.value }),
        tabindex: -1,
        "data-grid-cell": "",
        "data-grid-cell-row": _unref(cell).row.index,
        "data-grid-cell-col": _unref(cell).column.index,
        onKeydown: onCellKeyDown,
        onDblclick: _withModifiers(onCellDoubleClick, ["prevent"])
      }, [ _createElementVNode("div", {
          class: _normalizeClass([
  			_ctx.$style.root,
  			[(_unref(cell).violation.valid || _unref(cell).selected) ? {} : _ctx.$style.error],
  			[_unref(cell).selected ? _ctx.$style.selected : {}],
  			/*  行が選択されているときは範囲選択色の適用を行側に任せる */
  			[(_unref(cell).ranged && !_unref(cell).row.ranged) ? _ctx.$style.ranged : {}],
  			[needsContentCentering.value ? _ctx.$style.center : {}],
  		])
        }, [ (!editing.value) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: _normalizeClass([_ctx.$style.contentArea]),
              style: _normalizeStyle(cellType.value === 'boolean' ? 'justify-content: center' : '')
            }, [ _createElementVNode("div", {
                ref_key: "contentAreaEl", ref: contentAreaEl,
                class: _normalizeClass(_ctx.$style.content)
              }, [ (cellType.value === 'text') ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(cell).value), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (cellType.value === 'number') ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(cell).value), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (cellType.value === 'date') ? (_openBlock(), _createElementBlock("div", { key: 0 }, _toDisplayString(_unref(cell).value), 1 /* TEXT */)) : (cellType.value === 'boolean') ? (_openBlock(), _createElementBlock("div", { key: 1 }, [ _createElementVNode("div", {
                        class: _normalizeClass([_ctx.$style.bool, {
  							[_ctx.$style.boolTrue]: _unref(cell).value === true,
  							'ti ti-check': _unref(cell).value === true,
  						}])
                      }, null, 2 /* CLASS */) ])) : (cellType.value === 'image') ? (_openBlock(), _createElementBlock("div", { key: 2 }, [ (_unref(cell).value && typeof _unref(cell).value === 'string') ? (_openBlock(), _createElementBlock("img", {
                          key: 0,
                          src: _unref(cell).value,
                          alt: _unref(cell).value,
                          class: _normalizeClass(_ctx.$style.viewImage),
                          onLoad: emitContentSizeChanged
                        })) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ], 512 /* NEED_PATCH */) ])) : (_openBlock(), _createElementBlock("div", {
              key: 1,
              ref: "inputAreaEl",
              class: _normalizeClass(_ctx.$style.inputArea)
            }, [ (cellType.value === 'text') ? (_openBlock(), _createElementBlock("input", {
                  key: 0,
                  type: "text",
                  class: _normalizeClass(_ctx.$style.editingInput),
                  value: editingValue.value,
                  onInput: onInputText,
                  onMousedown: _withModifiers(() => {}, ["stop"]),
                  onContextmenu: _withModifiers(() => {}, ["stop"])
                })) : _createCommentVNode("v-if", true), (cellType.value === 'number') ? (_openBlock(), _createElementBlock("input", {
                  key: 0,
                  type: "number",
                  class: _normalizeClass(_ctx.$style.editingInput),
                  value: editingValue.value,
                  onInput: onInputText,
                  onMousedown: _withModifiers(() => {}, ["stop"]),
                  onContextmenu: _withModifiers(() => {}, ["stop"])
                })) : _createCommentVNode("v-if", true), (cellType.value === 'date') ? (_openBlock(), _createElementBlock("input", {
                  key: 0,
                  type: "date",
                  class: _normalizeClass(_ctx.$style.editingInput),
                  value: editingValue.value,
                  onInput: onInputText,
                  onMousedown: _withModifiers(() => {}, ["stop"]),
                  onContextmenu: _withModifiers(() => {}, ["stop"])
                })) : _createCommentVNode("v-if", true) ])) ], 2 /* CLASS */) ]))
      : _createCommentVNode("v-if", true)
}
}

})
