import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", null, "Process")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("div", null, "Active")
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("div", null, "Delayed")
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("div", null, "Waiting")
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("div", null, "Process")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("div", null, "Active")
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("div", null, "Delayed")
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("div", null, "Waiting")
import { onUnmounted, reactive, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import { useStream } from '@/stream.js'
import kmg from '@/filters/kmg.js'
import * as sound from '@/utility/sound.js'
import { deepClone } from '@/utility/clone.js'
import { prefer } from '@/preferences.js'
import { genId } from '@/utility/id.js'
import { i18n } from '@/i18n.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'jobQueue';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetJobQueue',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	transparent: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.transparent,
		default: false,
	},
	sound: {
		type: 'boolean',
		label: i18n.ts._widgetOptions._jobQueue.sound,
		default: false,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const connection = useStream().useChannel('queueStats');
const current = reactive({
	inbox: {
		activeSincePrevTick: 0,
		active: 0,
		waiting: 0,
		delayed: 0,
	},
	deliver: {
		activeSincePrevTick: 0,
		active: 0,
		waiting: 0,
		delayed: 0,
	},
});
const prev = reactive({} as typeof current);
const jammedAudioBuffer = ref<AudioBuffer | null>(null);
const jammedSoundNodePlaying = ref<boolean>(false);
if (prefer.s['sound.masterVolume']) {
	sound.loadAudio('/client-assets/sounds/syuilo/queue-jammed.mp3').then(buf => {
		if (!buf) throw new Error('[WidgetJobQueue] Failed to initialize AudioBuffer');
		jammedAudioBuffer.value = buf;
	});
}
for (const domain of ['inbox', 'deliver']) {
	const d = domain as 'inbox' | 'deliver';
	prev[d] = deepClone(current[d]);
}
const onStats = (stats: Misskey.entities.QueueStats) => {
	for (const domain of ['inbox', 'deliver']) {
		const d = domain as 'inbox' | 'deliver';
		prev[d] = deepClone(current[d]);
		current[d].activeSincePrevTick = stats[d].activeSincePrevTick;
		current[d].active = stats[d].active;
		current[d].waiting = stats[d].waiting;
		current[d].delayed = stats[d].delayed;

		if (current[d].waiting > 0 && widgetProps.sound && jammedAudioBuffer.value && !jammedSoundNodePlaying.value) {
			const soundNode = sound.createSourceNode(jammedAudioBuffer.value, {}).soundSource;
			if (soundNode != null) {
				jammedSoundNodePlaying.value = true;
				soundNode.onended = () => jammedSoundNodePlaying.value = false;
				soundNode.start();
			}
		}
	}
};
const onStatsLog = (statsLog: Misskey.entities.QueueStatsLog) => {
	for (const stats of [...statsLog].reverse()) {
		onStats(stats);
	}
};
connection.on('stats', onStats);
connection.on('statsLog', onStatsLog);
connection.send('requestLog', {
	id: genId(),
	length: 1,
});
onUnmounted(() => {
	connection.off('stats', onStats);
	connection.off('statsLog', onStatsLog);
	connection.dispose();
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      "data-cy-mkw-jobQueue": "",
      class: _normalizeClass(["mkw-jobQueue _monospace", { _panel: !_unref(widgetProps).transparent }])
    }, [ _createElementVNode("div", { class: "inbox" }, [ _createElementVNode("div", { class: "label" }, [ _createTextVNode("Inbox queue"), (current.inbox.waiting > 0) ? (_openBlock(), _createElementBlock("i", {
              key: 0,
              class: "ti ti-alert-triangle icon"
            })) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", { class: "values" }, [ _createElementVNode("div", null, [ _hoisted_1, _createElementVNode("div", {
              class: _normalizeClass({ inc: current.inbox.activeSincePrevTick > prev.inbox.activeSincePrevTick, dec: current.inbox.activeSincePrevTick < prev.inbox.activeSincePrevTick }),
              title: `${current.inbox.activeSincePrevTick}`
            }, _toDisplayString(kmg(current.inbox.activeSincePrevTick, 2)), 11 /* TEXT, CLASS, PROPS */, ["title"]) ]), _createElementVNode("div", null, [ _hoisted_2, _createElementVNode("div", {
              class: _normalizeClass({ inc: current.inbox.active > prev.inbox.active, dec: current.inbox.active < prev.inbox.active }),
              title: `${current.inbox.active}`
            }, _toDisplayString(kmg(current.inbox.active, 2)), 11 /* TEXT, CLASS, PROPS */, ["title"]) ]), _createElementVNode("div", null, [ _hoisted_3, _createElementVNode("div", {
              class: _normalizeClass({ inc: current.inbox.delayed > prev.inbox.delayed, dec: current.inbox.delayed < prev.inbox.delayed }),
              title: `${current.inbox.delayed}`
            }, _toDisplayString(kmg(current.inbox.delayed, 2)), 11 /* TEXT, CLASS, PROPS */, ["title"]) ]), _createElementVNode("div", null, [ _hoisted_4, _createElementVNode("div", {
              class: _normalizeClass({ inc: current.inbox.waiting > prev.inbox.waiting, dec: current.inbox.waiting < prev.inbox.waiting }),
              title: `${current.inbox.waiting}`
            }, _toDisplayString(kmg(current.inbox.waiting, 2)), 11 /* TEXT, CLASS, PROPS */, ["title"]) ]) ]) ]), _createElementVNode("div", { class: "deliver" }, [ _createElementVNode("div", { class: "label" }, [ _createTextVNode("Deliver queue"), (current.deliver.waiting > 0) ? (_openBlock(), _createElementBlock("i", {
              key: 0,
              class: "ti ti-alert-triangle icon"
            })) : _createCommentVNode("v-if", true) ]), _createElementVNode("div", { class: "values" }, [ _createElementVNode("div", null, [ _hoisted_5, _createElementVNode("div", {
              class: _normalizeClass({ inc: current.deliver.activeSincePrevTick > prev.deliver.activeSincePrevTick, dec: current.deliver.activeSincePrevTick < prev.deliver.activeSincePrevTick }),
              title: `${current.deliver.activeSincePrevTick}`
            }, _toDisplayString(kmg(current.deliver.activeSincePrevTick, 2)), 11 /* TEXT, CLASS, PROPS */, ["title"]) ]), _createElementVNode("div", null, [ _hoisted_6, _createElementVNode("div", {
              class: _normalizeClass({ inc: current.deliver.active > prev.deliver.active, dec: current.deliver.active < prev.deliver.active }),
              title: `${current.deliver.active}`
            }, _toDisplayString(kmg(current.deliver.active, 2)), 11 /* TEXT, CLASS, PROPS */, ["title"]) ]), _createElementVNode("div", null, [ _hoisted_7, _createElementVNode("div", {
              class: _normalizeClass({ inc: current.deliver.delayed > prev.deliver.delayed, dec: current.deliver.delayed < prev.deliver.delayed }),
              title: `${current.deliver.delayed}`
            }, _toDisplayString(kmg(current.deliver.delayed, 2)), 11 /* TEXT, CLASS, PROPS */, ["title"]) ]), _createElementVNode("div", null, [ _hoisted_8, _createElementVNode("div", {
              class: _normalizeClass({ inc: current.deliver.waiting > prev.deliver.waiting, dec: current.deliver.waiting < prev.deliver.waiting }),
              title: `${current.deliver.waiting}`
            }, _toDisplayString(kmg(current.deliver.waiting, 2)), 11 /* TEXT, CLASS, PROPS */, ["title"]) ]) ]) ]) ], 2 /* CLASS */))
}
}

})
