import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue";
export default {
  __name: "MkRolePreview",
  props: {
    role: {
      type: null,
      required: true
    },
    forModeration: {
      type: Boolean,
      required: true
    },
    detailed: {
      type: Boolean,
      required: false,
      default: true
    }
  },
  setup(__props) {
    const props = __props;
    return (_ctx, _cache) => {
      const _component_MkA = _resolveComponent("MkA");
      const _directive_adaptive_bg = _resolveDirective("adaptive-bg");
      return _openBlock(), _createBlock(_component_MkA, {
        to: __props.forModeration ? `/admin/roles/${__props.role.id}` : `/roles/${__props.role.id}`,
        class: _normalizeClass(_ctx.$style.root),
        tabindex: "-1",
        style: _normalizeStyle({ "--color": __props.role.color })
      }, {
        default: _withCtx(() => [__props.forModeration ? (_openBlock(), _createElementBlock(
          _Fragment,
          { key: 0 },
          ["isPublic" in __props.role && __props.role.isPublic ? (_openBlock(), _createElementBlock(
            "i",
            {
              key: 0,
              class: _normalizeClass(["ti ti-world", _ctx.$style.icon]),
              style: "color: var(--MI_THEME-success)"
            },
            null,
            2
            /* CLASS */
          )) : (_openBlock(), _createElementBlock(
            "i",
            {
              key: 1,
              class: _normalizeClass(["ti ti-lock", _ctx.$style.icon]),
              style: "color: var(--MI_THEME-warn)"
            },
            null,
            2
            /* CLASS */
          ))],
          64
          /* STABLE_FRAGMENT */
        )) : _createCommentVNode("v-if", true), _withDirectives(_createElementVNode(
          "div",
          { class: _normalizeClass(["_panel", _ctx.$style.body]) },
          [_createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.bodyTitle) },
            [
              _createElementVNode(
                "span",
                { class: _normalizeClass(_ctx.$style.bodyIcon) },
                [__props.role.iconUrl ? (_openBlock(), _createElementBlock("img", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.bodyBadge),
                  src: __props.role.iconUrl
                }, null, 10, ["src"])) : (_openBlock(), _createElementBlock(
                  _Fragment,
                  { key: 1 },
                  [__props.role.isAdministrator ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-crown",
                    style: "color: var(--MI_THEME-accent);"
                  })) : __props.role.isModerator ? (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-shield",
                    style: "color: var(--MI_THEME-accent);"
                  })) : (_openBlock(), _createElementBlock("i", {
                    key: 2,
                    class: "ti ti-user",
                    style: "opacity: 0.7;"
                  }))],
                  64
                  /* STABLE_FRAGMENT */
                ))],
                2
                /* CLASS */
              ),
              _createElementVNode(
                "span",
                { class: _normalizeClass(_ctx.$style.bodyName) },
                _toDisplayString(__props.role.name),
                3
                /* TEXT, CLASS */
              ),
              __props.detailed && "target" in __props.role && "usersCount" in __props.role ? (_openBlock(), _createElementBlock(
                _Fragment,
                { key: 0 },
                [__props.role.target === "manual" ? (_openBlock(), _createElementBlock(
                  "span",
                  {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.bodyUsers)
                  },
                  _toDisplayString(__props.role.usersCount) + " users",
                  3
                  /* TEXT, CLASS */
                )) : __props.role.target === "conditional" ? (_openBlock(), _createElementBlock(
                  "span",
                  {
                    key: 1,
                    class: _normalizeClass(_ctx.$style.bodyUsers)
                  },
                  "? users",
                  2
                  /* CLASS */
                )) : _createCommentVNode("v-if", true)],
                64
                /* STABLE_FRAGMENT */
              )) : _createCommentVNode("v-if", true)
            ],
            2
            /* CLASS */
          ), _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.bodyDescription) },
            _toDisplayString(__props.role.description),
            3
            /* TEXT, CLASS */
          )],
          2
          /* CLASS */
        ), [[_directive_adaptive_bg]])]),
        _: 2
      }, 1038, ["to"]);
    };
  }
};
