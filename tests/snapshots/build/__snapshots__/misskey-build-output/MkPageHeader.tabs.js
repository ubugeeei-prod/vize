import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue";
import { nextTick, onMounted, onUnmounted, useTemplateRef, watch } from "vue";
import { prefer } from "@/preferences.js";
import { genId } from "@/utility/id.js";
export {};
export default {
  __name: "MkPageHeader.tabs",
  props: {
    tabs: {
      type: Array,
      required: false,
      default: () => []
    },
    tab: {
      type: String,
      required: false
    },
    rootEl: {
      type: null,
      required: false
    }
  },
  emits: ["update:tab", "tabClick"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const cssAnchorSupported = CSS.supports("position-anchor", "--anchor-name");
    const tabAnchorName = `--${genId()}-currentTab`;
    const el = useTemplateRef("el");
    const tabHighlightEl = useTemplateRef("tabHighlightEl");
    const tabRefs = {};
    function getTabStyle(t) {
      if (!cssAnchorSupported) return {};
      if (t.key === props.tab) {
        return { anchorName: tabAnchorName };
      } else {
        return {};
      }
    }
    function onTabMousedown(tab, ev) {
      // ユーザビリティの観点からmousedown時にはonClickは呼ばない
      if (tab.key) {
        emit("update:tab", tab.key);
      }
    }
    function onTabClick(t, ev) {
      emit("tabClick", t.key);
      if (t.onClick) {
        ev.preventDefault();
        ev.stopPropagation();
        t.onClick(ev);
      }
      if (t.key) {
        emit("update:tab", t.key);
      }
    }
    function renderTab() {
      if (cssAnchorSupported) return;
      const tabEl = props.tab ? tabRefs[props.tab] : undefined;
      if (tabEl && tabHighlightEl.value && tabHighlightEl.value.parentElement) {
        // offsetWidth や offsetLeft は少数を丸めてしまうため getBoundingClientRect を使う必要がある
        // https://developer.mozilla.org/ja/docs/Web/API/HTMLElement/offsetWidth#%E5%80%A4
        const parentRect = tabHighlightEl.value.parentElement.getBoundingClientRect();
        const rect = tabEl.getBoundingClientRect();
        tabHighlightEl.value.style.width = rect.width + "px";
        tabHighlightEl.value.style.left = rect.left - parentRect.left + tabHighlightEl.value.parentElement.scrollLeft + "px";
      }
    }
    function onTabWheel(ev) {
      if (ev.deltaY !== 0 && ev.deltaX === 0) {
        ev.preventDefault();
        ev.stopPropagation();
        ev.currentTarget.scrollBy({
          left: ev.deltaY,
          behavior: "instant"
        });
      }
      return false;
    }
    let entering = false;
    async function enter(el) {
      if (!(el instanceof HTMLElement)) return;
      entering = true;
      const elementWidth = el.getBoundingClientRect().width;
      el.style.width = "0";
      el.style.paddingLeft = "0";
      el.offsetWidth;
      el.style.width = `${elementWidth}px`;
      el.style.paddingLeft = "";
      nextTick(() => {
        entering = false;
      });
      window.setTimeout(renderTab, 170);
    }
    function afterEnter(el) {
      if (!(el instanceof HTMLElement)) return;
      // element.style.width = '';
    }
    async function leave(el) {
      if (!(el instanceof HTMLElement)) return;
      const elementWidth = el.getBoundingClientRect().width;
      el.style.width = `${elementWidth}px`;
      el.style.paddingLeft = "";
      el.offsetWidth;
      el.style.width = "0";
      el.style.paddingLeft = "0";
    }
    function afterLeave(el) {
      if (!(el instanceof HTMLElement)) return;
      el.style.width = "";
    }
    let ro2;
    onMounted(() => {
      if (!cssAnchorSupported) {
        watch([() => props.tab, () => props.tabs], () => {
          nextTick(() => {
            if (entering) return;
            renderTab();
          });
        }, { immediate: true });
        if (props.rootEl) {
          ro2 = new ResizeObserver(() => {
            if (window.document.body.contains(el.value)) {
              nextTick(() => renderTab());
            }
          });
          ro2.observe(props.rootEl);
        }
      }
    });
    onUnmounted(() => {
      if (ro2) ro2.disconnect();
    });
    return (_ctx, _cache) => {
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock(
        "div",
        {
          ref_key: "el",
          ref: el,
          class: _normalizeClass(_ctx.$style.tabs),
          style: _normalizeStyle({ "--tabAnchorName": _unref(tabAnchorName) }),
          onWheel: onTabWheel
        },
        [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.tabsInner) },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(__props.tabs, (t) => {
              return _withDirectives((_openBlock(), _createElementBlock("button", {
                ref: (el) => tabRefs[t.key] = el,
                class: _normalizeClass(["_button", [_ctx.$style.tab, {
                  [_ctx.$style.active]: t.key != null && t.key === props.tab,
                  [_ctx.$style.animate]: _unref(prefer).s.animation
                }]]),
                style: _normalizeStyle(getTabStyle(t)),
                onMousedown: (ev) => onTabMousedown(t, ev),
                onClick: (ev) => onTabClick(t, ev)
              }, [_createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.tabInner) },
                [t.icon ? (_openBlock(), _createElementBlock(
                  "i",
                  {
                    key: 0,
                    class: _normalizeClass([_ctx.$style.tabIcon, t.icon])
                  },
                  null,
                  2
                  /* CLASS */
                )) : _createCommentVNode("v-if", true), !t.iconOnly || !_unref(prefer).s.animation && t.key === __props.tab ? (_openBlock(), _createElementBlock(
                  "div",
                  {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.tabTitle)
                  },
                  _toDisplayString(t.title),
                  3
                  /* TEXT, CLASS */
                )) : (_openBlock(), _createBlock(_Transition, {
                  key: 1,
                  mode: "in-out",
                  onEnter: enter,
                  onAfterEnter: afterEnter,
                  onLeave: leave,
                  onAfterLeave: afterLeave
                }, {
                  default: _withCtx(() => [_withDirectives(_createElementVNode(
                    "div",
                    { class: _normalizeClass([_ctx.$style.tabTitle, _ctx.$style.animate]) },
                    _toDisplayString(t.title),
                    3
                    /* TEXT, CLASS */
                  ), [[_vShow, t.key === __props.tab]])]),
                  _: 2
                }))],
                2
                /* CLASS */
              )], 558, ["onMousedown", "onClick"])), [[
                _directive_tooltip,
                t.title,
                void 0,
                { noDelay: true }
              ]]);
            }),
            256
            /* UNKEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        ), _createElementVNode(
          "div",
          {
            ref_key: "tabHighlightEl",
            ref: tabHighlightEl,
            class: _normalizeClass([_ctx.$style.tabHighlight, { [_ctx.$style.animate]: _unref(prefer).s.animation }])
          },
          null,
          2
          /* CLASS */
        )],
        38
        /* CLASS, STYLE, NEED_HYDRATION */
      );
    };
  }
};
