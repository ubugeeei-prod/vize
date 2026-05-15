import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { ref } from "vue";
import MkButton from "@/components/MkButton.vue";
import { i18n } from "@/i18n.js";
import { misskeyApi } from "@/utility/misskey-api.js";
export default {
  __name: "MkUserSetupDialog.User",
  props: { user: {
    type: null,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const isFollowing = ref(false);
    async function follow() {
      isFollowing.value = true;
      misskeyApi("following/create", { userId: props.user.id });
    }
    return (_ctx, _cache) => {
      const _component_MkAvatar = _resolveComponent("MkAvatar");
      const _component_MkUserName = _resolveComponent("MkUserName");
      const _component_MkAcct = _resolveComponent("MkAcct");
      const _component_Mfm = _resolveComponent("Mfm");
      const _directive_adaptive_bg = _resolveDirective("adaptive-bg");
      return _withDirectives((_openBlock(), _createElementBlock("div", {
        class: "_panel",
        style: "position: relative;"
      }, [
        _createElementVNode(
          "div",
          {
            class: _normalizeClass(_ctx.$style.banner),
            style: _normalizeStyle(__props.user.bannerUrl ? { backgroundImage: `url(${__props.user.bannerUrl})` } : "")
          },
          null,
          6
          /* CLASS, STYLE */
        ),
        _createVNode(_component_MkAvatar, {
          class: _normalizeClass(_ctx.$style.avatar),
          user: __props.user,
          indicator: ""
        }, null, 10, ["user"]),
        _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.title) },
          [_createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.name) },
            [_createVNode(_component_MkUserName, {
              user: __props.user,
              nowrap: false
            }, null, 8, ["user", "nowrap"])],
            2
            /* CLASS */
          ), _createElementVNode(
            "p",
            { class: _normalizeClass(_ctx.$style.username) },
            [_createVNode(_component_MkAcct, { user: __props.user }, null, 8, ["user"])],
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        ),
        _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.description) },
          [__props.user.description ? (_openBlock(), _createElementBlock(
            "div",
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.mfm)
            },
            [_createVNode(_component_Mfm, {
              text: __props.user.description,
              author: __props.user
            }, null, 8, ["text", "author"])],
            2
            /* CLASS */
          )) : (_openBlock(), _createElementBlock(
            "span",
            {
              key: 1,
              style: "opacity: 0.7;"
            },
            _toDisplayString(_unref(i18n).ts.noAccountDescription),
            1
            /* TEXT */
          ))],
          2
          /* CLASS */
        ),
        _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.footer) },
          [!isFollowing.value ? (_openBlock(), _createBlock(MkButton, {
            key: 0,
            primary: "",
            gradate: "",
            rounded: "",
            full: "",
            onClick: follow
          }, {
            default: _withCtx(() => [
              _hoisted_1,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.follow),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })) : (_openBlock(), _createElementBlock("div", {
            key: 1,
            style: "opacity: 0.7; text-align: center;"
          }, [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.youFollowing) + " ",
            1
            /* TEXT */
          ), _hoisted_2]))],
          2
          /* CLASS */
        )
      ])), [[_directive_adaptive_bg]]);
    };
  }
};
