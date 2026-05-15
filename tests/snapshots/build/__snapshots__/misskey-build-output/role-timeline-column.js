import { openBlock as _openBlock, createBlock as _createBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-badge" });
const _hoisted_2 = { style: "margin-left: 8px;" };
import { onMounted, ref, useTemplateRef, watch } from "vue";
import XColumn from "./column.vue";
import { updateColumn } from "@/deck.js";
import MkStreamingNotesTimeline from "@/components/MkStreamingNotesTimeline.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { soundSettingsButton } from "@/ui/deck/tl-note-notification.js";
export default {
  __name: "role-timeline-column",
  props: {
    column: {
      type: null,
      required: true
    },
    isStacked: {
      type: Boolean,
      required: true
    }
  },
  setup(__props) {
    const props = __props;
    const timeline = useTemplateRef("timeline");
    const soundSetting = ref(props.column.soundSetting ?? {
      type: null,
      volume: 1
    });
    onMounted(() => {
      if (props.column.roleId == null) {
        setRole();
      } else if (props.column.timelineNameCache == null) {
        misskeyApi("roles/show", { roleId: props.column.roleId }).then((value) => updateColumn(props.column.id, { timelineNameCache: value.name }));
      }
    });
    watch(soundSetting, (v) => {
      updateColumn(props.column.id, { soundSetting: v });
    });
    async function setRole() {
      const roles = (await misskeyApi("roles/list")).filter((x) => x.isExplorable);
      const { canceled, result: roleId } = await os.select({
        title: i18n.ts.role,
        items: roles.map((x) => ({
          value: x.id,
          label: x.name
        })),
        default: roles.find((x) => x.id === props.column.roleId)?.id
      });
      if (canceled || roleId == null) return;
      const role = roles.find((x) => x.id === roleId);
      updateColumn(props.column.id, {
        roleId: role.id,
        timelineNameCache: role.name
      });
    }
    const menu = [{
      icon: "ti ti-pencil",
      text: i18n.ts.role,
      action: setRole
    }, {
      icon: "ti ti-bell",
      text: i18n.ts._deck.newNoteNotificationSettings,
      action: () => soundSettingsButton(soundSetting)
    }];
    /*
    function focus() {
    timeline.focus();
    }
    defineExpose({
    focus,
    });
    */
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(XColumn, {
        menu,
        column: __props.column,
        isStacked: __props.isStacked,
        refresher: async () => {
          await _unref(timeline)?.reloadTimeline();
        }
      }, {
        header: _withCtx(() => [_hoisted_1, _createElementVNode(
          "span",
          _hoisted_2,
          _toDisplayString(__props.column.name || __props.column.timelineNameCache || _unref(i18n).ts._deck._columns.roleTimeline),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [__props.column.roleId ? (_openBlock(), _createBlock(MkStreamingNotesTimeline, {
          key: 0,
          ref_key: "timeline",
          ref: timeline,
          src: "role",
          role: __props.column.roleId
        }, null, 8, ["role"])) : _createCommentVNode("v-if", true)]),
        _: 2
      }, 1032, [
        "menu",
        "column",
        "isStacked",
        "refresher"
      ]);
    };
  }
};
