import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-list" })
const _hoisted_2 = { style: "margin-left: 8px;" }
import { watch, useTemplateRef, ref, onMounted } from 'vue'
import XColumn from './column.vue'
import type { entities as MisskeyEntities } from 'misskey-js'
import type { Column } from '@/deck.js'
import type { MenuItem } from '@/types/menu.js'
import type { SoundStore } from '@/preferences/def.js'
import { updateColumn } from '@/deck.js'
import MkStreamingNotesTimeline from '@/components/MkStreamingNotesTimeline.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { userListsCache } from '@/cache.js'
import { soundSettingsButton } from '@/ui/deck/tl-note-notification.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'list-column',
  props: {
    column: { type: null, required: true },
    isStacked: { type: Boolean, required: true }
  },
  setup(__props: any) {

const props = __props
const timeline = useTemplateRef('timeline');
const withRenotes = ref(props.column.withRenotes ?? true);
const soundSetting = ref<SoundStore>(props.column.soundSetting ?? { type: null, volume: 1 });
onMounted(() => {
	if (props.column.listId == null) {
		setList();
	} else if (props.column.timelineNameCache == null) {
		misskeyApi('users/lists/show', { listId: props.column.listId })
			.then(value => updateColumn(props.column.id, { timelineNameCache: value.name }));
	}
});
watch(withRenotes, v => {
	updateColumn(props.column.id, {
		withRenotes: v,
	});
});
watch(soundSetting, v => {
	updateColumn(props.column.id, { soundSetting: v });
});
async function setList() {
	const lists = await misskeyApi('users/lists/list');
	const { canceled, result: listIdOrOperation } = await os.select({
		title: i18n.ts.selectList,
		items: [
			{ value: '_CREATE_', label: i18n.ts.createNew },
			(lists.length > 0 ? {
				type: 'group' as const,
				label: i18n.ts.createdLists,
				items: lists.map(x => ({
					value: x.id, label: x.name,
				})),
			} : undefined),
		],
		default: lists.find(x => x.id === props.column.listId)?.id,
	});
	if (canceled || listIdOrOperation == null) return;
	if (listIdOrOperation === '_CREATE_') {
		const { canceled, result: name } = await os.inputText({
			title: i18n.ts.enterListName,
		});
		if (canceled || name == null || name === '') return;
		const res = await os.apiWithDialog('users/lists/create', { name: name });
		userListsCache.delete();
		updateColumn(props.column.id, {
			listId: res.id,
			timelineNameCache: res.name,
		});
	} else {
		const list = lists.find(x => x.id === listIdOrOperation)!;
		updateColumn(props.column.id, {
			listId: list.id,
			timelineNameCache: list.name,
		});
	}
}
function editList() {
	os.pageWindow('my/lists/' + props.column.listId);
}
const menu: MenuItem[] = [
	{
		icon: 'ti ti-pencil',
		text: i18n.ts.selectList,
		action: setList,
	},
	{
		icon: 'ti ti-settings',
		text: i18n.ts.editList,
		action: editList,
	},
	{
		type: 'switch',
		text: i18n.ts.showRenotes,
		ref: withRenotes,
	},
	{
		icon: 'ti ti-bell',
		text: i18n.ts._deck.newNoteNotificationSettings,
		action: () => soundSettingsButton(soundSetting),
	},
];

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XColumn, {
      menu: menu,
      column: __props.column,
      isStacked: __props.isStacked,
      refresher: async () => { await _unref(timeline)?.reloadTimeline() }
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createElementVNode("span", _hoisted_2, _toDisplayString(__props.column.name || __props.column.timelineNameCache || _unref(i18n).ts._deck._columns.list), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        (__props.column.listId)
          ? (_openBlock(), _createBlock(MkStreamingNotesTimeline, {
            key: 0,
            ref: "timeline",
            src: "list",
            list: __props.column.listId,
            withRenotes: withRenotes.value
          }, null, 8 /* PROPS */, ["list", "withRenotes"]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["menu", "column", "isStacked", "refresher"]))
}
}

})
