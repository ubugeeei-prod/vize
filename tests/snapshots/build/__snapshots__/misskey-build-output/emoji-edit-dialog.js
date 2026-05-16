import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("br");
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-x" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-ban" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-trash" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { computed, watch, ref, useTemplateRef } from "vue";
import MkWindow from "@/components/MkWindow.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { selectFile } from "@/utility/drive.js";
export default {
  __name: "emoji-edit-dialog",
  props: { emoji: {
    type: null,
    required: false
  } },
  emits: ["done", "closed"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const windowEl = useTemplateRef("windowEl");
    const name = ref(props.emoji ? props.emoji.name : "");
    const category = ref(props.emoji?.category ? props.emoji.category : "");
    const aliases = ref(props.emoji ? props.emoji.aliases.join(" ") : "");
    const license = ref(props.emoji?.license ? props.emoji.license : "");
    const isSensitive = ref(props.emoji ? props.emoji.isSensitive : false);
    const localOnly = ref(props.emoji ? props.emoji.localOnly : false);
    const roleIdsThatCanBeUsedThisEmojiAsReaction = ref(props.emoji ? props.emoji.roleIdsThatCanBeUsedThisEmojiAsReaction : []);
    const rolesThatCanBeUsedThisEmojiAsReaction = ref([]);
    const file = ref();
    watch(roleIdsThatCanBeUsedThisEmojiAsReaction, async () => {
      rolesThatCanBeUsedThisEmojiAsReaction.value = (await Promise.all(roleIdsThatCanBeUsedThisEmojiAsReaction.value.map((id) => misskeyApi("admin/roles/show", { roleId: id }).catch(() => null)))).filter((x) => x != null);
    }, { immediate: true });
    const imgUrl = computed(() => file.value ? file.value.url : props.emoji ? props.emoji.url : null);
    async function changeImage(ev) {
      file.value = await selectFile({
        anchorElement: ev.currentTarget ?? ev.target,
        multiple: false
      });
      const candidate = file.value.name.replace(/\.(.+)$/, "");
      if (candidate.match(/^[a-z0-9_]+$/)) {
        name.value = candidate;
      }
    }
    async function addRole() {
      const roles = await misskeyApi("admin/roles/list");
      const currentRoleIds = rolesThatCanBeUsedThisEmojiAsReaction.value.map((x) => x.id);
      const { canceled, result: roleId } = await os.select({ items: roles.filter((r) => r.isPublic).filter((r) => !currentRoleIds.includes(r.id)).map((r) => ({
        label: r.name,
        value: r.id
      })) });
      if (canceled || roleId == null) return;
      rolesThatCanBeUsedThisEmojiAsReaction.value.push(roles.find((r) => r.id === roleId));
    }
    async function removeRole(role) {
      rolesThatCanBeUsedThisEmojiAsReaction.value = rolesThatCanBeUsedThisEmojiAsReaction.value.filter((x) => x.id !== role.id);
    }
    async function done() {
      const params = {
        name: name.value,
        category: category.value === "" ? null : category.value,
        aliases: aliases.value.split(" ").filter((x) => x !== ""),
        license: license.value === "" ? null : license.value,
        isSensitive: isSensitive.value,
        localOnly: localOnly.value,
        roleIdsThatCanBeUsedThisEmojiAsReaction: rolesThatCanBeUsedThisEmojiAsReaction.value.map((x) => x.id),
        fileId: file.value ? file.value.id : undefined
      };
      if (props.emoji) {
        const emojiDetailed = {
          id: props.emoji.id,
          aliases: params.aliases,
          name: params.name,
          category: params.category,
          host: props.emoji.host,
          url: file.value ? file.value.url : props.emoji.url,
          license: params.license,
          isSensitive: params.isSensitive,
          localOnly: params.localOnly,
          roleIdsThatCanBeUsedThisEmojiAsReaction: params.roleIdsThatCanBeUsedThisEmojiAsReaction
        };
        await os.apiWithDialog("admin/emoji/update", {
          id: props.emoji.id,
          ...params
        });
        emit("done", { updated: emojiDetailed });
        windowEl.value?.close();
      } else {
        if (params.fileId == null) return;
        const created = await os.apiWithDialog("admin/emoji/add", {
          ...params,
          fileId: params.fileId
        });
        emit("done", { created });
        windowEl.value?.close();
      }
    }
    async function del() {
      if (!props.emoji) return;
      const { canceled } = await os.confirm({
        type: "warning",
        text: i18n.tsx.removeAreYouSure({ x: name.value })
      });
      if (canceled) return;
      misskeyApi("admin/emoji/delete", { id: props.emoji.id }).then(() => {
        emit("done", { deleted: true });
        windowEl.value?.close();
      });
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkWindow, {
        ref_key: "windowEl",
        ref: windowEl,
        initialWidth: 400,
        initialHeight: 500,
        canResize: true,
        onClose: _cache[0] || (_cache[0] = ($event) => _unref(windowEl)?.close()),
        onClosed: _cache[1] || (_cache[1] = ($event) => emit("closed"))
      }, _createSlots({ _: 2 }, [__props.emoji ? {
        name: "header",
        fn: _withCtx(() => [_createTextVNode(
          ":" + _toDisplayString(__props.emoji.name) + ":",
          1
          /* TEXT */
        )]),
        key: "0"
      } : {
        name: "header",
        fn: _withCtx(() => [_createTextVNode("New emoji")]),
        key: "1"
      }]), 1032, [
        "initialWidth",
        "initialHeight",
        "canResize"
      ]);
    };
  }
};
