import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { computed } from "vue";
import MkSelect from "@/components/MkSelect.vue";
import MkButton from "@/components/MkButton.vue";
import { useMkSelect } from "@/composables/use-mkselect.js";
import { i18n } from "@/i18n.js";
const notificationConfigTypes = [
  "all",
  "following",
  "follower",
  "mutualFollow",
  "followingOrFollower",
  "list",
  "never"
];
export {};
export default {
  __name: "notifications.notification-config",
  props: {
    value: {
      type: Object,
      required: true
    },
    userLists: {
      type: Array,
      required: true
    },
    configurableTypes: {
      type: Array,
      required: false
    }
  },
  emits: ["update"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const notificationConfigTypesI18nMap = {
      all: i18n.ts.all,
      following: i18n.ts.following,
      follower: i18n.ts.followers,
      mutualFollow: i18n.ts.mutualFollow,
      followingOrFollower: i18n.ts.followingOrFollower,
      list: i18n.ts.userList,
      never: i18n.ts.none
    };
    const { model: type, def: typeDef } = useMkSelect({
      items: computed(() => (props.configurableTypes ?? notificationConfigTypes).map((t) => ({
        label: notificationConfigTypesI18nMap[t],
        value: t
      }))),
      initialValue: props.value.type
    });
    const { model: userListId, def: userListIdDef } = useMkSelect({
      items: computed(() => props.userLists.map((list) => ({
        label: list.name,
        value: list.id
      }))),
      initialValue: props.value.type === "list" ? props.value.userListId : null
    });
    function save() {
      emit("update", type.value === "list" ? {
        type: type.value,
        userListId: userListId.value
      } : { type: type.value });
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [
        _createVNode(MkSelect, {
          items: _unref(typeDef),
          modelValue: _unref(type),
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => type.value = $event)
        }, null, 8, ["items", "modelValue"]),
        _unref(type) === "list" ? (_openBlock(), _createBlock(MkSelect, {
          key: 0,
          items: _unref(userListIdDef),
          modelValue: _unref(userListId),
          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => userListId.value = $event)
        }, {
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.userList),
            1
            /* TEXT */
          )]),
          _: 1
        }, 8, ["items", "modelValue"])) : _createCommentVNode("v-if", true),
        _createElementVNode("div", { class: "_buttons" }, [_createVNode(MkButton, {
          inline: "",
          primary: "",
          disabled: _unref(type) === "list" && _unref(userListId) === null,
          onClick: save
        }, {
          default: _withCtx(() => [
            _hoisted_1,
            _createTextVNode(" "),
            _createTextVNode(
              _toDisplayString(_unref(i18n).ts.save),
              1
              /* TEXT */
            )
          ]),
          _: 1
        }, 8, ["disabled"])])
      ]);
    };
  }
};
