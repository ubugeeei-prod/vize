import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, renderSlot as _renderSlot, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue";
import { onMounted, watch, onBeforeUnmount, ref, useTemplateRef } from "vue";
import tinycolor from "tinycolor2";
const SAFE_FOR_HTML_ID = "abcdefghijklmnopqrstuvwxyz";
export default {
  __name: "MkTagCloud",
  setup(__props, { expose: __expose }) {
    const loaded = !!window.TagCanvas;
    const computedStyle = getComputedStyle(window.document.documentElement);
    const idForCanvas = Array.from({ length: 16 }, () => SAFE_FOR_HTML_ID[Math.floor(Math.random() * SAFE_FOR_HTML_ID.length)]).join("");
    const idForTags = Array.from({ length: 16 }, () => SAFE_FOR_HTML_ID[Math.floor(Math.random() * SAFE_FOR_HTML_ID.length)]).join("");
    const available = ref(false);
    const rootEl = useTemplateRef("rootEl");
    const canvasEl = useTemplateRef("canvasEl");
    const tagsEl = useTemplateRef("tagsEl");
    const width = ref(300);
    watch(available, () => {
      try {
        window.TagCanvas.Start(idForCanvas, idForTags, {
          textColour: "#ffffff",
          outlineColour: tinycolor(computedStyle.getPropertyValue("--MI_THEME-accent")).toHexString(),
          outlineRadius: 10,
          initial: [-.03, -.01],
          frontSelect: true,
          imageRadius: 8,
          //dragControl: true,
          dragThreshold: 3,
          wheelZoom: false,
          reverse: true,
          depth: .5,
          maxSpeed: .2,
          minSpeed: .003,
          stretchX: .8,
          stretchY: .8
        });
      } catch (err) {}
    });
    onMounted(() => {
      if (rootEl.value) width.value = rootEl.value.offsetWidth;
      if (loaded) {
        available.value = true;
      } else {
        window.document.head.appendChild(Object.assign(window.document.createElement("script"), {
          async: true,
          src: "/client-assets/tagcanvas.min.js"
        })).addEventListener("load", () => available.value = true);
      }
    });
    onBeforeUnmount(() => {
      if (window.TagCanvas) window.TagCanvas.Delete(idForCanvas);
    });
    __expose({ update: () => {
      window.TagCanvas.Update(idForCanvas);
    } });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        {
          ref_key: "rootEl",
          ref: rootEl,
          class: _normalizeClass(_ctx.$style.root)
        },
        [_createElementVNode("canvas", {
          id: _unref(idForCanvas),
          ref_key: "canvasEl",
          ref: canvasEl,
          style: "display: block;",
          width: width.value,
          height: "300",
          onContextmenu: _cache[0] || (_cache[0] = _withModifiers(() => {}, ["prevent"]))
        }, null, 40, ["id", "width"]), _createElementVNode("div", {
          id: _unref(idForTags),
          ref_key: "tagsEl",
          ref: tagsEl,
          class: _normalizeClass(_ctx.$style.tags)
        }, [_createElementVNode("ul", null, [_renderSlot(_ctx.$slots, "default")])], 10, ["id"])],
        2
        /* CLASS */
      );
    };
  }
};
