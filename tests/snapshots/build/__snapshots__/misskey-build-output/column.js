import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, renderSlot as _renderSlot, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "M149.512,4.707L108.507,4.707C116.252,4.719 118.758,14.958 118.758,14.958C118.758,14.958 121.381,25.283 129.009,25.209L149.512,25.209L149.512,4.707Z", style: "fill:var(--MI_THEME-deckBg);" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-up" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("path", { fill: "currentColor", d: "M10 13a1 1 0 1 1 0-2 1 1 0 0 1 0 2Zm0-4a1 1 0 1 1 0-2 1 1 0 0 1 0 2Zm-4 4a1 1 0 1 1 0-2 1 1 0 0 1 0 2Zm5-9a1 1 0 1 1-2 0 1 1 0 0 1 2 0ZM7 8a1 1 0 1 1-2 0 1 1 0 0 1 2 0ZM6 5a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-dots" })
import { onBeforeUnmount, onMounted, provide, watch, useTemplateRef, ref, computed } from 'vue'
import type { Column } from '@/deck.js'
import type { MenuItem } from '@/types/menu.js'
import { updateColumn, swapLeftColumn, swapRightColumn, swapUpColumn, swapDownColumn, stackLeftColumn, popRightColumn, removeColumn, swapColumn } from '@/deck.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import { DI } from '@/di.js'
import { checkDragDataType, getDragData, setDragData } from '@/drag-and-drop.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'column',
  props: {
    column: { type: null, required: true },
    isStacked: { type: Boolean, required: false, default: false },
    naked: { type: Boolean, required: false, default: false },
    handleScrollToTop: { type: Boolean, required: false, default: true },
    menu: { type: Array, required: false },
    refresher: { type: Function, required: false }
  },
  emits: ["headerWheel", "headerClick"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
provide('shouldHeaderThin', true);
provide('shouldOmitHeaderTitle', true);
const withWallpaper = prefer.s['deck.wallpaper'] != null;
const body = useTemplateRef('body');
const dragging = ref(false);
watch(dragging, v => os.deckGlobalEvents.emit(v ? 'column.dragStart' : 'column.dragEnd'));
const draghover = ref(false);
const dropready = ref(false);
const isMainColumn = computed(() => props.column.type === 'main');
const active = computed(() => props.column.active !== false);
onMounted(() => {
	os.deckGlobalEvents.on('column.dragStart', onOtherDragStart);
	os.deckGlobalEvents.on('column.dragEnd', onOtherDragEnd);
});
onBeforeUnmount(() => {
	os.deckGlobalEvents.off('column.dragStart', onOtherDragStart);
	os.deckGlobalEvents.off('column.dragEnd', onOtherDragEnd);
});
function onOtherDragStart() {
	dropready.value = true;
}
function onOtherDragEnd() {
	dropready.value = false;
}
function toggleActive() {
	if (!props.isStacked) return;
	updateColumn(props.column.id, {
		active: props.column.active == null ? false : !props.column.active,
	});
}
function getMenu() {
	const menuItems: MenuItem[] = [];
	if (props.menu) {
		menuItems.push(...props.menu);
	}
	if (props.refresher) {
		menuItems.push({
			icon: 'ti ti-refresh',
			text: i18n.ts.reload,
			action: () => {
				if (props.refresher) {
					props.refresher();
				}
			},
		});
	}
	if (menuItems.length > 0) {
		menuItems.push({
			type: 'divider',
		});
	}
	menuItems.push({
		icon: 'ti ti-settings',
		text: i18n.ts._deck.configureColumn,
		action: async () => {
			const name = props.column.name ?? i18n.ts._deck._columns[props.column.type];
			const { canceled, result } = await os.form(name, {
				name: {
					type: 'string',
					label: i18n.ts.name,
					default: props.column.name,
				},
				width: {
					type: 'number',
					label: i18n.ts.width,
					description: i18n.ts._deck.usedAsMinWidthWhenFlexible,
					default: props.column.width,
				},
				flexible: {
					type: 'boolean',
					label: i18n.ts._deck.flexible,
					default: props.column.flexible ?? null,
				},
			});
			if (canceled) return;
			updateColumn(props.column.id, result);
		},
	});
	const flexibleRef = ref(props.column.flexible ?? false);
	watch(flexibleRef, flexible => {
		updateColumn(props.column.id, {
			flexible,
		});
	});
	menuItems.push({
		type: 'switch',
		icon: 'ti ti-arrows-horizontal',
		text: i18n.ts._deck.flexible,
		ref: flexibleRef,
	});
	const moveToMenuItems: MenuItem[] = [];
	moveToMenuItems.push({
		icon: 'ti ti-arrow-left',
		text: i18n.ts._deck.swapLeft,
		action: () => {
			swapLeftColumn(props.column.id);
		},
	}, {
		icon: 'ti ti-arrow-right',
		text: i18n.ts._deck.swapRight,
		action: () => {
			swapRightColumn(props.column.id);
		},
	});
	if (props.isStacked) {
		moveToMenuItems.push({
			icon: 'ti ti-arrow-up',
			text: i18n.ts._deck.swapUp,
			action: () => {
				swapUpColumn(props.column.id);
			},
		}, {
			icon: 'ti ti-arrow-down',
			text: i18n.ts._deck.swapDown,
			action: () => {
				swapDownColumn(props.column.id);
			},
		});
	}
	menuItems.push({
		type: 'parent',
		text: i18n.ts.move + '...',
		icon: 'ti ti-arrows-move',
		children: moveToMenuItems,
	}, {
		icon: 'ti ti-stack-2',
		text: i18n.ts._deck.stackLeft,
		action: () => {
			stackLeftColumn(props.column.id);
		},
	});
	if (props.isStacked) {
		menuItems.push({
			icon: 'ti ti-window-maximize',
			text: i18n.ts._deck.popRight,
			action: () => {
				popRightColumn(props.column.id);
			},
		});
	}
	menuItems.push({ type: 'divider' }, {
		icon: 'ti ti-trash',
		text: i18n.ts.remove,
		danger: true,
		action: () => {
			removeColumn(props.column.id);
		},
	});
	return menuItems;
}
function showSettingsMenu(ev: PointerEvent) {
	os.popupMenu(getMenu(), ev.currentTarget ?? ev.target);
}
function onContextmenu(ev: PointerEvent) {
	os.contextMenu(getMenu(), ev);
}
function goTop(ev: PointerEvent) {
	emit('headerClick', ev);
	if (!props.handleScrollToTop) return;
	if (body.value) {
		body.value.scrollTo({
			top: 0,
			behavior: 'smooth',
		});
	}
}
function onDragstart(ev: DragEvent) {
	if (ev.dataTransfer == null) return;
	ev.dataTransfer.effectAllowed = 'move';
	setDragData(ev, 'deckColumn', props.column.id);
	// Chromeのバグで、Dragstartハンドラ内ですぐにDOMを変更する(=リアクティブなプロパティを変更する)とDragが終了してしまう
	// SEE: https://stackoverflow.com/questions/19639969/html5-dragend-event-firing-immediately
	window.setTimeout(() => {
		dragging.value = true;
	}, 10);
}
function onDragend(ev: DragEvent) {
	dragging.value = false;
}
function onDragover(ev: DragEvent) {
	if (ev.dataTransfer == null) return;
	// 自分自身がドラッグされている場合
	if (dragging.value) {
		// 自分自身にはドロップさせない
		ev.dataTransfer.dropEffect = 'none';
	} else {
		const isDeckColumn = checkDragDataType(ev, ['deckColumn']);
		ev.dataTransfer.dropEffect = isDeckColumn ? 'move' : 'none';
		if (isDeckColumn) draghover.value = true;
	}
}
function onDragleave() {
	draghover.value = false;
}
function onDrop(ev: DragEvent) {
	draghover.value = false;
	os.deckGlobalEvents.emit('column.dragEnd');
	const id = getDragData(ev, 'deckColumn');
	if (id != null) {
		swapColumn(props.column.id, id);
	}
}

return (_ctx: any,_cache: any) => {
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_forceShrinkSpacer", [_ctx.$style.root, { [_ctx.$style.paged]: isMainColumn.value, [_ctx.$style.naked]: __props.naked, [_ctx.$style.active]: active.value, [_ctx.$style.draghover]: draghover.value, [_ctx.$style.dragging]: dragging.value, [_ctx.$style.dropready]: dropready.value, [_ctx.$style.withWallpaper]: _unref(withWallpaper) }]]),
      onDragover: _withModifiers(onDragover, ["prevent","stop"]),
      onDragleave: onDragleave,
      onDrop: _withModifiers(onDrop, ["prevent","stop"])
    }, [ _createElementVNode("header", {
        class: _normalizeClass([_ctx.$style.header]),
        draggable: "true",
        onClick: goTop,
        onDragstart: onDragstart,
        onDragend: onDragend,
        onContextmenu: _withModifiers(onContextmenu, ["prevent","stop"]),
        onWheelPassive: _cache[0] || (_cache[0] = ($event: any) => (emit('headerWheel', $event)))
      }, [ _createElementVNode("svg", {
          viewBox: "0 0 256 128",
          class: _normalizeClass(_ctx.$style.tabShape)
        }, [ _createElementVNode("g", { transform: "matrix(6.2431,0,0,6.2431,-677.417,-29.3839)" }, [ _hoisted_1 ]) ]), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.color)
        }), (__props.isStacked && !isMainColumn.value) ? (_openBlock(), _createElementBlock("button", {
            key: 0,
            class: _normalizeClass(["_button", _ctx.$style.toggleActive]),
            onClick: toggleActive
          }, [ (active.value) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _hoisted_2 ], 64 /* STABLE_FRAGMENT */)) : (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _hoisted_3 ], 64 /* STABLE_FRAGMENT */)) ])) : _createCommentVNode("v-if", true), _createElementVNode("span", {
          class: _normalizeClass(_ctx.$style.title)
        }, [ _renderSlot(_ctx.$slots, "header") ]), _createElementVNode("svg", {
          viewBox: "0 0 16 16",
          version: "1.1",
          class: _normalizeClass(_ctx.$style.grabber)
        }, [ _hoisted_4 ]), _createElementVNode("button", {
          class: _normalizeClass(["_button", _ctx.$style.menu]),
          onClick: _withModifiers(showSettingsMenu, ["stop"])
        }, [ _hoisted_5 ]) ], 32 /* NEED_HYDRATION */), (active.value) ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          ref: "body",
          class: _normalizeClass(_ctx.$style.body)
        }, [ _renderSlot(_ctx.$slots, "default") ])) : _createCommentVNode("v-if", true) ], 34 /* CLASS, NEED_HYDRATION */))
}
}

})
