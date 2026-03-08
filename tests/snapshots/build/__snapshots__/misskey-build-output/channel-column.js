import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-tv" })
const _hoisted_2 = { style: "margin-left: 8px;" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
import { onMounted, ref, shallowRef, watch, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import XColumn from './column.vue'
import type { Column } from '@/deck.js'
import type { MenuItem } from '@/types/menu.js'
import type { SoundStore } from '@/preferences/def.js'
import { updateColumn } from '@/deck.js'
import MkStreamingNotesTimeline from '@/components/MkStreamingNotesTimeline.vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { favoritedChannelsCache } from '@/cache.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { soundSettingsButton } from '@/ui/deck/tl-note-notification.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'channel-column',
  props: {
    column: { type: null, required: true },
    isStacked: { type: Boolean, required: true }
  },
  setup(__props: any) {

const props = __props
const timeline = useTemplateRef('timeline');
const channel = shallowRef<Misskey.entities.Channel>();
const soundSetting = ref<SoundStore>(props.column.soundSetting ?? { type: null, volume: 1 });
onMounted(() => {
	if (props.column.channelId == null) {
		setChannel();
	} else if (!props.column.name && props.column.channelId) {
		misskeyApi('channels/show', { channelId: props.column.channelId })
			.then(value => updateColumn(props.column.id, { timelineNameCache: value.name }));
	}
});
watch(soundSetting, v => {
	updateColumn(props.column.id, { soundSetting: v });
});
async function setChannel() {
	const channels = await favoritedChannelsCache.fetch();
	const { canceled, result: chosenChannelId } = await os.select({
		title: i18n.ts.selectChannel,
		items: channels.map(x => ({
			value: x.id, label: x.name,
		})),
		default: channels.find(x => x.id === props.column.channelId)?.id,
	});
	if (canceled || chosenChannelId == null) return;
	const chosenChannel = channels.find(x => x.id === chosenChannelId)!;
	updateColumn(props.column.id, {
		channelId: chosenChannel.id,
		timelineNameCache: chosenChannel.name,
	});
}
async function post() {
	if (props.column.channelId == null) return;
	if (!channel.value || channel.value.id !== props.column.channelId) {
		channel.value = await misskeyApi('channels/show', {
			channelId: props.column.channelId,
		});
	}
	os.post({
		channel: channel.value,
	});
}
const menu: MenuItem[] = [{
	icon: 'ti ti-pencil',
	text: i18n.ts.selectChannel,
	action: setChannel,
}, {
	icon: 'ti ti-bell',
	text: i18n.ts._deck.newNoteNotificationSettings,
	action: () => soundSettingsButton(soundSetting),
}];

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(XColumn, {
      menu: menu,
      column: __props.column,
      isStacked: __props.isStacked,
      refresher: async () => { await _unref(timeline)?.reloadTimeline() }
    }, {
      header: _withCtx(() => [
        _hoisted_1,
        _createElementVNode("span", _hoisted_2, _toDisplayString(__props.column.name || __props.column.timelineNameCache || _unref(i18n).ts._deck._columns.channel), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        (__props.column.channelId)
          ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [
            _createElementVNode("div", { style: "padding: 8px; text-align: center;" }, [
              _createVNode(MkButton, {
                primary: "",
                gradate: "",
                rounded: "",
                inline: "",
                small: "",
                onClick: post
              }, {
                default: _withCtx(() => [
                  _hoisted_3
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _createVNode(MkStreamingNotesTimeline, {
              ref_key: "timeline", ref: timeline,
              src: "channel",
              channel: __props.column.channelId
            }, null, 8 /* PROPS */, ["channel"])
          ], 64 /* STABLE_FRAGMENT */))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["menu", "column", "isStacked", "refresher"]))
}
}

})
