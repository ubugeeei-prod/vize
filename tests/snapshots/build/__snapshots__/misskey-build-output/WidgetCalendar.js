import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("span", { style: "display: inline-block; transform: scaleX(-1);" }, "🎉");
import { ref, watch } from "vue";
import { useWidgetPropsManager } from "./widget.js";
import { i18n } from "@/i18n.js";
import { useLowresTime, TIME_UPDATE_INTERVAL } from "@/composables/use-lowres-time.js";
const name = "calendar";
export default {
  __name: "WidgetCalendar",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = { transparent: {
      type: "boolean",
      label: i18n.ts._widgetOptions.transparent,
      default: false
    } };
    const { widgetProps, configure } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    const fNow = useLowresTime();
    const year = ref(0);
    const month = ref(0);
    const day = ref(0);
    const weekDay = ref("");
    const yearP = ref(0);
    const monthP = ref(0);
    const dayP = ref(0);
    const isHoliday = ref(false);
    const nextDay = new Date();
    nextDay.setHours(24, 0, 0, 0);
    let nextDayMidnightTime = nextDay.getTime();
    let nextDayTimer = null;
    function update(time) {
      const now = new Date(time);
      const nd = now.getDate();
      const nm = now.getMonth();
      const ny = now.getFullYear();
      year.value = ny;
      month.value = nm + 1;
      day.value = nd;
      weekDay.value = [
        i18n.ts._weekday.sunday,
        i18n.ts._weekday.monday,
        i18n.ts._weekday.tuesday,
        i18n.ts._weekday.wednesday,
        i18n.ts._weekday.thursday,
        i18n.ts._weekday.friday,
        i18n.ts._weekday.saturday
      ][now.getDay()];
      const dayNumer = now.getTime() - new Date(ny, nm, nd).getTime();
      const dayDenom = 1e3 * 60 * 60 * 24;
      const monthNumer = now.getTime() - new Date(ny, nm, 1).getTime();
      const monthDenom = new Date(ny, nm + 1, 1).getTime() - new Date(ny, nm, 1).getTime();
      const yearNumer = now.getTime() - new Date(ny, 0, 1).getTime();
      const yearDenom = new Date(ny + 1, 0, 1).getTime() - new Date(ny, 0, 1).getTime();
      dayP.value = dayNumer / dayDenom * 100;
      monthP.value = monthNumer / monthDenom * 100;
      yearP.value = yearNumer / yearDenom * 100;
      isHoliday.value = now.getDay() === 0 || now.getDay() === 6;
    }
    watch(fNow, (to) => {
      update(to);
      // 次回更新までに日付が変わる場合、日付が変わった直後に強制的に更新するタイマーをセットする
      if (nextDayMidnightTime - to <= TIME_UPDATE_INTERVAL) {
        if (nextDayTimer != null) {
          window.clearTimeout(nextDayTimer);
          nextDayTimer = null;
        }
        nextDayTimer = window.setTimeout(() => {
          update(nextDayMidnightTime);
          nextDayTimer = null;
        }, nextDayMidnightTime - to);
      }
    }, { immediate: true });
    watch(day, () => {
      nextDay.setHours(24, 0, 0, 0);
      nextDayMidnightTime = nextDay.getTime();
    });
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        {
          class: _normalizeClass([_ctx.$style.root, { _panel: !_unref(widgetProps).transparent }]),
          "data-cy-mkw-calendar": ""
        },
        [_createElementVNode(
          "div",
          { class: _normalizeClass([_ctx.$style.calendar, { [_ctx.$style.isHoliday]: isHoliday.value }]) },
          [
            _createElementVNode(
              "p",
              { class: _normalizeClass(_ctx.$style.monthAndYear) },
              [_createElementVNode(
                "span",
                { class: _normalizeClass(_ctx.$style.year) },
                _toDisplayString(_unref(i18n).tsx.yearX({ year: year.value })),
                3
                /* TEXT, CLASS */
              ), _createElementVNode(
                "span",
                { class: _normalizeClass(_ctx.$style.month) },
                _toDisplayString(_unref(i18n).tsx.monthX({ month: month.value })),
                3
                /* TEXT, CLASS */
              )],
              2
              /* CLASS */
            ),
            month.value === 1 && day.value === 1 ? (_openBlock(), _createElementBlock("p", {
              key: 0,
              class: "day"
            }, [_createTextVNode(
              "🎉" + _toDisplayString(_unref(i18n).tsx.dayX({ day: day.value })),
              1
              /* TEXT */
            ), _hoisted_1])) : (_openBlock(), _createElementBlock(
              "p",
              {
                key: 1,
                class: _normalizeClass(_ctx.$style.day)
              },
              _toDisplayString(_unref(i18n).tsx.dayX({ day: day.value })),
              3
              /* TEXT, CLASS */
            )),
            _createElementVNode(
              "p",
              { class: _normalizeClass(_ctx.$style.weekDay) },
              _toDisplayString(weekDay.value),
              3
              /* TEXT, CLASS */
            )
          ],
          2
          /* CLASS */
        ), _createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.info) },
          [
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.infoSection) },
              [_createElementVNode(
                "p",
                { class: _normalizeClass(_ctx.$style.infoText) },
                [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.today),
                  1
                  /* TEXT */
                ), _createElementVNode(
                  "b",
                  { class: _normalizeClass(_ctx.$style.percentage) },
                  _toDisplayString(dayP.value.toFixed(1)) + "%",
                  3
                  /* TEXT, CLASS */
                )],
                2
                /* CLASS */
              ), _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.meter) },
                [_createElementVNode(
                  "div",
                  {
                    class: _normalizeClass(_ctx.$style.meterVal),
                    style: _normalizeStyle({ width: `${dayP.value}%` })
                  },
                  null,
                  6
                  /* CLASS, STYLE */
                )],
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.infoSection) },
              [_createElementVNode(
                "p",
                { class: _normalizeClass(_ctx.$style.infoText) },
                [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.thisMonth),
                  1
                  /* TEXT */
                ), _createElementVNode(
                  "b",
                  { class: _normalizeClass(_ctx.$style.percentage) },
                  _toDisplayString(monthP.value.toFixed(1)) + "%",
                  3
                  /* TEXT, CLASS */
                )],
                2
                /* CLASS */
              ), _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.meter) },
                [_createElementVNode(
                  "div",
                  {
                    class: _normalizeClass(_ctx.$style.meterVal),
                    style: _normalizeStyle({ width: `${monthP.value}%` })
                  },
                  null,
                  6
                  /* CLASS, STYLE */
                )],
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.infoSection) },
              [_createElementVNode(
                "p",
                { class: _normalizeClass(_ctx.$style.infoText) },
                [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.thisYear),
                  1
                  /* TEXT */
                ), _createElementVNode(
                  "b",
                  { class: _normalizeClass(_ctx.$style.percentage) },
                  _toDisplayString(yearP.value.toFixed(1)) + "%",
                  3
                  /* TEXT, CLASS */
                )],
                2
                /* CLASS */
              ), _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.meter) },
                [_createElementVNode(
                  "div",
                  {
                    class: _normalizeClass(_ctx.$style.meterVal),
                    style: _normalizeStyle({ width: `${yearP.value}%` })
                  },
                  null,
                  6
                  /* CLASS, STYLE */
                )],
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            )
          ],
          2
          /* CLASS */
        )],
        2
        /* CLASS */
      );
    };
  }
};
