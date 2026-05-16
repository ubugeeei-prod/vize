import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { ref, computed } from "vue";
import XDecoration from "./avatar-decoration.decoration.vue";
import XDialog from "./avatar-decoration.dialog.vue";
import MkButton from "@/components/MkButton.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { ensureSignin } from "@/i.js";
import MkInfo from "@/components/MkInfo.vue";
import { definePage } from "@/page.js";
export default {
  __name: "avatar-decoration",
  setup(__props) {
    const $i = ensureSignin();
    const loading = ref(true);
    const avatarDecorations = ref([]);
    misskeyApi("get-avatar-decorations").then((_avatarDecorations) => {
      avatarDecorations.value = _avatarDecorations;
      loading.value = false;
    });
    function openAttachedDecoration(index) {
      openDecoration(avatarDecorations.value.find((d) => d.id === $i.avatarDecorations[index].id) ?? {
        id: "",
        url: "",
        name: "?",
        roleIdsThatCanBeUsedThisDecoration: []
      }, index);
    }
    async function openDecoration(avatarDecoration, index) {
      const { dispose } = os.popup(XDialog, {
        decoration: avatarDecoration,
        usingIndex: index ?? null
      }, {
        "attach": async (payload) => {
          const decoration = {
            id: avatarDecoration.id,
            url: avatarDecoration.url,
            angle: payload.angle,
            flipH: payload.flipH,
            offsetX: payload.offsetX,
            offsetY: payload.offsetY
          };
          const update = [...$i.avatarDecorations, decoration];
          await os.apiWithDialog("i/update", { avatarDecorations: update });
          $i.avatarDecorations = update;
        },
        "update": async (payload) => {
          const decoration = {
            id: avatarDecoration.id,
            url: avatarDecoration.url,
            angle: payload.angle,
            flipH: payload.flipH,
            offsetX: payload.offsetX,
            offsetY: payload.offsetY
          };
          const update = [...$i.avatarDecorations];
          update[index] = decoration;
          await os.apiWithDialog("i/update", { avatarDecorations: update });
          $i.avatarDecorations = update;
        },
        "detach": async () => {
          const update = [...$i.avatarDecorations];
          update.splice(index, 1);
          await os.apiWithDialog("i/update", { avatarDecorations: update });
          $i.avatarDecorations = update;
        },
        closed: () => dispose()
      });
    }
    function detachAllDecorations() {
      os.confirm({
        type: "warning",
        text: i18n.ts.areYouSure
      }).then(async ({ canceled }) => {
        if (canceled) return;
        await os.apiWithDialog("i/update", { avatarDecorations: [] });
        $i.avatarDecorations = [];
      });
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.avatarDecorations,
      icon: "ti ti-sparkles"
    }));
    return (_ctx, _cache) => {
      const _component_MkAvatar = _resolveComponent("MkAvatar");
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      const _directive_panel = _resolveDirective("panel");
      return _openBlock(), _createBlock(_component_SearchMarker, {
        path: "/settings/avatar-decoration",
        label: _unref(i18n).ts.avatarDecorations,
        keywords: [
          "avatar",
          "icon",
          "decoration"
        ],
        icon: "ti ti-sparkles"
      }, {
        default: _withCtx(() => [_createElementVNode("div", null, [!loading.value ? (_openBlock(), _createElementBlock("div", {
          key: 0,
          class: "_gaps"
        }, [
          _createVNode(MkInfo, null, {
            default: _withCtx(() => [
              _createTextVNode(
                _toDisplayString(_unref(i18n).tsx._profile.avatarDecorationMax({ max: _unref($i).policies.avatarDecorationLimit })),
                1
                /* TEXT */
              ),
              _createTextVNode(" ("),
              _createTextVNode(
                _toDisplayString(_unref(i18n).tsx.remainingN({ n: _unref($i).policies.avatarDecorationLimit - _unref($i).avatarDecorations.length })),
                1
                /* TEXT */
              ),
              _createTextVNode(")")
            ]),
            _: 1
          }),
          _createVNode(_component_MkAvatar, {
            class: _normalizeClass(_ctx.$style.avatar),
            user: _unref($i),
            forceShowDecoration: ""
          }, null, 10, ["user"]),
          _unref($i).avatarDecorations.length > 0 ? _withDirectives((_openBlock(), _createElementBlock(
            "div",
            {
              key: 0,
              class: _normalizeClass(["_gaps_s", _ctx.$style.current])
            },
            [
              _createElementVNode(
                "div",
                null,
                _toDisplayString(_unref(i18n).ts.inUse),
                1
                /* TEXT */
              ),
              _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.decorations) },
                [(_openBlock(true), _createElementBlock(
                  _Fragment,
                  null,
                  _renderList(_unref($i).avatarDecorations, (avatarDecoration, i) => {
                    return _openBlock(), _createBlock(XDecoration, {
                      decoration: avatarDecorations.value.find((d) => d.id === avatarDecoration.id) ?? {
                        id: "",
                        url: "",
                        name: "?",
                        roleIdsThatCanBeUsedThisDecoration: []
                      },
                      angle: avatarDecoration.angle,
                      flipH: avatarDecoration.flipH,
                      offsetX: avatarDecoration.offsetX,
                      offsetY: avatarDecoration.offsetY,
                      active: true,
                      onClick: ($event) => openAttachedDecoration(i)
                    }, null, 8, [
                      "decoration",
                      "angle",
                      "flipH",
                      "offsetX",
                      "offsetY",
                      "active",
                      "onClick"
                    ]);
                  }),
                  256
                  /* UNKEYED_FRAGMENT */
                ))],
                2
                /* CLASS */
              ),
              _createVNode(MkButton, {
                danger: "",
                onClick: detachAllDecorations
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.detachAll),
                  1
                  /* TEXT */
                )]),
                _: 1
              })
            ],
            2
            /* CLASS */
          )), [[_directive_panel]]) : _createCommentVNode("v-if", true),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.decorations) },
            [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(avatarDecorations.value, (avatarDecoration) => {
                return _openBlock(), _createBlock(XDecoration, {
                  key: avatarDecoration.id,
                  decoration: avatarDecoration,
                  onClick: ($event) => openDecoration(avatarDecoration)
                }, null, 8, ["decoration", "onClick"]);
              }),
              128
              /* KEYED_FRAGMENT */
            ))],
            2
            /* CLASS */
          )
        ])) : (_openBlock(), _createElementBlock("div", { key: 1 }, [_createVNode(_component_MkLoading)]))])]),
        _: 1
      }, 8, ["label", "keywords"]);
    };
  }
};
