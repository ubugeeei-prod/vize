import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-trash" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-menu" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-x" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-device-floppy" });
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-trash" });
import { computed, ref, watch } from "vue";
import MkButton from "@/components/MkButton.vue";
import MkInput from "@/components/MkInput.vue";
import MkColorInput from "@/components/MkColorInput.vue";
import { selectFile } from "@/utility/drive.js";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { definePage } from "@/page.js";
import { i18n } from "@/i18n.js";
import MkFolder from "@/components/MkFolder.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkTextarea from "@/components/MkTextarea.vue";
import MkDraggable from "@/components/MkDraggable.vue";
import { useRouter } from "@/router.js";
export default {
  __name: "channel-editor",
  props: { channelId: {
    type: String,
    required: false
  } },
  setup(__props) {
    const props = __props;
    const router = useRouter();
    const channel = ref(null);
    const name = ref("");
    const description = ref(null);
    const bannerUrl = ref(null);
    const bannerId = ref(null);
    const color = ref("#000");
    const isSensitive = ref(false);
    const allowRenoteToExternal = ref(true);
    const pinnedNoteIds = ref([]);
    watch(() => bannerId.value, async () => {
      if (bannerId.value == null) {
        bannerUrl.value = null;
      } else {
        bannerUrl.value = (await misskeyApi("drive/files/show", { fileId: bannerId.value })).url;
      }
    });
    async function fetchChannel() {
      if (props.channelId == null) return;
      const result = await misskeyApi("channels/show", { channelId: props.channelId });
      name.value = result.name;
      description.value = result.description;
      bannerId.value = result.bannerId;
      bannerUrl.value = result.bannerUrl;
      isSensitive.value = result.isSensitive;
      pinnedNoteIds.value = result.pinnedNoteIds;
      color.value = result.color;
      allowRenoteToExternal.value = result.allowRenoteToExternal;
      channel.value = result;
    }
    fetchChannel();
    async function addPinnedNote() {
      const { canceled, result: value } = await os.inputText({ title: i18n.ts.noteIdOrUrl });
      if (canceled || value == null) return;
      const fromUrl = value.includes("/") ? value.split("/").pop() : null;
      const note = await os.apiWithDialog("notes/show", { noteId: fromUrl ?? value });
      pinnedNoteIds.value.unshift(note.id);
    }
    function removePinnedNote(id) {
      pinnedNoteIds.value = pinnedNoteIds.value.filter((x) => x !== id);
    }
    function save() {
      const params = {
        name: name.value,
        description: description.value,
        bannerId: bannerId.value,
        color: color.value,
        isSensitive: isSensitive.value,
        allowRenoteToExternal: allowRenoteToExternal.value
      };
      if (props.channelId != null) {
        os.apiWithDialog("channels/update", {
          ...params,
          channelId: props.channelId,
          pinnedNoteIds: pinnedNoteIds.value
        });
      } else {
        os.apiWithDialog("channels/create", params).then((created) => {
          router.push("/channels/:channelId", { params: { channelId: created.id } });
        });
      }
    }
    async function archive() {
      if (props.channelId == null) return;
      const { canceled } = await os.confirm({
        type: "warning",
        title: i18n.tsx.channelArchiveConfirmTitle({ name: name.value }),
        text: i18n.ts.channelArchiveConfirmDescription
      });
      if (canceled) return;
      misskeyApi("channels/update", {
        channelId: props.channelId,
        isArchived: true
      }).then(() => {
        os.success();
      });
    }
    function setBannerImage(evt) {
      selectFile({
        anchorElement: evt.currentTarget ?? evt.target,
        multiple: false
      }).then((file) => {
        bannerId.value = file.id;
      });
    }
    function removeBannerImage() {
      bannerId.value = null;
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: props.channelId ? i18n.ts._channel.edit : i18n.ts._channel.create,
      icon: "ti ti-device-tv"
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
        }, [__props.channelId == null || channel.value != null ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps_m"
        }, [
          _createVNode(MkInput, {
            modelValue: name.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => name.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.name),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["modelValue"]),
          _createVNode(MkTextarea, {
            mfmAutocomplete: "",
            mfmPreview: true,
            modelValue: description.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => description.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.description),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["mfmPreview", "modelValue"]),
          _createVNode(MkColorInput, {
            modelValue: color.value,
            "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => color.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.color),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["modelValue"]),
          _createVNode(MkSwitch, {
            modelValue: isSensitive.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => isSensitive.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.sensitive),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["modelValue"]),
          _createVNode(MkSwitch, {
            modelValue: allowRenoteToExternal.value,
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => allowRenoteToExternal.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._channel.allowRenoteToExternal),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["modelValue"]),
          _createElementVNode("div", null, [bannerId.value == null ? (_openBlock(), _createBlock(MkButton, {
            key: 0,
            onClick: setBannerImage
          }, {
            default: _withCtx(() => [
              _hoisted_1,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts._channel.setBanner),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })) : bannerUrl.value ? (_openBlock(), _createElementBlock("div", { key: 1 }, [_createElementVNode("img", {
            src: bannerUrl.value,
            style: "width: 100%;"
          }, null, 8, ["src"]), _createVNode(MkButton, { onClick: _cache[5] || (_cache[5] = ($event) => removeBannerImage()) }, {
            default: _withCtx(() => [
              _hoisted_2,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts._channel.removeBanner),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })])) : _createCommentVNode("v-if", true)]),
          _createVNode(MkFolder, { defaultOpen: true }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.pinnedNotes),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps" }, [_createVNode(MkButton, {
              primary: "",
              rounded: "",
              onClick: _cache[6] || (_cache[6] = ($event) => addPinnedNote())
            }, {
              default: _withCtx(() => [_hoisted_3]),
              _: 1
            }), _createVNode(MkDraggable, {
              modelValue: pinnedNoteIds.value.map((id) => ({ id })),
              direction: "vertical",
              "onUpdate:modelValue": _cache[7] || (_cache[7] = (v) => pinnedNoteIds.value = v.map((x) => x.id))
            }, {
              default: _withCtx(({ item }) => [_createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.pinnedNote) },
                [
                  _createElementVNode(
                    "button",
                    { class: _normalizeClass(["_button", _ctx.$style.pinnedNoteHandle]) },
                    [_hoisted_4],
                    2
                    /* CLASS */
                  ),
                  _createTextVNode(
                    " " + _toDisplayString(item.id) + " ",
                    1
                    /* TEXT */
                  ),
                  _createElementVNode("button", {
                    class: _normalizeClass(["_button", _ctx.$style.pinnedNoteRemove]),
                    onClick: ($event) => removePinnedNote(item.id)
                  }, [_hoisted_5], 10, ["onClick"])
                ],
                2
                /* CLASS */
              )]),
              _: 1
            }, 8, ["modelValue"])])]),
            _: 1
          }, 8, ["defaultOpen"]),
          _createElementVNode("div", { class: "_buttons" }, [_createVNode(MkButton, {
            primary: "",
            onClick: _cache[8] || (_cache[8] = ($event) => save())
          }, {
            default: _withCtx(() => [
              _hoisted_6,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(__props.channelId ? _unref(i18n).ts.save : _unref(i18n).ts.create),
                1
                /* TEXT */
              )
            ]),
            _: 1
          }), __props.channelId ? (_openBlock(), _createBlock(MkButton, {
            key: 0,
            danger: "",
            onClick: _cache[9] || (_cache[9] = ($event) => archive())
          }, {
            default: _withCtx(() => [
              _hoisted_7,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.archive),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })) : _createCommentVNode("v-if", true)])
        ])) : _createCommentVNode("v-if", true)])]),
        _: 1
      }, 8, ["actions", "tabs"]);
    };
  }
};
