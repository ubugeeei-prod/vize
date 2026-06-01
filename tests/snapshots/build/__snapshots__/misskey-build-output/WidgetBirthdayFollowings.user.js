import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("span", null, " / ");
const _hoisted_2 = { class: "_monospace" };
import { computed } from "vue";
import MkUserCardMini from "@/components/MkUserCardMini.vue";
import * as os from "@/os.js";
import { i18n } from "@/i18n.js";
import { useLowresTime } from "@/composables/use-lowres-time.js";
import { userPage, acct } from "@/filters/user.js";
export default {
  __name: "WidgetBirthdayFollowings.user",
  props: { item: {
    type: null,
    required: true
  } },
  setup(__props) {
    const props = __props;
    const now = useLowresTime();
    const nowDate = computed(() => {
      const date = new Date(now.value);
      date.setHours(0, 0, 0, 0);
      return date;
    });
    const birthdayDate = computed(() => {
      const [year, month, day] = props.item.birthday.split("-").map((v) => parseInt(v, 10));
      return new Date(year, month - 1, day, 0, 0, 0, 0);
    });
    const countdownDate = computed(() => {
      const days = Math.floor((birthdayDate.value.getTime() - nowDate.value.getTime()) / (1e3 * 60 * 60 * 24));
      if (days === 0) {
        return i18n.ts.today;
      } else if (days > 0) {
        return i18n.tsx._timeIn.days({ n: days });
      } else {
        return i18n.tsx._ago.daysAgo({ n: Math.abs(days) });
      }
    });
    return (_ctx, _cache) => {
      const _component_MkA = _resolveComponent("MkA");
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [_createVNode(_component_MkA, {
          to: _unref(userPage)(__props.item.user),
          style: "overflow: clip;"
        }, {
          default: _withCtx(() => [_createVNode(MkUserCardMini, {
            user: __props.item.user,
            withChart: false,
            style: "text-overflow: ellipsis; background: inherit; border-radius: unset;"
          }, {
            sub: _withCtx(() => [
              _createElementVNode(
                "span",
                null,
                _toDisplayString(countdownDate.value),
                1
                /* TEXT */
              ),
              _hoisted_1,
              _createElementVNode(
                "span",
                _hoisted_2,
                "@" + _toDisplayString(_unref(acct)(__props.item.user)),
                1
                /* TEXT */
              )
            ]),
            _: 1
          }, 8, ["user", "withChart"])]),
          _: 1
        }, 8, ["to"]), _withDirectives(_createElementVNode(
          "button",
          {
            class: _normalizeClass(["_button", _ctx.$style.post]),
            onClick: _cache[0] || (_cache[0] = ($event) => os.post({ initialText: `@${__props.item.user.username}${__props.item.user.host ? `@${__props.item.user.host}` : ""} ` }))
          },
          [_createElementVNode(
            "i",
            { class: _normalizeClass(["ti-fw ti ti-confetti", _ctx.$style.postIcon]) },
            null,
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        ), [[
          _directive_tooltip,
          _unref(i18n).ts.note,
          void 0,
          { noDelay: true }
        ]])],
        2
        /* CLASS */
      );
    };
  }
};
