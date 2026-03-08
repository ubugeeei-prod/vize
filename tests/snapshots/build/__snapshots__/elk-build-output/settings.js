import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'settings',
  setup(__props) {

definePageMeta({
  wideLayout: true,
})
const { t } = useI18n()
useHydratedHead({
  title: () => t('nav.settings'),
})
const route = useRoute()
const isRootPath = computed(() => route.name === 'settings')

return (_ctx: any,_cache: any) => {
  const _component_MainTitle = _resolveComponent("MainTitle")
  const _component_SettingsItem = _resolveComponent("SettingsItem")
  const _component_MainContent = _resolveComponent("MainContent")
  const _component_NuxtPage = _resolveComponent("NuxtPage")
  const _component_ClientOnly = _resolveComponent("ClientOnly")

  return (_openBlock(), _createElementBlock("div", null, [ _createElementVNode("div", {
        "min-h-screen": "",
        flex: ""
      }, [ _createElementVNode("div", {
          border: "e base",
          class: _normalizeClass(isRootPath.value ? 'block lg:flex-none flex-1' : 'hidden lg:block')
        }, [ _createVNode(_component_MainContent, null, {
            title: _withCtx(() => [
              _createVNode(_component_MainTitle, { icon: "i-ri:settings-3-line" }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_ctx.$t('nav.settings')), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            default: _withCtx(() => [
              _createElementVNode("div", {
                "xl:w-97": "",
                "lg:w-78": "",
                "w-full": ""
              }, [
                (_ctx.currentUser)
                  ? (_openBlock(), _createBlock(_component_SettingsItem, {
                    key: 0,
                    command: "",
                    icon: "i-ri:user-line",
                    text: _ctx.$t('settings.profile.label'),
                    to: "/settings/profile",
                    match: _ctx.$route.path.startsWith('/settings/profile/')
                  }, null, 8 /* PROPS */, ["text", "match"]))
                  : _createCommentVNode("v-if", true),
                _createVNode(_component_SettingsItem, {
                  command: "",
                  icon: "i-ri-compasses-2-line",
                  text: _ctx.$t('settings.interface.label'),
                  to: "/settings/interface",
                  match: _ctx.$route.path.startsWith('/settings/interface/')
                }, null, 8 /* PROPS */, ["text", "match"]),
                (_ctx.currentUser)
                  ? (_openBlock(), _createBlock(_component_SettingsItem, {
                    key: 0,
                    command: "",
                    icon: "i-ri:notification-badge-line",
                    text: _ctx.$t('settings.notifications_settings'),
                    to: "/settings/notifications",
                    match: _ctx.$route.path.startsWith('/settings/notifications/')
                  }, null, 8 /* PROPS */, ["text", "match"]))
                  : _createCommentVNode("v-if", true),
                _createVNode(_component_SettingsItem, {
                  command: "",
                  icon: "i-ri-globe-line",
                  text: _ctx.$t('settings.language.label'),
                  to: "/settings/language",
                  match: _ctx.$route.path.startsWith('/settings/language/')
                }, null, 8 /* PROPS */, ["text", "match"]),
                _createVNode(_component_SettingsItem, {
                  command: "",
                  icon: "i-ri-equalizer-line",
                  text: _ctx.$t('settings.preferences.label'),
                  to: "/settings/preferences",
                  match: _ctx.$route.path.startsWith('/settings/preferences/')
                }, null, 8 /* PROPS */, ["text", "match"]),
                _createVNode(_component_SettingsItem, {
                  command: "",
                  icon: "i-ri-group-line",
                  text: _ctx.$t('settings.users.label'),
                  to: "/settings/users",
                  match: _ctx.$route.path.startsWith('/settings/users/')
                }, null, 8 /* PROPS */, ["text", "match"]),
                _createVNode(_component_SettingsItem, {
                  command: "",
                  icon: "i-ri:information-line",
                  text: _ctx.$t('settings.about.label'),
                  to: "/settings/about",
                  match: _ctx.$route.path.startsWith('/settings/about/')
                }, null, 8 /* PROPS */, ["text", "match"])
              ])
            ]),
            _: 1 /* STABLE */
          }) ], 2 /* CLASS */), _createElementVNode("div", {
          "flex-1": "",
          class: _normalizeClass(isRootPath.value ? 'hidden lg:block' : 'block')
        }, [ _createVNode(_component_ClientOnly, null, {
            default: _withCtx(() => [
              _createVNode(_component_NuxtPage)
            ]),
            _: 1 /* STABLE */
          }) ], 2 /* CLASS */) ]) ]))
}
}

})
