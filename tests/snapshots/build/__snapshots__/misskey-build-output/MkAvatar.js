import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, mergeProps as _mergeProps, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue";
import { watch, ref, computed } from "vue";
import { extractAvgColorFromBlurhash } from "@@/js/extract-avg-color-from-blurhash.js";
import MkImgWithBlurhash from "../MkImgWithBlurhash.vue";
import MkA from "./MkA.vue";
import { getStaticImageUrl } from "@/utility/media-proxy.js";
import { acct, userPage } from "@/filters/user.js";
import MkUserOnlineIndicator from "@/components/MkUserOnlineIndicator.vue";
import { prefer } from "@/preferences.js";
export default {
  __name: "MkAvatar",
  props: {
    user: {
      type: null,
      required: true
    },
    target: {
      type: [String, null],
      required: false,
      default: null
    },
    link: {
      type: Boolean,
      required: false,
      default: false
    },
    preview: {
      type: Boolean,
      required: false,
      default: false
    },
    indicator: {
      type: Boolean,
      required: false,
      default: false
    },
    decorations: {
      type: Array,
      required: false,
      default: undefined
    },
    forceShowDecoration: {
      type: Boolean,
      required: false,
      default: false
    }
  },
  emits: ["click"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const animation = ref(prefer.s.animation);
    const squareAvatars = ref(prefer.s.squareAvatars);
    const showDecoration = props.forceShowDecoration || prefer.s.showAvatarDecorations;
    const bound = computed(() => props.link ? {
      to: userPage(props.user),
      target: props.target
    } : {});
    const url = computed(() => {
      if (prefer.s.disableShowingAnimatedImages || prefer.s.dataSaver.avatar) return getStaticImageUrl(props.user.avatarUrl);
      return props.user.avatarUrl;
    });
    function onClick(ev) {
      if (props.link) return;
      emit("click", ev);
    }
    function getDecorationUrl(decoration) {
      if (prefer.s.disableShowingAnimatedImages || prefer.s.dataSaver.avatar) return getStaticImageUrl(decoration.url);
      return decoration.url;
    }
    function getDecorationAngle(decoration) {
      const angle = decoration.angle ?? 0;
      return angle === 0 ? undefined : `${angle * 360}deg`;
    }
    function getDecorationScale(decoration) {
      const scaleX = decoration.flipH ? -1 : 1;
      return scaleX === 1 ? undefined : `${scaleX} 1`;
    }
    function getDecorationOffset(decoration) {
      const offsetX = decoration.offsetX ?? 0;
      const offsetY = decoration.offsetY ?? 0;
      return offsetX === 0 && offsetY === 0 ? undefined : `${offsetX * 100}% ${offsetY * 100}%`;
    }
    function getDecorationIsBrink(decoration) {
      return "blink" in decoration && decoration.blink === true;
    }
    const color = ref();
    watch(() => props.user.avatarBlurhash, () => {
      if (props.user.avatarBlurhash == null) return;
      color.value = extractAvgColorFromBlurhash(props.user.avatarBlurhash);
    }, { immediate: true });
    return (_ctx, _cache) => {
      const _directive_user_preview = _resolveDirective("user-preview");
      return _withDirectives((_openBlock(), _createBlock(_resolveDynamicComponent(__props.link ? MkA : "span"), _mergeProps(bound.value, {
        class: ["_noSelect", [_ctx.$style.root, {
          [_ctx.$style.animation]: animation.value,
          [_ctx.$style.cat]: __props.user.isCat,
          [_ctx.$style.square]: squareAvatars.value
        }]],
        style: { color: color.value },
        title: _unref(acct)(__props.user),
        onClick
      }), {
        default: _withCtx(() => [
          _unref(prefer).s.enableHighQualityImagePlaceholders ? (_openBlock(), _createBlock(MkImgWithBlurhash, {
            key: 0,
            class: _normalizeClass(_ctx.$style.inner),
            src: url.value,
            hash: __props.user.avatarBlurhash,
            cover: true,
            onlyAvgColor: true
          }, null, 10, [
            "src",
            "hash",
            "cover",
            "onlyAvgColor"
          ])) : (_openBlock(), _createElementBlock("img", {
            key: 1,
            class: _normalizeClass(_ctx.$style.inner),
            src: url.value,
            alt: "",
            decoding: "async",
            style: "pointer-events: none;"
          }, null, 10, ["src"])),
          __props.indicator ? (_openBlock(), _createBlock(MkUserOnlineIndicator, {
            key: 0,
            class: _normalizeClass(_ctx.$style.indicator),
            user: __props.user
          }, null, 10, ["user"])) : _createCommentVNode("v-if", true),
          __props.user.isCat ? (_openBlock(), _createElementBlock(
            "div",
            {
              key: 0,
              class: _normalizeClass([_ctx.$style.ears])
            },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.earLeft) },
              [false ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.layer)
                },
                [
                  _createElementVNode(
                    "div",
                    {
                      class: _normalizeClass(_ctx.$style.plot),
                      style: _normalizeStyle({ backgroundImage: `url(${JSON.stringify(url.value)})` })
                    },
                    null,
                    6
                    /* CLASS, STYLE */
                  ),
                  _createElementVNode(
                    "div",
                    {
                      class: _normalizeClass(_ctx.$style.plot),
                      style: _normalizeStyle({ backgroundImage: `url(${JSON.stringify(url.value)})` })
                    },
                    null,
                    6
                    /* CLASS, STYLE */
                  ),
                  _createElementVNode(
                    "div",
                    {
                      class: _normalizeClass(_ctx.$style.plot),
                      style: _normalizeStyle({ backgroundImage: `url(${JSON.stringify(url.value)})` })
                    },
                    null,
                    6
                    /* CLASS, STYLE */
                  )
                ],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true)],
              2
              /* CLASS */
            ), _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.earRight) },
              [false ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.layer)
                },
                [
                  _createElementVNode(
                    "div",
                    {
                      class: _normalizeClass(_ctx.$style.plot),
                      style: _normalizeStyle({ backgroundImage: `url(${JSON.stringify(url.value)})` })
                    },
                    null,
                    6
                    /* CLASS, STYLE */
                  ),
                  _createElementVNode(
                    "div",
                    {
                      class: _normalizeClass(_ctx.$style.plot),
                      style: _normalizeStyle({ backgroundImage: `url(${JSON.stringify(url.value)})` })
                    },
                    null,
                    6
                    /* CLASS, STYLE */
                  ),
                  _createElementVNode(
                    "div",
                    {
                      class: _normalizeClass(_ctx.$style.plot),
                      style: _normalizeStyle({ backgroundImage: `url(${JSON.stringify(url.value)})` })
                    },
                    null,
                    6
                    /* CLASS, STYLE */
                  )
                ],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true)],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          )) : _createCommentVNode("v-if", true),
          _unref(showDecoration) ? (_openBlock(), _createElementBlock(
            _Fragment,
            { key: 0 },
            [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(__props.decorations ?? __props.user.avatarDecorations, (decoration) => {
                return _openBlock(), _createElementBlock("img", {
                  class: _normalizeClass([_ctx.$style.decoration, { [_ctx.$style.decorationBlink]: getDecorationIsBrink(decoration) }]),
                  src: getDecorationUrl(decoration),
                  style: _normalizeStyle([{ "-webkit-user-drag": "none" }, {
                    rotate: getDecorationAngle(decoration),
                    scale: getDecorationScale(decoration),
                    translate: getDecorationOffset(decoration)
                  }]),
                  alt: "",
                  draggable: "false"
                }, 14, ["src"]);
              }),
              256
              /* UNKEYED_FRAGMENT */
            ))],
            64
            /* STABLE_FRAGMENT */
          )) : _createCommentVNode("v-if", true)
        ]),
        _: 2
      }, 1040, ["title"])), [[_directive_user_preview, __props.preview ? __props.user.id : undefined]]);
    };
  }
};
