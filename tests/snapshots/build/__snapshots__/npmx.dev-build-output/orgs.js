import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "font-mono text-2xl sm:text-3xl font-medium" }
const _hoisted_2 = { class: "text-fg-muted text-sm mt-1" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:arrow-left rtl-flip w-4 h-4", "aria-hidden": "true" })
const _hoisted_4 = { class: "text-fg-muted mb-4" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("code", { class: "font-mono bg-bg-subtle px-1.5 py-0.5 rounded" }, "npx @npmx.dev/cli")
const _hoisted_6 = { class: "text-fg-muted" }
const _hoisted_7 = { class: "text-fg-muted mb-4" }
const _hoisted_8 = { class: "text-fg-muted" }
const _hoisted_9 = { class: "text-fg-subtle text-sm mt-2" }
const _hoisted_10 = { class: "text-xs text-fg-subtle uppercase tracking-wider mb-4" }
const _hoisted_11 = { class: "text-2xl text-fg-subtle font-mono" }
const _hoisted_12 = { class: "font-mono text-lg text-fg truncate" }
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:box w-4 h-4", "aria-hidden": "true" })

interface OrgInfo {
  name: string
  role: 'developer' | 'admin' | 'owner' | null
  packageCount: number | null
  isLoadingDetails: boolean
}

export default /*@__PURE__*/_defineComponent({
  __name: 'orgs',
  setup(__props) {

const route = useRoute('~username-orgs')
const username = computed(() => route.params.username)
const { isConnected, npmUser, listUserOrgs, listOrgUsers } = useConnector()
// Only allow viewing your own orgs page
const isOwnProfile = computed(() => {
  return isConnected.value && npmUser.value?.toLowerCase() === username.value.toLowerCase()
})
const isLoading = shallowRef(true)
const orgs = shallowRef<OrgInfo[]>([])
const error = shallowRef<string | null>(null)
async function loadOrgDetails(org: OrgInfo) {
  org.isLoadingDetails = true
  // Fetch package count using our server API (proxies to npm registry)
  try {
    const response = await $fetch<{ count: number }>(
      `/api/registry/org/${encodeURIComponent(org.name)}/packages`,
      { timeout: 5000 },
    )
    org.packageCount = response.count
  } catch {
    org.packageCount = null
  }
  // Fetch user's role in this org
  try {
    const users = await listOrgUsers(org.name)
    if (users && npmUser.value) {
      const lowerUser = npmUser.value.toLowerCase()
      const entry = Object.entries(users).find(([k]) => k.toLowerCase() === lowerUser)
      org.role = entry?.[1] ?? null
    }
  } catch {
    org.role = null
  }
  org.isLoadingDetails = false
}
async function loadOrgs() {
  if (!isOwnProfile.value) {
    isLoading.value = false
    return
  }
  isLoading.value = true
  error.value = null
  try {
    const orgList = await listUserOrgs()
    if (orgList) {
      orgs.value = orgList.map(name => ({
        name,
        role: null,
        packageCount: null,
        isLoadingDetails: true,
      }))
      // Load details for each org in parallel
      await Promise.all(orgs.value.map(org => loadOrgDetails(org)))
    } else {
      error.value = $t('header.orgs_dropdown.error')
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : $t('header.orgs_dropdown.error')
  } finally {
    isLoading.value = false
  }
}
error.value = $t('header.orgs_dropdown.error')
// Load on mount and when connection status changes
watch(isOwnProfile, loadOrgs, { immediate: true })
function getRoleBadgeClass(role: string | null): string {
  switch (role) {
    case 'owner':
      return 'bg-purple-500/20 text-purple-300'
    case 'admin':
      return 'bg-blue-500/20 text-blue-300'
    case 'developer':
      return 'bg-green-500/20 text-green-300'
    default:
      return 'bg-fg-subtle/20 text-fg-muted'
  }
}
useSeoMeta({
  title: () => `@${username.value} Organizations - npmx`,
  ogTitle: () => `@${username.value} Organizations - npmx`,
  twitterTitle: () => `@${username.value} Organizations - npmx`,
  description: () => `npm organizations for ${username.value}`,
  ogDescription: () => `npm organizations for ${username.value}`,
  twitterDescription: () => `npm organizations for ${username.value}`,
})
defineOgImageComponent('Default', {
  title: () => `@${username.value}`,
  description: () => {
    if (isLoading.value) return 'npm organizations'
    if (orgs.value.length === 0) return 'No organizations found'
    const count = orgs.value.length
    return `${count} ${count === 1 ? 'organization' : 'organizations'}`
  },
  primaryColor: '#60a5fa',
})

return (_ctx: any,_cache: any) => {
  const _component_UserAvatar = _resolveComponent("UserAvatar")
  const _component_NuxtLink = _resolveComponent("NuxtLink")
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_LoadingSpinner = _resolveComponent("LoadingSpinner")
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_SkeletonInline = _resolveComponent("SkeletonInline")
  const _component_ClientOnly = _resolveComponent("ClientOnly")

  return (_openBlock(), _createElementBlock("main", { class: "container flex-1 py-8 sm:py-12 w-full" }, [ _createElementVNode("header", { class: "mb-8 pb-8 border-b border-border" }, [ _createElementVNode("div", { class: "flex flex-wrap items-center gap-4 mb-4" }, [ _createVNode(_component_UserAvatar, { username: username.value }, null, 8 /* PROPS */, ["username"]), _createElementVNode("div", null, [ _createElementVNode("h1", _hoisted_1, "~" + _toDisplayString(username.value), 1 /* TEXT */), _createElementVNode("p", _hoisted_2, _toDisplayString(_ctx.$t('user.orgs_page.title')), 1 /* TEXT */) ]) ]), _createTextVNode("\n\n      " + "\n      "), _createElementVNode("nav", { "aria-labelledby": "back-to-profile" }, [ _createVNode(_component_NuxtLink, {
            to: { name: '~username', params: { username: username.value } },
            id: "back-to-profile",
            class: "link-subtle font-mono text-sm inline-flex items-center gap-1.5"
          }, {
            default: _withCtx(() => [
              _hoisted_3,
              _createTextVNode("\n          "),
              _createTextVNode(_toDisplayString(_ctx.$t('user.orgs_page.back_to_profile')), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["to"]) ]) ]), _createTextVNode("\n\n    " + "\n    "), _createVNode(_component_ClientOnly, null, {
        default: _withCtx(() => [
          (!_unref(isConnected))
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "py-12 text-center"
            }, [
              _createElementVNode("p", _hoisted_4, _toDisplayString(_ctx.$t('user.orgs_page.connect_required')), 1 /* TEXT */),
              _createElementVNode("p", { class: "text-fg-subtle text-sm" }, [
                _createTextVNode(_toDisplayString(_ctx.$t('user.orgs_page.connect_hint_prefix')) + "\n          ", 1 /* TEXT */),
                _hoisted_5,
                _createTextVNode("\n          " + _toDisplayString(_ctx.$t('user.orgs_page.connect_hint_suffix')), 1 /* TEXT */)
              ])
            ]))
            : (!isOwnProfile.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                class: "py-12 text-center"
              }, [
                _createElementVNode("p", _hoisted_6, _toDisplayString(_ctx.$t('user.orgs_page.own_orgs_only')), 1 /* TEXT */),
                _createVNode(_component_LinkBase, {
                  variant: "button-secondary",
                  to: {
  	name: "~username-orgs",
  	params: { username: _unref(npmUser) }
  },
                  class: "mt-4"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_ctx.$t('user.orgs_page.view_your_orgs')), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["to"])
              ]))
            : (isLoading.value)
              ? (_openBlock(), _createBlock(_component_LoadingSpinner, {
                key: 2,
                text: _ctx.$t('user.orgs_page.loading')
              }, null, 8 /* PROPS */, ["text"]))
            : (error.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 3,
                role: "alert",
                class: "py-12 text-center"
              }, [
                _createElementVNode("p", _hoisted_7, _toDisplayString(error.value), 1 /* TEXT */),
                _createVNode(_component_ButtonBase, { onClick: loadOrgs }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_ctx.$t('common.try_again')), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                })
              ]))
            : (orgs.value.length === 0)
              ? (_openBlock(), _createElementBlock("div", {
                key: 4,
                class: "py-12 text-center"
              }, [
                _createElementVNode("p", _hoisted_8, _toDisplayString(_ctx.$t('user.orgs_page.empty')), 1 /* TEXT */),
                _createElementVNode("p", _hoisted_9, _toDisplayString(_ctx.$t('user.orgs_page.empty_hint')), 1 /* TEXT */)
              ]))
            : (_openBlock(), _createElementBlock("section", {
              key: 5,
              "aria-label": _ctx.$t('user.orgs_page.title')
            }, [
              _createElementVNode("h2", _hoisted_10, _toDisplayString(_ctx.$t('user.orgs_page.count', { count: orgs.value.length }, orgs.value.length)), 1 /* TEXT */),
              _createElementVNode("ul", { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-3" }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(orgs.value, (org) => {
                  return (_openBlock(), _createElementBlock("li", { key: org.name }, [
                    _createVNode(_component_NuxtLink, {
                      to: { name: 'org', params: { org: org.name } },
                      class: "block p-5 bg-bg-subtle border border-border rounded-lg hover:border-fg-subtle transition-colors h-full"
                    }, {
                      default: _withCtx(() => [
                        _createElementVNode("div", { class: "flex items-start gap-4 mb-4" }, [
                          _createElementVNode("div", {
                            class: "w-14 h-14 rounded-lg bg-bg-muted border border-border flex items-center justify-center flex-shrink-0",
                            "aria-hidden": "true"
                          }, [
                            _createElementVNode("span", _hoisted_11, _toDisplayString(org.name.charAt(0).toUpperCase()), 1 /* TEXT */)
                          ]),
                          _createElementVNode("div", { class: "min-w-0 flex-1" }, [
                            _createElementVNode("h3", _hoisted_12, "@" + _toDisplayString(org.name), 1 /* TEXT */),
                            _createTextVNode("\n                  " + "\n                  "),
                            (org.role)
                              ? (_openBlock(), _createElementBlock("span", {
                                key: 0,
                                class: _normalizeClass(["inline-block mt-1 px-2 py-0.5 text-xs font-mono rounded", getRoleBadgeClass(org.role)])
                              }, _toDisplayString(org.role), 1 /* TEXT */))
                              : (org.isLoadingDetails)
                                ? (_openBlock(), _createBlock(_component_SkeletonInline, {
                                  key: 1,
                                  class: "mt-1 h-5 w-16 rounded"
                                }))
                              : _createCommentVNode("v-if", true)
                          ])
                        ]),
                        _createTextVNode("\n\n              "),
                        _createTextVNode("\n              "),
                        _createElementVNode("div", { class: "flex items-center gap-4 text-sm text-fg-muted" }, [
                          _createElementVNode("div", { class: "flex items-center gap-1.5" }, [
                            _hoisted_13,
                            (org.packageCount !== null)
                              ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(_ctx.$t(
                          'user.orgs_page.packages_count',
                          { count: org.packageCount },
                          org.packageCount,
                        )), 1 /* TEXT */))
                              : (org.isLoadingDetails)
                                ? (_openBlock(), _createBlock(_component_SkeletonInline, {
                                  key: 1,
                                  class: "h-4 w-20"
                                }))
                              : (_openBlock(), _createElementBlock("span", {
                                key: 2,
                                class: "text-fg-subtle"
                              }, "—"))
                          ])
                        ])
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to"])
                  ]))
                }), 128 /* KEYED_FRAGMENT */))
              ])
            ])),
          _createTextVNode("\n\n      "),
          _createTextVNode("\n      "),
          _createTextVNode("\n\n      "),
          _createTextVNode("\n      "),
          _createTextVNode("\n\n      "),
          _createTextVNode("\n      "),
          _createTextVNode("\n\n      "),
          _createTextVNode("\n      "),
          _createTextVNode("\n\n      "),
          _createTextVNode("\n      ")
        ]),
        _: 1 /* STABLE */
      }) ]))
}
}

})
