import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-pin" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { ref, computed, watch } from "vue";
import MkButton from "@/components/MkButton.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import { $i } from "@/i.js";
import { prefer } from "@/preferences.js";
import { updateCurrentAccountPartial } from "@/accounts.js";
export default {
  __name: "announcement",
  props: { announcementId: {
    type: String,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const announcement = ref(null);
    const error = ref(null);
    const path = computed(() => props.announcementId);
    function _fetch_() {
      announcement.value = null;
      misskeyApi("announcements/show", { announcementId: props.announcementId }).then(async (_announcement) => {
        announcement.value = _announcement;
      }).catch((err) => {
        error.value = err;
      });
    }
    async function read(target) {
      if (target.needConfirmationToRead) {
        const confirm = await os.confirm({
          type: "question",
          title: i18n.ts._announcement.readConfirmTitle,
          text: i18n.tsx._announcement.readConfirmText({ title: target.title })
        });
        if (confirm.canceled) return;
      }
      target.isRead = true;
      await misskeyApi("i/read-announcement", { announcementId: target.id });
      if ($i) {
        updateCurrentAccountPartial({ unreadAnnouncements: $i.unreadAnnouncements.filter((a) => a.id !== target.id) });
      }
    }
    watch(() => path.value, _fetch_, { immediate: true });
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: announcement.value ? announcement.value.title : i18n.ts.announcements,
      icon: "ti ti-speakerphone"
    }));
    return (_ctx, _cache) => {
      const _component_Mfm = _resolveComponent("Mfm");
      const _component_MkTime = _resolveComponent("MkTime");
      const _component_MkError = _resolveComponent("MkError");
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value
      }, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [_createVNode(_Transition, {
          enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.fadeEnterActive : "",
          leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.fadeLeaveActive : "",
          enterFromClass: _unref(prefer).s.animation ? _ctx.$style.fadeEnterFrom : "",
          leaveToClass: _unref(prefer).s.animation ? _ctx.$style.fadeLeaveTo : "",
          mode: "out-in"
        }, {
          default: _withCtx(() => [announcement.value ? (_openBlock(), _createElementBlock(
            "div",
            {
              key: announcement.value.id,
              class: _normalizeClass(["_panel", _ctx.$style.announcement])
            },
            [
              announcement.value.forYou ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.forYou)
                },
                [_hoisted_1, _createTextVNode(
                  " " + _toDisplayString(_unref(i18n).ts.forYou),
                  1
                  /* TEXT */
                )],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true),
              _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.header) },
                [
                  _unref($i) && !announcement.value.silence && !announcement.value.isRead ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    style: "margin-right: 0.5em;"
                  }, "🆕")) : _createCommentVNode("v-if", true),
                  _createElementVNode("span", { style: "margin-right: 0.5em;" }, [announcement.value.icon === "info" ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-info-circle"
                  })) : announcement.value.icon === "warning" ? (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-alert-triangle",
                    style: "color: var(--MI_THEME-warn);"
                  })) : announcement.value.icon === "error" ? (_openBlock(), _createElementBlock("i", {
                    key: 2,
                    class: "ti ti-circle-x",
                    style: "color: var(--MI_THEME-error);"
                  })) : announcement.value.icon === "success" ? (_openBlock(), _createElementBlock("i", {
                    key: 3,
                    class: "ti ti-check",
                    style: "color: var(--MI_THEME-success);"
                  })) : _createCommentVNode("v-if", true)]),
                  _createVNode(_component_Mfm, {
                    text: announcement.value.title,
                    class: "_selectable"
                  }, null, 8, ["text"])
                ],
                2
                /* CLASS */
              ),
              _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.content) },
                [
                  _createVNode(_component_Mfm, {
                    text: announcement.value.text,
                    class: "_selectable"
                  }, null, 8, ["text"]),
                  announcement.value.imageUrl ? (_openBlock(), _createElementBlock("img", {
                    key: 0,
                    src: announcement.value.imageUrl
                  }, null, 8, ["src"])) : _createCommentVNode("v-if", true),
                  _createElementVNode("div", { style: "margin-top: 8px; opacity: 0.7; font-size: 85%;" }, [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.createdAt) + ": ",
                    1
                    /* TEXT */
                  ), _createVNode(_component_MkTime, {
                    time: announcement.value.createdAt,
                    mode: "detail"
                  }, null, 8, ["time"])]),
                  announcement.value.updatedAt ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    style: "opacity: 0.7; font-size: 85%;"
                  }, [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.updatedAt) + ": ",
                    1
                    /* TEXT */
                  ), _createVNode(_component_MkTime, {
                    time: announcement.value.updatedAt,
                    mode: "detail"
                  }, null, 8, ["time"])])) : _createCommentVNode("v-if", true)
                ],
                2
                /* CLASS */
              ),
              _unref($i) && !announcement.value.silence && !announcement.value.isRead ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.footer)
                },
                [_createVNode(MkButton, {
                  primary: "",
                  onClick: _cache[0] || (_cache[0] = ($event) => read(announcement.value))
                }, {
                  default: _withCtx(() => [
                    _hoisted_2,
                    _createTextVNode(" "),
                    _createTextVNode(
                      _toDisplayString(_unref(i18n).ts.gotIt),
                      1
                      /* TEXT */
                    )
                  ]),
                  _: 1
                })],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true)
            ],
            2
            /* CLASS */
          )) : error.value ? (_openBlock(), _createBlock(_component_MkError, {
            key: 1,
            onRetry: _cache[1] || (_cache[1] = ($event) => _fetch_())
          })) : (_openBlock(), _createBlock(_component_MkLoading, { key: 2 }))]),
          _: 2
        }, 1032, [
          "enterActiveClass",
          "leaveActiveClass",
          "enterFromClass",
          "leaveToClass"
        ])])]),
        _: 1
      }, 8, ["actions", "tabs"]);
    };
  }
};
