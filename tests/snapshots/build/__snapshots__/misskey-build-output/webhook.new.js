import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-lock" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { ref, computed } from 'vue'
import * as Misskey from 'misskey-js'
import MkInput from '@/components/MkInput.vue'
import FormSection from '@/components/form/section.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'webhook.new',
  setup(__props) {

const name = ref('');
const url = ref('');
const secret = ref('');
const event_follow = ref(true);
const event_followed = ref(true);
const event_note = ref(true);
const event_reply = ref(true);
const event_renote = ref(true);
const event_reaction = ref(true);
const event_mention = ref(true);
async function create(): Promise<void> {
	const events = [] as Misskey.entities.UserWebhook['on'];
	if (event_follow.value) events.push('follow');
	if (event_followed.value) events.push('followed');
	if (event_note.value) events.push('note');
	if (event_reply.value) events.push('reply');
	if (event_renote.value) events.push('renote');
	if (event_reaction.value) events.push('reaction');
	if (event_mention.value) events.push('mention');
	os.apiWithDialog('i/webhooks/create', {
		name: name.value,
		url: url.value,
		secret: secret.value,
		on: events,
	});
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: 'Create new webhook',
	icon: 'ti ti-webhook',
}));

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(MkInput, {
        modelValue: name.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((name).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings.name), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkInput, {
        type: "url",
        modelValue: url.value,
        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((url).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode("URL")
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkInput, {
        modelValue: secret.value,
        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((secret).value = $event))
      }, {
        prefix: _withCtx(() => [
          _hoisted_1
        ]),
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings.secret), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(FormSection, null, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings.trigger), 1 /* TEXT */)
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(MkSwitch, {
              modelValue: event_follow.value,
              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((event_follow).value = $event))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings._events.follow), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]),
            _createVNode(MkSwitch, {
              modelValue: event_followed.value,
              "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((event_followed).value = $event))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings._events.followed), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]),
            _createVNode(MkSwitch, {
              modelValue: event_note.value,
              "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((event_note).value = $event))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings._events.note), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]),
            _createVNode(MkSwitch, {
              modelValue: event_reply.value,
              "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((event_reply).value = $event))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings._events.reply), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]),
            _createVNode(MkSwitch, {
              modelValue: event_renote.value,
              "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((event_renote).value = $event))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings._events.renote), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]),
            _createVNode(MkSwitch, {
              disabled: true,
              modelValue: event_reaction.value,
              "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((event_reaction).value = $event))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings._events.reaction), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["disabled", "modelValue"]),
            _createVNode(MkSwitch, {
              modelValue: event_mention.value,
              "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((event_mention).value = $event))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._webhookSettings._events.mention), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"])
          ])
        ]),
        _: 1 /* STABLE */
      }), _createElementVNode("div", { class: "_buttons" }, [ _createVNode(MkButton, {
          primary: "",
          inline: "",
          onClick: create
        }, {
          default: _withCtx(() => [
            _hoisted_2,
            _createTextVNode(" "),
            _createTextVNode(_toDisplayString(_unref(i18n).ts.create), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }) ]) ]))
}
}

})
