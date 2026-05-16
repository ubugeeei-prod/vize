import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-x" });
import { computed, onMounted, ref, useTemplateRef, toRefs } from "vue";
import MkInput from "@/components/MkInput.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import { i18n } from "@/i18n.js";
import MkButton from "@/components/MkButton.vue";
import { misskeyApi } from "@/utility/misskey-api.js";
import MkModalWindow from "@/components/MkModalWindow.vue";
import MkFolder from "@/components/MkFolder.vue";
import * as os from "@/os.js";
export default {
  __name: "MkSystemWebhookEditor",
  emits: [
    "submitted",
    "canceled",
    "closed"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const dialogEl = useTemplateRef("dialogEl");
    const { mode, id, requiredEvents } = toRefs(props);
    const loading = ref(0);
    const title = ref("");
    const url = ref("");
    const secret = ref("");
    const events = ref({
      abuseReport: true,
      abuseReportResolved: true,
      userCreated: true,
      inactiveModeratorsWarning: true,
      inactiveModeratorsInvitationOnlyChanged: true
    });
    const isActive = ref(true);
    const disabledEvents = ref({
      abuseReport: false,
      abuseReportResolved: false,
      userCreated: false,
      inactiveModeratorsWarning: false,
      inactiveModeratorsInvitationOnlyChanged: false
    });
    const disableSubmitButton = computed(() => {
      if (!title.value) {
        return true;
      }
      if (!url.value) {
        return true;
      }
      if (!secret.value) {
        return true;
      }
      return false;
    });
    async function onSubmitClicked() {
      await loadingScope(async () => {
        const params = {
          isActive: isActive.value,
          name: title.value,
          url: url.value,
          secret: secret.value,
          on: Object.keys(events.value).filter((ev) => events.value[ev])
        };
        try {
          switch (mode.value) {
            case "create": {
              const result = await misskeyApi("admin/system-webhook/create", params);
              dialogEl.value?.close();
              emit("submitted", result);
              break;
            }
            case "edit": {
              // eslint-disable-next-line
              const result = await misskeyApi("admin/system-webhook/update", {
                id: id.value,
                ...params
              });
              dialogEl.value?.close();
              emit("submitted", result);
              break;
            }
          }
        } catch (ex) {
          const msg = ex.message ?? i18n.ts.internalServerErrorDescription;
          await os.alert({
            type: "error",
            title: i18n.ts.error,
            text: msg
          });
          dialogEl.value?.close();
          emit("canceled");
        }
      });
    }
    function onCancelClicked() {
      dialogEl.value?.close();
      emit("canceled");
    }
    async function loadingScope(fn) {
      loading.value++;
      try {
        return await fn();
      } finally {
        loading.value--;
      }
    }
    async function test(type) {
      if (!id.value) {
        return Promise.resolve();
      }
      await os.apiWithDialog("admin/system-webhook/test", {
        webhookId: id.value,
        type,
        override: {
          secret: secret.value,
          url: url.value
        }
      });
    }
    onMounted(async () => {
      await loadingScope(async () => {
        switch (mode.value) {
          case "edit": {
            if (!id.value) {
              throw new Error("id is required");
            }
            try {
              const res = await misskeyApi("admin/system-webhook/show", { id: id.value });
              title.value = res.name;
              url.value = res.url;
              secret.value = res.secret;
              isActive.value = res.isActive;
              for (const ev of Object.keys(events.value)) {
                events.value[ev] = res.on.includes(ev);
              }
            } catch (ex) {
              const msg = ex.message ?? i18n.ts.internalServerErrorDescription;
              await os.alert({
                type: "error",
                title: i18n.ts.error,
                text: msg
              });
              dialogEl.value?.close();
              emit("canceled");
            }
            break;
          }
        }
        for (const ev of requiredEvents.value ?? []) {
          disabledEvents.value[ev] = true;
        }
      });
    });
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      return _openBlock(), _createBlock(MkModalWindow, {
        ref_key: "dialogEl",
        ref: dialogEl,
        width: 450,
        height: 590,
        canClose: true,
        withOkButton: false,
        okButtonDisabled: false,
        onClick: onCancelClicked,
        onClose: onCancelClicked,
        onClosed: _cache[0] || (_cache[0] = ($event) => emit("closed"))
      }, {
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(mode) === "create" ? _unref(i18n).ts._webhookSettings.createWebhook : _unref(i18n).ts._webhookSettings.modifyWebhook),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createElementVNode("div", { style: "display: flex; flex-direction: column; min-height: 100%;" }, [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-min: 20px; --MI_SPACER-max: 28px; flex-grow: 1;"
        }, [loading.value !== 0 ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : (_openBlock(), _createElementBlock(
          "div",
          {
            key: 1,
            class: _normalizeClass(["_gaps_m", _ctx.$style.root])
          },
          [
            _createVNode(MkInput, {
              modelValue: title.value,
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => title.value = $event)
            }, {
              label: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._webhookSettings.name),
                1
                /* TEXT */
              )]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkInput, {
              modelValue: url.value,
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => url.value = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("URL")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkInput, {
              modelValue: secret.value,
              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => secret.value = $event)
            }, {
              label: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._webhookSettings.secret),
                1
                /* TEXT */
              )]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkFolder, { defaultOpen: true }, {
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
                    disabled: disabledEvents.value.abuseReport,
                    modelValue: events.value.abuseReport,
                    "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => events.value.abuseReport = $event)
                  }, {
                    label: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts._webhookSettings._systemEvents.abuseReport),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  }, 8, ["disabled", "modelValue"]), _withDirectives(_createVNode(MkButton, {
                    transparent: "",
                    class: _normalizeClass(_ctx.$style.testButton),
                    disabled: !(isActive.value && events.value.abuseReport),
                    onClick: _cache[5] || (_cache[5] = ($event) => test("abuseReport"))
                  }, {
                    default: _withCtx(() => [_hoisted_1]),
                    _: 1
                  }, 10, ["disabled"]), [[_vShow, _unref(mode) === "edit"]])],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.switchBox) },
                  [_createVNode(MkSwitch, {
                    disabled: disabledEvents.value.abuseReportResolved,
                    modelValue: events.value.abuseReportResolved,
                    "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event) => events.value.abuseReportResolved = $event)
                  }, {
                    label: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts._webhookSettings._systemEvents.abuseReportResolved),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  }, 8, ["disabled", "modelValue"]), _withDirectives(_createVNode(MkButton, {
                    transparent: "",
                    class: _normalizeClass(_ctx.$style.testButton),
                    disabled: !(isActive.value && events.value.abuseReportResolved),
                    onClick: _cache[7] || (_cache[7] = ($event) => test("abuseReportResolved"))
                  }, {
                    default: _withCtx(() => [_hoisted_2]),
                    _: 1
                  }, 10, ["disabled"]), [[_vShow, _unref(mode) === "edit"]])],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.switchBox) },
                  [_createVNode(MkSwitch, {
                    disabled: disabledEvents.value.userCreated,
                    modelValue: events.value.userCreated,
                    "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event) => events.value.userCreated = $event)
                  }, {
                    label: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts._webhookSettings._systemEvents.userCreated),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  }, 8, ["disabled", "modelValue"]), _withDirectives(_createVNode(MkButton, {
                    transparent: "",
                    class: _normalizeClass(_ctx.$style.testButton),
                    disabled: !(isActive.value && events.value.userCreated),
                    onClick: _cache[9] || (_cache[9] = ($event) => test("userCreated"))
                  }, {
                    default: _withCtx(() => [_hoisted_3]),
                    _: 1
                  }, 10, ["disabled"]), [[_vShow, _unref(mode) === "edit"]])],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.switchBox) },
                  [_createVNode(MkSwitch, {
                    disabled: disabledEvents.value.inactiveModeratorsWarning,
                    modelValue: events.value.inactiveModeratorsWarning,
                    "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event) => events.value.inactiveModeratorsWarning = $event)
                  }, {
                    label: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts._webhookSettings._systemEvents.inactiveModeratorsWarning),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  }, 8, ["disabled", "modelValue"]), _withDirectives(_createVNode(MkButton, {
                    transparent: "",
                    class: _normalizeClass(_ctx.$style.testButton),
                    disabled: !(isActive.value && events.value.inactiveModeratorsWarning),
                    onClick: _cache[11] || (_cache[11] = ($event) => test("inactiveModeratorsWarning"))
                  }, {
                    default: _withCtx(() => [_hoisted_4]),
                    _: 1
                  }, 10, ["disabled"]), [[_vShow, _unref(mode) === "edit"]])],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.switchBox) },
                  [_createVNode(MkSwitch, {
                    disabled: disabledEvents.value.inactiveModeratorsInvitationOnlyChanged,
                    modelValue: events.value.inactiveModeratorsInvitationOnlyChanged,
                    "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event) => events.value.inactiveModeratorsInvitationOnlyChanged = $event)
                  }, {
                    label: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts._webhookSettings._systemEvents.inactiveModeratorsInvitationOnlyChanged),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  }, 8, ["disabled", "modelValue"]), _withDirectives(_createVNode(MkButton, {
                    transparent: "",
                    class: _normalizeClass(_ctx.$style.testButton),
                    disabled: !(isActive.value && events.value.inactiveModeratorsInvitationOnlyChanged),
                    onClick: _cache[13] || (_cache[13] = ($event) => test("inactiveModeratorsInvitationOnlyChanged"))
                  }, {
                    default: _withCtx(() => [_hoisted_5]),
                    _: 1
                  }, 10, ["disabled"]), [[_vShow, _unref(mode) === "edit"]])],
                  2
                  /* CLASS */
                )
              ]), _withDirectives(_createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.description) },
                _toDisplayString(_unref(i18n).ts._webhookSettings.testRemarks),
                3
                /* TEXT, CLASS */
              ), [[_vShow, _unref(mode) === "edit"]])])]),
              _: 1
            }, 8, ["defaultOpen"]),
            _createVNode(MkSwitch, {
              modelValue: isActive.value,
              "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event) => isActive.value = $event)
            }, {
              label: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.enable),
                1
                /* TEXT */
              )]),
              _: 1
            }, 8, ["modelValue"])
          ],
          2
          /* CLASS */
        ))]), _createElementVNode(
          "div",
          { class: _normalizeClass(["_buttonsCenter", _ctx.$style.footer]) },
          [_createVNode(MkButton, {
            primary: "",
            rounded: "",
            disabled: disableSubmitButton.value,
            onClick: onSubmitClicked
          }, {
            default: _withCtx(() => [
              _hoisted_6,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.ok),
                1
                /* TEXT */
              )
            ]),
            _: 1
          }, 8, ["disabled"]), _createVNode(MkButton, {
            rounded: "",
            onClick: onCancelClicked
          }, {
            default: _withCtx(() => [
              _hoisted_7,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.cancel),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })],
          2
          /* CLASS */
        )])]),
        _: 1
      }, 8, [
        "width",
        "height",
        "canClose",
        "withOkButton",
        "okButtonDisabled"
      ]);
    };
  }
};
