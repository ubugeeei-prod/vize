import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { style: "margin-left: 8px;" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-circle-minus" })
import { onMounted, watch, ref, useTemplateRef, computed } from 'vue'
import XColumn from './column.vue'
import type { Column } from '@/deck.js'
import type { MenuItem } from '@/types/menu.js'
import type { SoundStore } from '@/preferences/def.js'
import { removeColumn, updateColumn } from '@/deck.js'
import MkStreamingNotesTimeline from '@/components/MkStreamingNotesTimeline.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { hasWithReplies, isAvailableBasicTimeline, basicTimelineIconClass } from '@/timelines.js'
import { soundSettingsButton } from '@/ui/deck/tl-note-notification.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'tl-column',
  props: {
    column: { type: null, required: true },
    isStacked: { type: Boolean, required: true }
  },
  setup(__props: any) {

const props = __props
const timeline = useTemplateRef('timeline');
const soundSetting = ref<SoundStore>(props.column.soundSetting ?? { type: null, volume: 1 });
const withRenotes = ref(props.column.withRenotes ?? true);
const withReplies = ref(props.column.withReplies ?? false);
const withSensitive = ref(props.column.withSensitive ?? true);
const onlyFiles = ref(props.column.onlyFiles ?? false);
watch(withRenotes, v => {
	updateColumn(props.column.id, {
		withRenotes: v,
	});
});
watch(withReplies, v => {
	updateColumn(props.column.id, {
		withReplies: v,
	});
});
watch(withSensitive, v => {
	updateColumn(props.column.id, {
		withSensitive: v,
	});
});
watch(onlyFiles, v => {
	updateColumn(props.column.id, {
		onlyFiles: v,
	});
});
watch(soundSetting, v => {
	updateColumn(props.column.id, { soundSetting: v });
});
onMounted(() => {
	if (props.column.tl == null) {
		setType();
	}
});
async function setType() {
	const { canceled, result: src } = await os.select({
		title: i18n.ts.timeline,
		items: [{
			value: 'home', label: i18n.ts._timelines.home,
		}, {
			value: 'local', label: i18n.ts._timelines.local,
		}, {
			value: 'social', label: i18n.ts._timelines.social,
		}, {
			value: 'global', label: i18n.ts._timelines.global,
		}],
		default: props.column.tl,
	});
	if (canceled) {
		if (props.column.tl == null) {
			removeColumn(props.column.id);
		}
		return;
	}
	if (src == null) return;
	updateColumn(props.column.id, {
		tl: src ?? undefined,
	});
}
const menu = computed<MenuItem[]>(() => {
	const menuItems: MenuItem[] = [];

	menuItems.push({
		icon: 'ti ti-pencil',
		text: i18n.ts.timeline,
		action: setType,
	}, {
		icon: 'ti ti-bell',
		text: i18n.ts._deck.newNoteNotificationSettings,
		action: () => soundSettingsButton(soundSetting),
	}, {
		type: 'switch',
		text: i18n.ts.showRenotes,
		ref: withRenotes,
	});

	if (hasWithReplies(props.column.tl)) {
		menuItems.push({
			type: 'switch',
			text: i18n.ts.showRepliesToOthersInTimeline,
			ref: withReplies,
			disabled: onlyFiles,
		});
	}

	menuItems.push({
		type: 'switch',
		text: i18n.ts.fileAttachedOnly,
		ref: onlyFiles,
		disabled: hasWithReplies(props.column.tl) ? withReplies : false,
	}, {
		type: 'switch',
		text: i18n.ts.withSensitive,
		ref: withSensitive,
	});

	return menuItems;
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XColumn, {
      menu: menu.value,
      column: __props.column,
      isStacked: __props.isStacked,
      refresher: async () => { await _unref(timeline)?.reloadTimeline() }
    }, {
      header: _withCtx(() => [
        (__props.column.tl != null)
          ? (_openBlock(), _createElementBlock("i", {
            key: 0,
            class: _normalizeClass(_unref(basicTimelineIconClass)(__props.column.tl))
          }))
          : _createCommentVNode("v-if", true),
        _createElementVNode("span", _hoisted_1, _toDisplayString(__props.column.name || (__props.column.tl ? _unref(i18n).ts._timelines[__props.column.tl] : null) || _unref(i18n).ts._deck._columns.tl), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        (!_unref(isAvailableBasicTimeline)(__props.column.tl))
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.disabled)
          }, [
            _createElementVNode("p", {
              class: _normalizeClass(_ctx.$style.disabledTitle)
            }, [
              _hoisted_2,
              _createTextVNode("\n\t\t\t" + _toDisplayString(_unref(i18n).ts._disabledTimeline.title), 1 /* TEXT */)
            ]),
            _createElementVNode("p", {
              class: _normalizeClass(_ctx.$style.disabledDescription)
            }, _toDisplayString(_unref(i18n).ts._disabledTimeline.description), 1 /* TEXT */)
          ]))
          : (__props.column.tl)
            ? (_openBlock(), _createBlock(MkStreamingNotesTimeline, {
              key: __props.column.tl + withRenotes.value + withReplies.value + onlyFiles.value,
              ref: "timeline",
              src: __props.column.tl,
              withRenotes: withRenotes.value,
              withReplies: withReplies.value,
              withSensitive: withSensitive.value,
              onlyFiles: onlyFiles.value,
              sound: true,
              customSound: soundSetting.value
            }, null, 8 /* PROPS */, ["src", "withRenotes", "withReplies", "withSensitive", "onlyFiles", "sound", "customSound"]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["menu", "column", "isStacked", "refresher"]))
}
}

})
