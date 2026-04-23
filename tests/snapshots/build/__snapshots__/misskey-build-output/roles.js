import { withAsyncContext as _withAsyncContext } from "vue";
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-search" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", {
  class: "ti ti-alert-triangle",
  style: "color: var(--MI_THEME-warn);"
});
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", {
  class: "ti ti-alert-triangle",
  style: "color: var(--MI_THEME-warn);"
});
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
import { computed, reactive, ref } from "vue";
import MkInput from "@/components/MkInput.vue";
import MkFolder from "@/components/MkFolder.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkButton from "@/components/MkButton.vue";
import MkSelect from "@/components/MkSelect.vue";
import MkRange from "@/components/MkRange.vue";
import MkRolePreview from "@/components/MkRolePreview.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import { instance, fetchInstance } from "@/instance.js";
import MkFoldableSection from "@/components/MkFoldableSection.vue";
import { useRouter } from "@/router.js";
import { deepClone } from "@/utility/clone.js";
import MkTextarea from "@/components/MkTextarea.vue";
export default {
  __name: "roles",
  async setup(__props) {
    let __temp, __restore;
    const router = useRouter();
    const baseRoleQ = ref("");
    const roles = ([__temp, __restore] = _withAsyncContext(() => misskeyApi("admin/roles/list")), __temp = await __temp, __restore(), __temp);
    const policies = reactive(deepClone(instance.policies));
    const avatarDecorationLimit = computed({
      get: () => Math.min(16, Math.max(0, policies.avatarDecorationLimit)),
      set: (value) => {
        policies.avatarDecorationLimit = Math.min(Number(value), 16);
      }
    });
    function updateAvatarDecorationLimit(value) {
      avatarDecorationLimit.value = Number(value);
    }
    function matchQuery(keywords) {
      if (baseRoleQ.value.trim().length === 0) return true;
      return keywords.some((keyword) => keyword.toLowerCase().includes(baseRoleQ.value.toLowerCase()));
    }
    async function updateBaseRole() {
      await os.apiWithDialog("admin/roles/update-default-policies", { 
      //@ts-expect-error misskey-js側の型定義が不十分
policies });
      fetchInstance(true);
    }
    function create() {
      router.push("/admin/roles/new");
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.roles,
      icon: "ti ti-badges"
    }));
    return (_ctx, _cache) => {
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value
      }, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [_createElementVNode("div", { class: "_gaps" }, [
          _createVNode(MkFolder, null, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._role.baseRole),
              1
              /* TEXT */
            )]),
            footer: _withCtx(() => [_createVNode(MkButton, {
              primary: "",
              rounded: "",
              onClick: updateBaseRole
            }, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.save),
                1
                /* TEXT */
              )]),
              _: 1
            })]),
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [
              _createVNode(MkInput, {
                type: "search",
                modelValue: baseRoleQ.value,
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => baseRoleQ.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_1]),
                _: 1
              }, 8, ["modelValue"]),
              matchQuery([_unref(i18n).ts._role._options.rateLimitFactor, "rateLimitFactor"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.rateLimitFactor),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(Math.floor(policies.rateLimitFactor * 100)) + "%",
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkRange, {
                  modelValue: policies.rateLimitFactor * 100,
                  min: 30,
                  max: 300,
                  step: 10,
                  textConverter: (v) => `${v}%`,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = (v) => policies.rateLimitFactor = v / 100)
                }, {
                  caption: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._role._options.descriptionOfRateLimitFactor),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, [
                  "modelValue",
                  "min",
                  "max",
                  "step",
                  "textConverter"
                ])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.gtlAvailable, "gtlAvailable"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.gtlAvailable),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.gtlAvailable ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.gtlAvailable,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => policies.gtlAvailable = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.ltlAvailable, "ltlAvailable"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.ltlAvailable),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.ltlAvailable ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.ltlAvailable,
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => policies.ltlAvailable = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canPublicNote, "canPublicNote"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canPublicNote),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canPublicNote ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canPublicNote,
                  "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => policies.canPublicNote = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.chatAvailability, "chatAvailability"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.chatAvailability),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.chatAvailability === "available" ? _unref(i18n).ts.yes : policies.chatAvailability === "readonly" ? _unref(i18n).ts.readonly : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSelect, {
                  items: [
                    {
                      label: _unref(i18n).ts.enabled,
                      value: "available"
                    },
                    {
                      label: _unref(i18n).ts.readonly,
                      value: "readonly"
                    },
                    {
                      label: _unref(i18n).ts.disabled,
                      value: "unavailable"
                    }
                  ],
                  modelValue: policies.chatAvailability,
                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => policies.chatAvailability = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["items", "modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.mentionMax, "mentionLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.mentionMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.mentionLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.mentionLimit,
                  "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event) => policies.mentionLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canInvite, "canInvite"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canInvite),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canInvite ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canInvite,
                  "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event) => policies.canInvite = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.inviteLimit, "inviteLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.inviteLimit),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.inviteLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.inviteLimit,
                  "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event) => policies.inviteLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.inviteLimitCycle, "inviteLimitCycle"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.inviteLimitCycle),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.inviteLimitCycle + _unref(i18n).ts._time.minute),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.inviteLimitCycle,
                  "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event) => policies.inviteLimitCycle = $event)
                }, {
                  suffix: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._time.minute),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.inviteExpirationTime, "inviteExpirationTime"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.inviteExpirationTime),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.inviteExpirationTime + _unref(i18n).ts._time.minute),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.inviteExpirationTime,
                  "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event) => policies.inviteExpirationTime = $event)
                }, {
                  suffix: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._time.minute),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canManageAvatarDecorations, "canManageAvatarDecorations"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canManageAvatarDecorations),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canManageAvatarDecorations ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canManageAvatarDecorations,
                  "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event) => policies.canManageAvatarDecorations = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canManageCustomEmojis, "canManageCustomEmojis"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canManageCustomEmojis),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canManageCustomEmojis ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canManageCustomEmojis,
                  "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event) => policies.canManageCustomEmojis = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canSearchNotes, "canSearchNotes"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canSearchNotes),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canSearchNotes ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canSearchNotes,
                  "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event) => policies.canSearchNotes = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canSearchUsers, "canSearchUsers"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canSearchUsers),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canSearchUsers ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canSearchUsers,
                  "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event) => policies.canSearchUsers = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canUseTranslator, "canUseTranslator"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canUseTranslator),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canUseTranslator ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canUseTranslator,
                  "onUpdate:modelValue": _cache[15] || (_cache[15] = ($event) => policies.canUseTranslator = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.driveCapacity, "driveCapacityMb"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.driveCapacity),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.driveCapacityMb) + "MB",
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.driveCapacityMb,
                  "onUpdate:modelValue": _cache[16] || (_cache[16] = ($event) => policies.driveCapacityMb = $event)
                }, {
                  suffix: _withCtx(() => [_createTextVNode("MB")]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.maxFileSize, "maxFileSizeMb"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.maxFileSize),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.maxFileSizeMb) + "MB",
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.maxFileSizeMb,
                  "onUpdate:modelValue": _cache[17] || (_cache[17] = ($event) => policies.maxFileSizeMb = $event)
                }, {
                  suffix: _withCtx(() => [_createTextVNode("MB")]),
                  caption: _withCtx(() => [_createElementVNode("div", null, [_hoisted_2, _createTextVNode(
                    " " + _toDisplayString(_unref(i18n).ts._role._options.maxFileSize_caption),
                    1
                    /* TEXT */
                  )])]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.uploadableFileTypes, "uploadableFileTypes"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.uploadableFileTypes),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode("...")]),
                default: _withCtx(() => [_createVNode(MkTextarea, {
                  modelValue: policies.uploadableFileTypes.join("\n"),
                  "onUpdate:modelValue": _cache[18] || (_cache[18] = (v) => policies.uploadableFileTypes = v.split("\n"))
                }, {
                  caption: _withCtx(() => [_createElementVNode(
                    "div",
                    null,
                    _toDisplayString(_unref(i18n).ts._role._options.uploadableFileTypes_caption),
                    1
                    /* TEXT */
                  ), _createElementVNode("div", null, [_hoisted_3, _createTextVNode(
                    " " + _toDisplayString(_unref(i18n).tsx._role._options.uploadableFileTypes_caption2({ x: "application/octet-stream" })),
                    1
                    /* TEXT */
                  )])]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.alwaysMarkNsfw, "alwaysMarkNsfw"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.alwaysMarkNsfw),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.alwaysMarkNsfw ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.alwaysMarkNsfw,
                  "onUpdate:modelValue": _cache[19] || (_cache[19] = ($event) => policies.alwaysMarkNsfw = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canUpdateBioMedia, "canUpdateBioMedia"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canUpdateBioMedia),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canUpdateBioMedia ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canUpdateBioMedia,
                  "onUpdate:modelValue": _cache[20] || (_cache[20] = ($event) => policies.canUpdateBioMedia = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.pinMax, "pinLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.pinMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.pinLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.pinLimit,
                  "onUpdate:modelValue": _cache[21] || (_cache[21] = ($event) => policies.pinLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.antennaMax, "antennaLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.antennaMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.antennaLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.antennaLimit,
                  "onUpdate:modelValue": _cache[22] || (_cache[22] = ($event) => policies.antennaLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.wordMuteMax, "wordMuteLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.wordMuteMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.wordMuteLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.wordMuteLimit,
                  "onUpdate:modelValue": _cache[23] || (_cache[23] = ($event) => policies.wordMuteLimit = $event)
                }, {
                  suffix: _withCtx(() => [_createTextVNode("chars")]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.webhookMax, "webhookLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.webhookMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.webhookLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.webhookLimit,
                  "onUpdate:modelValue": _cache[24] || (_cache[24] = ($event) => policies.webhookLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.clipMax, "clipLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.clipMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.clipLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.clipLimit,
                  "onUpdate:modelValue": _cache[25] || (_cache[25] = ($event) => policies.clipLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.noteEachClipsMax, "noteEachClipsLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.noteEachClipsMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.noteEachClipsLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.noteEachClipsLimit,
                  "onUpdate:modelValue": _cache[26] || (_cache[26] = ($event) => policies.noteEachClipsLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.userListMax, "userListLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.userListMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.userListLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.userListLimit,
                  "onUpdate:modelValue": _cache[27] || (_cache[27] = ($event) => policies.userListLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.userEachUserListsMax, "userEachUserListsLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.userEachUserListsMax),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.userEachUserListsLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  modelValue: policies.userEachUserListsLimit,
                  "onUpdate:modelValue": _cache[28] || (_cache[28] = ($event) => policies.userEachUserListsLimit = $event)
                }, null, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canHideAds, "canHideAds"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canHideAds),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canHideAds ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canHideAds,
                  "onUpdate:modelValue": _cache[29] || (_cache[29] = ($event) => policies.canHideAds = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.avatarDecorationLimit, "avatarDecorationLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.avatarDecorationLimit),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.avatarDecorationLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  min: 0,
                  max: 16,
                  "onUpdate:modelValue": [updateAvatarDecorationLimit, ($event) => avatarDecorationLimit.value = $event],
                  modelValue: avatarDecorationLimit.value
                }, null, 8, [
                  "min",
                  "max",
                  "modelValue"
                ])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canImportAntennas, "canImportAntennas"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canImportAntennas),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canImportAntennas ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canImportAntennas,
                  "onUpdate:modelValue": _cache[30] || (_cache[30] = ($event) => policies.canImportAntennas = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canImportBlocking, "canImportBlocking"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canImportBlocking),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canImportBlocking ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canImportBlocking,
                  "onUpdate:modelValue": _cache[31] || (_cache[31] = ($event) => policies.canImportBlocking = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canImportFollowing, "canImportFollowing"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canImportFollowing),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canImportFollowing ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canImportFollowing,
                  "onUpdate:modelValue": _cache[32] || (_cache[32] = ($event) => policies.canImportFollowing = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canImportMuting, "canImportMuting"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canImportMuting),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canImportMuting ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canImportMuting,
                  "onUpdate:modelValue": _cache[33] || (_cache[33] = ($event) => policies.canImportMuting = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.canImportUserLists, "canImportUserList"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.canImportUserLists),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.canImportUserLists ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.canImportUserLists,
                  "onUpdate:modelValue": _cache[34] || (_cache[34] = ($event) => policies.canImportUserLists = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.noteDraftLimit, "noteDraftLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.noteDraftLimit),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.noteDraftLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  min: 0,
                  modelValue: policies.noteDraftLimit,
                  "onUpdate:modelValue": _cache[35] || (_cache[35] = ($event) => policies.noteDraftLimit = $event)
                }, null, 8, ["min", "modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.scheduledNoteLimit, "scheduledNoteLimit"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.scheduledNoteLimit),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.scheduledNoteLimit),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkInput, {
                  type: "number",
                  min: 0,
                  modelValue: policies.scheduledNoteLimit,
                  "onUpdate:modelValue": _cache[36] || (_cache[36] = ($event) => policies.scheduledNoteLimit = $event)
                }, null, 8, ["min", "modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              matchQuery([_unref(i18n).ts._role._options.watermarkAvailable, "watermarkAvailable"]) ? (_openBlock(), _createBlock(MkFolder, { key: 0 }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._role._options.watermarkAvailable),
                  1
                  /* TEXT */
                )]),
                suffix: _withCtx(() => [_createTextVNode(
                  _toDisplayString(policies.watermarkAvailable ? _unref(i18n).ts.yes : _unref(i18n).ts.no),
                  1
                  /* TEXT */
                )]),
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: policies.watermarkAvailable,
                  "onUpdate:modelValue": _cache[37] || (_cache[37] = ($event) => policies.watermarkAvailable = $event)
                }, {
                  label: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.enable),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })) : _createCommentVNode("v-if", true)
            ])]),
            _: 1
          }),
          _createVNode(MkButton, {
            primary: "",
            rounded: "",
            onClick: create
          }, {
            default: _withCtx(() => [
              _hoisted_4,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts._role.new),
                1
                /* TEXT */
              )
            ]),
            _: 1
          }),
          _createElementVNode("div", { class: "_gaps_s" }, [_createVNode(MkFoldableSection, null, {
            header: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._role.manualRoles),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(_unref(roles).filter((x) => x.target === "manual"), (role) => {
                return _openBlock(), _createBlock(MkRolePreview, {
                  key: role.id,
                  role,
                  forModeration: true
                }, null, 8, ["role", "forModeration"]);
              }),
              128
              /* KEYED_FRAGMENT */
            ))])]),
            _: 1
          }), _createVNode(MkFoldableSection, null, {
            header: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._role.conditionalRoles),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(_unref(roles).filter((x) => x.target === "conditional"), (role) => {
                return _openBlock(), _createBlock(MkRolePreview, {
                  key: role.id,
                  role,
                  forModeration: true
                }, null, 8, ["role", "forModeration"]);
              }),
              128
              /* KEYED_FRAGMENT */
            ))])]),
            _: 1
          })])
        ])])]),
        _: 1
      }, 8, ["actions", "tabs"]);
    };
  }
};
