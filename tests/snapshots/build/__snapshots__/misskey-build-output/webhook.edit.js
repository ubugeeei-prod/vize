import { withAsyncContext as _withAsyncContext } from "vue";
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-lock" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_8 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_9 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
const _hoisted_10 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-trash" });
import { ref, computed } from "vue";
import MkInput from "@/components/MkInput.vue";
import FormSection from "@/components/form/section.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkButton from "@/components/MkButton.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import { useRouter } from "@/router.js";
export default {
  __name: "webhook.edit",
  props: { webhookId: {
    type: String,
    required: true
  } },
  async setup(__props) {
    let __temp, __restore;
    const props = __props;
    const router = useRouter();
    const webhook = ([__temp, __restore] = _withAsyncContext(() => misskeyApi("i/webhooks/show", { webhookId: props.webhookId })), __temp = await __temp, __restore(), __temp);
    const name = ref(webhook.name);
    const url = ref(webhook.url);
    const secret = ref(webhook.secret);
    const active = ref(webhook.active);
    const event_follow = ref(webhook.on.includes("follow"));
    const event_followed = ref(webhook.on.includes("followed"));
    const event_note = ref(webhook.on.includes("note"));
    const event_reply = ref(webhook.on.includes("reply"));
    const event_renote = ref(webhook.on.includes("renote"));
    const event_reaction = ref(webhook.on.includes("reaction"));
    const event_mention = ref(webhook.on.includes("mention"));
    function save() {
      const events = [];
      if (event_follow.value) events.push("follow");
      if (event_followed.value) events.push("followed");
      if (event_note.value) events.push("note");
      if (event_reply.value) events.push("reply");
      if (event_renote.value) events.push("renote");
      if (event_reaction.value) events.push("reaction");
      if (event_mention.value) events.push("mention");
      os.apiWithDialog("i/webhooks/update", {
        name: name.value,
        url: url.value,
        secret: secret.value,
        webhookId: props.webhookId,
        on: events,
        active: active.value
      });
    }
    async function del() {
      const { canceled } = await os.confirm({
        type: "warning",
        text: i18n.tsx.deleteAreYouSure({ x: webhook.name })
      });
      if (canceled) return;
      await os.apiWithDialog("i/webhooks/delete", { webhookId: props.webhookId });
      router.push("/settings/connect");
    }
    async function test(type) {
      await os.apiWithDialog("i/webhooks/test", {
        webhookId: props.webhookId,
        type,
        override: {
          secret: secret.value,
          url: url.value
        }
      });
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: "Edit webhook",
      icon: "ti ti-webhook"
    }));
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [
        _createVNode(MkInput, {
          modelValue: name.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => name.value = $event)
        }, {
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._webhookSettings.name),
            1
            /* TEXT */
          )]),
          _: 1
        }, 8, ["modelValue"]),
        _createVNode(MkInput, {
          type: "url",
          modelValue: url.value,
          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => url.value = $event)
        }, {
          label: _withCtx(() => [_createTextVNode("URL")]),
          _: 1
        }, 8, ["modelValue"]),
        _createVNode(MkInput, {
          modelValue: secret.value,
          "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => secret.value = $event)
        }, {
          prefix: _withCtx(() => [_hoisted_1]),
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._webhookSettings.secret),
            1
            /* TEXT */
          )]),
          _: 1
        }, 8, ["modelValue"]),
        _createVNode(FormSection, null, {
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._webhookSettings.trigger),
            1
            /* TEXT */
          )]),
          default: _withCtx(() => [_createElementVNode("div", { class: "_gaps" }, [_createElementVNode("div", { class: "_gaps_s" }, [
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.switchBox) },
              [_createVNode(MkSwitch, {
                modelValue: event_follow.value,
                "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => event_follow.value = $event)
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._webhookSettings._events.follow),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["modelValue"]), _createVNode(MkButton, {
                transparent: "",
                class: _normalizeClass(_ctx.$style.testButton),
                disabled: !(active.value && event_follow.value),
                onClick: _cache[4] || (_cache[4] = ($event) => test("follow"))
              }, {
                default: _withCtx(() => [_hoisted_2]),
                _: 1
              }, 10, ["disabled"])],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.switchBox) },
              [_createVNode(MkSwitch, {
                modelValue: event_followed.value,
                "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => event_followed.value = $event)
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._webhookSettings._events.followed),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["modelValue"]), _createVNode(MkButton, {
                transparent: "",
                class: _normalizeClass(_ctx.$style.testButton),
                disabled: !(active.value && event_followed.value),
                onClick: _cache[6] || (_cache[6] = ($event) => test("followed"))
              }, {
                default: _withCtx(() => [_hoisted_3]),
                _: 1
              }, 10, ["disabled"])],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.switchBox) },
              [_createVNode(MkSwitch, {
                modelValue: event_note.value,
                "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event) => event_note.value = $event)
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._webhookSettings._events.note),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["modelValue"]), _createVNode(MkButton, {
                transparent: "",
                class: _normalizeClass(_ctx.$style.testButton),
                disabled: !(active.value && event_note.value),
                onClick: _cache[8] || (_cache[8] = ($event) => test("note"))
              }, {
                default: _withCtx(() => [_hoisted_4]),
                _: 1
              }, 10, ["disabled"])],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.switchBox) },
              [_createVNode(MkSwitch, {
                modelValue: event_reply.value,
                "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event) => event_reply.value = $event)
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._webhookSettings._events.reply),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["modelValue"]), _createVNode(MkButton, {
                transparent: "",
                class: _normalizeClass(_ctx.$style.testButton),
                disabled: !(active.value && event_reply.value),
                onClick: _cache[10] || (_cache[10] = ($event) => test("reply"))
              }, {
                default: _withCtx(() => [_hoisted_5]),
                _: 1
              }, 10, ["disabled"])],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.switchBox) },
              [_createVNode(MkSwitch, {
                modelValue: event_renote.value,
                "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event) => event_renote.value = $event)
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._webhookSettings._events.renote),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["modelValue"]), _createVNode(MkButton, {
                transparent: "",
                class: _normalizeClass(_ctx.$style.testButton),
                disabled: !(active.value && event_renote.value),
                onClick: _cache[12] || (_cache[12] = ($event) => test("renote"))
              }, {
                default: _withCtx(() => [_hoisted_6]),
                _: 1
              }, 10, ["disabled"])],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.switchBox) },
              [_createVNode(MkSwitch, {
                disabled: true,
                modelValue: event_reaction.value,
                "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event) => event_reaction.value = $event)
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._webhookSettings._events.reaction),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["disabled", "modelValue"]), _createVNode(MkButton, {
                transparent: "",
                class: _normalizeClass(_ctx.$style.testButton),
                disabled: !(active.value && event_reaction.value),
                onClick: _cache[14] || (_cache[14] = ($event) => test("reaction"))
              }, {
                default: _withCtx(() => [_hoisted_7]),
                _: 1
              }, 10, ["disabled"])],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.switchBox) },
              [_createVNode(MkSwitch, {
                modelValue: event_mention.value,
                "onUpdate:modelValue": _cache[15] || (_cache[15] = ($event) => event_mention.value = $event)
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._webhookSettings._events.mention),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["modelValue"]), _createVNode(MkButton, {
                transparent: "",
                class: _normalizeClass(_ctx.$style.testButton),
                disabled: !(active.value && event_mention.value),
                onClick: _cache[16] || (_cache[16] = ($event) => test("mention"))
              }, {
                default: _withCtx(() => [_hoisted_8]),
                _: 1
              }, 10, ["disabled"])],
              2
              /* CLASS */
            )
          ]), _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.description) },
            _toDisplayString(_unref(i18n).ts._webhookSettings.testRemarks),
            3
            /* TEXT, CLASS */
          )])]),
          _: 1
        }),
        _createVNode(MkSwitch, {
          modelValue: active.value,
          "onUpdate:modelValue": _cache[17] || (_cache[17] = ($event) => active.value = $event)
        }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._webhookSettings.active),
            1
            /* TEXT */
          )]),
          _: 1
        }, 8, ["modelValue"]),
        _createElementVNode("div", { class: "_buttons" }, [_createVNode(MkButton, {
          primary: "",
          inline: "",
          onClick: save
        }, {
          default: _withCtx(() => [
            _hoisted_9,
            _createTextVNode(" "),
            _createTextVNode(
              _toDisplayString(_unref(i18n).ts.save),
              1
              /* TEXT */
            )
          ]),
          _: 1
        }), _createVNode(MkButton, {
          danger: "",
          inline: "",
          onClick: del
        }, {
          default: _withCtx(() => [
            _hoisted_10,
            _createTextVNode(" "),
            _createTextVNode(
              _toDisplayString(_unref(i18n).ts.delete),
              1
              /* TEXT */
            )
          ]),
          _: 1
        })])
      ]);
    };
  }
};
