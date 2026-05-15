import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle } from "vue";
import { computed } from "vue";
import { instanceName as localInstanceName } from "@@/js/config.js";
import { instance as localInstance } from "@/instance.js";
import { getProxiedImageUrlNullable } from "@/utility/media-proxy.js";
export default {
  __name: "MkInstanceTicker",
  props: {
    host: {
      type: [String, null],
      required: true
    },
    instance: {
      type: Object,
      required: false
    }
  },
  setup(__props) {
    const props = __props;
    // if no instance data is given, this is for the local instance
    const instanceName = computed(() => props.host == null ? localInstanceName : props.instance?.name ?? props.host);
    const faviconUrl = computed(() => {
      let imageSrc = null;
      if (props.host == null) {
        if (localInstance.iconUrl == null) {
          return "/favicon.ico";
        } else {
          imageSrc = localInstance.iconUrl;
        }
      } else {
        imageSrc = props.instance?.faviconUrl ?? null;
      }
      return getProxiedImageUrlNullable(imageSrc);
    });
    const themeColorStyle = computed(() => {
      const themeColor = (props.host == null ? localInstance.themeColor : props.instance?.themeColor) ?? "#777777";
      return { background: `linear-gradient(90deg, ${themeColor}, ${themeColor}00)` };
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        {
          class: _normalizeClass(_ctx.$style.root),
          style: _normalizeStyle(themeColorStyle.value)
        },
        [faviconUrl.value ? (_openBlock(), _createElementBlock("img", {
          key: 0,
          class: _normalizeClass(_ctx.$style.icon),
          src: faviconUrl.value
        }, null, 10, ["src"])) : _createCommentVNode("v-if", true), _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.name) },
          _toDisplayString(instanceName.value),
          3
          /* TEXT, CLASS */
        )],
        6
        /* CLASS, STYLE */
      );
    };
  }
};
