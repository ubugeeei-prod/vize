import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, withCtx as _withCtx, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'DateTime',
  props: {
    datetime: { type: [String, Date], required: true },
    title: { type: String, required: false, default: undefined },
    dateStyle: { type: String, required: false, default: undefined },
    year: { type: String, required: false, default: undefined },
    month: { type: String, required: false, default: undefined },
    day: { type: String, required: false, default: undefined }
  },
  setup(__props: any) {

const props = __props
/**
 * DateTime component that wraps NuxtTime with settings-aware relative date support.
 * Uses the global settings to determine whether to show relative or absolute dates.
 *
 * Note: When relativeDates setting is enabled, the component switches between
 * relative and absolute display based on user preference. The title attribute
 * always shows the full date for accessibility.
 */
const { locale } = useI18n()
const relativeDates = useRelativeDates()
const dateFormatter = new Intl.DateTimeFormat(locale.value, {
  month: 'short',
  day: 'numeric',
  year: 'numeric',
  hour: 'numeric',
  minute: '2-digit',
  timeZoneName: 'short',
})
// Compute the title - always show full date for accessibility
const titleValue = computed(() => {
  if (props.title) return props.title
  const date = typeof props.datetime === 'string' ? new Date(props.datetime) : props.datetime
  return dateFormatter.format(date)
})

return (_ctx: any,_cache: any) => {
  const _component_NuxtTime = _resolveComponent("NuxtTime")
  const _component_ClientOnly = _resolveComponent("ClientOnly")

  return (_openBlock(), _createElementBlock("span", null, [ _createVNode(_component_ClientOnly, null, {
        fallback: _withCtx(() => [
          _createVNode(_component_NuxtTime, {
            datetime: __props.datetime,
            title: titleValue.value,
            "date-style": __props.dateStyle,
            year: __props.year,
            month: __props.month,
            day: __props.day,
            locale: _unref(locale)
          }, null, 8 /* PROPS */, ["datetime", "title", "date-style", "year", "month", "day", "locale"])
        ]),
        default: _withCtx(() => [
          (_unref(relativeDates))
            ? (_openBlock(), _createBlock(_component_NuxtTime, {
              key: 0,
              datetime: __props.datetime,
              title: titleValue.value,
              relative: "",
              locale: _unref(locale)
            }, null, 8 /* PROPS */, ["datetime", "title", "locale"]))
            : (_openBlock(), _createBlock(_component_NuxtTime, {
              key: 1,
              datetime: __props.datetime,
              title: titleValue.value,
              "date-style": __props.dateStyle,
              year: __props.year,
              month: __props.month,
              day: __props.day,
              locale: _unref(locale)
            }, null, 8 /* PROPS */, ["datetime", "title", "date-style", "year", "month", "day", "locale"]))
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
