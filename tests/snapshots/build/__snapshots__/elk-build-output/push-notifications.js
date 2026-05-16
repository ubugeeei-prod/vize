import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'push-notifications',
  setup(__props) {

definePageMeta({
  middleware: ['auth', () => {
    if (!useAppConfig().pwaEnabled)
      return navigateTo('/settings/notifications')
  }],
})
const { t } = useI18n()
useHydratedHead({
  title: () => `${t('settings.notifications.push_notifications.label')} | ${t('settings.notifications.label')} | ${t('nav.settings')}`,
})

return (_ctx: any,_cache: any) => {
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_NotificationPreferences = _resolveComponent("NotificationPreferences")
  const _component_MainContent = _resolveComponent("MainContent")

  return (_openBlock(), _createBlock(_component_MainContent, { back: "" }, {
      title: _withCtx(() => [
        _createVNode(_component_MainTitle, {
          as: "h1",
          secondary: ""
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_ctx.$t('settings.notifications.push_notifications.label')), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })
      ]),
      default: _withCtx(() => [
        _createVNode(_component_NotificationPreferences, { show: "" })
      ]),
      _: 1 /* STABLE */
    }))
}
}

})
