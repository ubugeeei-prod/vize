import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-antenna" })
const _hoisted_2 = { style: "margin-left: 8px;" }
import { onMounted, ref, useTemplateRef, watch, defineAsyncComponent } from 'vue'
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
import { antennasCache } from '@/cache.js'
import { soundSettingsButton } from '@/ui/deck/tl-note-notification.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'antenna-column',
  props: {
    column: { type: null, required: true },
    isStacked: { type: Boolean, required: true }
  },
  setup(__props: any) {

const props = __props
const timeline = useTemplateRef('timeline');
const soundSetting = ref<SoundStore>(props.column.soundSetting ?? { type: null, volume: 1 });
onMounted(() => {
	if (props.column.antennaId == null) {
		setAntenna();
	} else if (props.column.timelineNameCache == null) {
		misskeyApi('antennas/show', { antennaId: props.column.antennaId })
			.then(value => updateColumn(props.column.id, { timelineNameCache: value.name }));
	}
});
watch(soundSetting, v => {
	updateColumn(props.column.id, { soundSetting: v });
});
async function setAntenna() {
	const antennas = await misskeyApi('antennas/list');
	const { canceled, result: antennaIdOrOperation } = await os.select({
		title: i18n.ts.selectAntenna,
		items: [
			{ value: '_CREATE_', label: i18n.ts.createNew },
			(antennas.length > 0 ? {
				type: 'group' as const,
				label: i18n.ts.createdAntennas,
				items: antennas.map(x => ({
					value: x.id, label: x.name,
				})),
			} : undefined),
		],
		default: antennas.find(x => x.id === props.column.antennaId)?.id,
	});
	if (canceled || antennaIdOrOperation == null) return;
	if (antennaIdOrOperation === '_CREATE_') {
		const { dispose } = await os.popupAsyncWithDialog(import('@/components/MkAntennaEditorDialog.vue').then(x => x.default), {}, {
			created: (newAntenna: MisskeyEntities.Antenna) => {
				antennasCache.delete();
				updateColumn(props.column.id, {
					antennaId: newAntenna.id,
					timelineNameCache: newAntenna.name,
				});
			},
			closed: () => {
				dispose();
			},
		});
		return;
	}
	const antenna = antennas.find(x => x.id === antennaIdOrOperation)!;
	updateColumn(props.column.id, {
		antennaId: antenna.id,
		timelineNameCache: antenna.name,
	});
}
function editAntenna() {
	os.pageWindow('my/antennas/' + props.column.antennaId);
}
const menu: MenuItem[] = [
	{
		icon: 'ti ti-pencil',
		text: i18n.ts.selectAntenna,
		action: setAntenna,
	},
	{
		icon: 'ti ti-settings',
		text: i18n.ts.editAntenna,
		action: editAntenna,
	},
	{
		icon: 'ti ti-bell',
		text: i18n.ts._deck.newNoteNotificationSettings,
		action: () => soundSettingsButton(soundSetting),
	},
];
/*
function focus() {
	timeline.focus();
}
*/

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XColumn, {
      menu: menu,
      column: __props.column,
      isStacked: __props.isStacked,
      refresher: async () => { await _unref(timeline)?.reloadTimeline() }
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createElementVNode("span", _hoisted_2, _toDisplayString(__props.column.name || __props.column.timelineNameCache || _unref(i18n).ts._deck._columns.antenna), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        (__props.column.antennaId)
          ? (_openBlock(), _createBlock(MkStreamingNotesTimeline, {
            key: 0,
            ref: "timeline",
            src: "antenna",
            antenna: __props.column.antennaId
          }, null, 8 /* PROPS */, ["antenna"]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["menu", "column", "isStacked", "refresher"]))
}
}

})
