import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue";
import { shallowRef, useTemplateRef } from "vue";
import { useInterval } from "@@/js/use-interval.js";
import { useWidgetPropsManager } from "./widget.js";
import MkContainer from "@/components/MkContainer.vue";
import MkTagCloud from "@/components/MkTagCloud.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { getProxiedImageUrlNullable } from "@/utility/media-proxy.js";
import { i18n } from "@/i18n.js";
const name = "instanceCloud";
export default {
  __name: "WidgetInstanceCloud",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = { transparent: {
      type: "boolean",
      label: i18n.ts._widgetOptions.transparent,
      default: false
    } };
    const { widgetProps, configure } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    const cloud = useTemplateRef("cloud");
    const activeInstances = shallowRef(null);
    function onInstanceClick(i) {
      os.pageWindow(`/instance-info/${i.host}`);
    }
    useInterval(() => {
      misskeyApi("federation/instances", {
        sort: "+latestRequestReceivedAt",
        limit: 25
      }).then((res) => {
        activeInstances.value = res;
        if (cloud.value) cloud.value.update();
      });
    }, 1e3 * 60 * 3, {
      immediate: true,
      afterMounted: true
    });
    function getInstanceIcon(instance) {
      return getProxiedImageUrlNullable(instance.iconUrl, "preview") ?? getProxiedImageUrlNullable(instance.faviconUrl, "preview") ?? "/client-assets/dummy.png";
    }
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkContainer, {
        naked: _unref(widgetProps).transparent,
        showHeader: false,
        class: "mkw-instance-cloud"
      }, {
        default: _withCtx(() => [_createElementVNode("div", { class: "" }, [activeInstances.value ? (_openBlock(), _createBlock(
          MkTagCloud,
          {
            key: 0,
            ref_key: "cloud",
            ref: cloud
          },
          {
            default: _withCtx(() => [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(activeInstances.value, (instance) => {
                return _openBlock(), _createElementBlock("li", { key: instance.id }, [_createElementVNode("a", { onClick: _withModifiers(($event) => onInstanceClick(instance), ["prevent"]) }, [_createElementVNode("img", {
                  style: "width: 32px;",
                  src: getInstanceIcon(instance)
                }, null, 8, ["src"])], 8, ["onClick"])]);
              }),
              128
              /* KEYED_FRAGMENT */
            ))]),
            _: 2
          },
          1536
          /* NEED_PATCH, DYNAMIC_SLOTS */
        )) : _createCommentVNode("v-if", true)])]),
        _: 1
      }, 8, ["naked", "showHeader"]);
    };
  }
};
