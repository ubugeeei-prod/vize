import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = { id: "access-heading", class: "text-xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-4 h-4 text-fg-muted animate-spin mx-auto", "aria-hidden": "true" })
const _hoisted_3 = { class: "font-mono text-sm text-fg-muted truncate" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-4 h-4", "aria-hidden": "true" })
import type { NewOperation } from '~/composables/useConnector'
import { buildScopeTeam } from '~/utils/npm/common'

export default /*@__PURE__*/_defineComponent({
  __name: 'AccessControls',
  props: {
    packageName: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const {
  isConnected,
  lastExecutionTime,
  listOrgTeams,
  listPackageCollaborators,
  addOperation,
  error: connectorError,
} = useConnector()
// Extract org name from scoped package (e.g., "@nuxt/kit" -> "nuxt")
const orgName = computed(() => {
  if (!props.packageName.startsWith('@')) return null
  const match = props.packageName.match(/^@([^/]+)\//)
  return match ? match[1] : null
})
// Data
const collaborators = shallowRef<Record<string, 'read-only' | 'read-write'>>({})
const teams = shallowRef<string[]>([])
const isLoadingCollaborators = shallowRef(false)
const isLoadingTeams = shallowRef(false)
const error = shallowRef<string | null>(null)
// Grant access form
const showGrantAccess = shallowRef(false)
const selectedTeam = shallowRef('')
const permission = shallowRef<'read-only' | 'read-write'>('read-only')
const isGranting = shallowRef(false)
// Computed collaborator list with type detection
const collaboratorList = computed(() => {
  return Object.entries(collaborators.value)
    .map(([name, perm]) => {
      // Check if this looks like a team (org:team format) or user
      const isTeam = name.includes(':')
      return {
        name,
        permission: perm,
        isTeam,
        displayName: isTeam ? name.split(':')[1] : name,
      }
    })
    .sort((a, b) => {
      // Teams first, then users
      if (a.isTeam !== b.isTeam) return a.isTeam ? -1 : 1
      return a.name.localeCompare(b.name)
    })
})
// Load collaborators
async function loadCollaborators() {
  if (!isConnected.value) return
  isLoadingCollaborators.value = true
  error.value = null
  try {
    const result = await listPackageCollaborators(props.packageName)
    if (result) {
      collaborators.value = result
    } else {
      error.value = connectorError.value || 'Failed to load collaborators'
    }
  } finally {
    isLoadingCollaborators.value = false
  }
}
// Load teams for dropdown
async function loadTeams() {
  if (!isConnected.value || !orgName.value) return
  isLoadingTeams.value = true
  try {
    const result = await listOrgTeams(orgName.value)
    if (result) {
      // Teams come as "org:team" format, extract just the team name
      teams.value = result.map((t: string) => t.replace(`${orgName.value}:`, ''))
    }
  } finally {
    isLoadingTeams.value = false
  }
}
// Grant access
async function handleGrantAccess() {
  if (!selectedTeam.value || !orgName.value) return
  isGranting.value = true
  try {
    const scopeTeam = buildScopeTeam(orgName.value, selectedTeam.value)
    const operation: NewOperation = {
      type: 'access:grant',
      params: {
        permission: permission.value,
        scopeTeam,
        pkg: props.packageName,
      },
      description: `Grant ${permission.value} access to ${scopeTeam} for ${props.packageName}`,
      command: `npm access grant ${permission.value} ${scopeTeam} ${props.packageName}`,
    }
    await addOperation(operation)
    selectedTeam.value = ''
    showGrantAccess.value = false
  } finally {
    isGranting.value = false
  }
}
// Revoke access
async function handleRevokeAccess(collaboratorName: string) {
  // For teams, we use the full org:team format
  // For users... actually npm access revoke only works for teams
  // Users get access via maintainers/owners which is managed separately
  const operation: NewOperation = {
    type: 'access:revoke',
    params: {
      scopeTeam: collaboratorName,
      pkg: props.packageName,
    },
    description: `Revoke ${collaboratorName} access to ${props.packageName}`,
    command: `npm access revoke ${collaboratorName} ${props.packageName}`,
  }
  await addOperation(operation)
}
// Reload when package changes
watch(
  () => [isConnected.value, props.packageName, lastExecutionTime.value],
  ([connected]) => {
    if (connected && orgName.value) {
      loadCollaborators()
      loadTeams()
    }
  },
  { immediate: true },
)

return (_ctx: any,_cache: any) => {
  const _component_SelectField = _resolveComponent("SelectField")

  return (_unref(isConnected) && orgName.value)
      ? (_openBlock(), _createElementBlock("section", { key: 0 }, [ _createElementVNode("div", { class: "flex items-center justify-between mb-3" }, [ _createElementVNode("h2", _hoisted_1, _toDisplayString(_ctx.$t('package.access.title')), 1 /* TEXT */), _createElementVNode("button", {
            type: "button",
            class: "p-1 text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
            "aria-label": _ctx.$t('package.access.refresh'),
            disabled: isLoadingCollaborators.value,
            onClick: loadCollaborators
          }, [ _createElementVNode("span", {
              class: _normalizeClass(["i-lucide:refresh-ccw w-3.5 h-3.5", { 'motion-safe:animate-spin': isLoadingCollaborators.value }]),
              "aria-hidden": "true"
            }, null, 2 /* CLASS */) ], 8 /* PROPS */, ["aria-label", "disabled"]) ]), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (isLoadingCollaborators.value && collaboratorList.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "py-4 text-center"
          }, [ _hoisted_2 ])) : (error.value) ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "text-xs text-red-400 mb-2",
              role: "alert"
            }, _toDisplayString(error.value), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (collaboratorList.value.length > 0) ? (_openBlock(), _createElementBlock("ul", {
            key: 0,
            class: "space-y-1 mb-3",
            "aria-label": _ctx.$t('package.access.list_label')
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(collaboratorList.value, (collab) => {
              return (_openBlock(), _createElementBlock("li", {
                key: collab.name,
                class: "flex items-center justify-between py-1"
              }, [
                _createElementVNode("div", { class: "flex items-center gap-2 min-w-0" }, [
                  (collab.isTeam)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: "i-lucide:users w-3.5 h-3.5 text-fg-subtle shrink-0",
                      "aria-hidden": "true"
                    }))
                    : (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      class: "i-lucide:user w-3.5 h-3.5 text-fg-subtle shrink-0",
                      "aria-hidden": "true"
                    })),
                  _createElementVNode("span", _hoisted_3, _toDisplayString(collab.isTeam ? collab.displayName : `@${collab.name}`), 1 /* TEXT */),
                  _createElementVNode("span", {
                    class: _normalizeClass(["px-1 py-0.5 font-mono text-xs rounded shrink-0", 
                collab.permission === 'read-write'
                  ? 'bg-green-500/20 text-green-400'
                  : 'bg-fg-subtle/20 text-fg-muted'
              ])
                  }, _toDisplayString(collab.permission === 'read-write' ? _ctx.$t('package.access.rw') : _ctx.$t('package.access.ro')), 3 /* TEXT, CLASS */)
                ]),
                _createTextVNode("\n        " + "\n        "),
                (collab.isTeam)
                  ? (_openBlock(), _createElementBlock("button", {
                    key: 0,
                    type: "button",
                    class: "p-1 text-fg-subtle hover:text-red-400 transition-colors duration-200 shrink-0 rounded focus-visible:outline-accent/70",
                    "aria-label": _ctx.$t('package.access.revoke_access', { name: collab.displayName }),
                    onClick: _cache[0] || (_cache[0] = ($event: any) => (handleRevokeAccess(collab.name)))
                  }, [
                    _hoisted_4
                  ]))
                  : (_openBlock(), _createElementBlock("span", {
                    key: 1,
                    class: "text-xs text-fg-subtle"
                  }, _toDisplayString(_ctx.$t('package.access.owner')), 1 /* TEXT */))
              ]))
            }), 128 /* KEYED_FRAGMENT */)) ])) : (!isLoadingCollaborators.value && !error.value) ? (_openBlock(), _createElementBlock("p", {
              key: 1,
              class: "text-xs text-fg-subtle mb-3"
            }, _toDisplayString(_ctx.$t('package.access.no_access')), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (showGrantAccess.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("form", {
              class: "space-y-2",
              onSubmit: _withModifiers(handleGrantAccess, ["prevent"])
            }, [ _createElementVNode("div", { class: "flex items-center gap-2" }, [ _createVNode(_component_SelectField, {
                  label: _ctx.$t('package.access.select_team_label'),
                  "hidden-label": "",
                  id: "grant-team-select",
                  name: "grant-team",
                  block: "",
                  size: "sm",
                  disabled: isLoadingTeams.value,
                  items: [{
  	label: isLoadingTeams.value ? _ctx.$t("package.access.loading_teams") : _ctx.$t("package.access.select_team"),
  	value: "",
  	disabled: true
  }, ...teams.value.map((team) => ({
  	label: `${orgName.value}:${team}`,
  	value: team
  }))],
                  modelValue: selectedTeam.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((selectedTeam).value = $event))
                }, null, 8 /* PROPS */, ["label", "disabled", "items", "modelValue"]), _createVNode(_component_SelectField, {
                  label: _ctx.$t('package.access.permission_label'),
                  "hidden-label": "",
                  id: "grant-permission-select",
                  name: "grant-permission",
                  block: "",
                  size: "sm",
                  items: [
                { label: _ctx.$t('package.access.permission.read_only'), value: 'read-only' },
                { label: _ctx.$t('package.access.permission.read_write'), value: 'read-write' },
              ],
                  modelValue: permission.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((permission).value = $event))
                }, null, 8 /* PROPS */, ["label", "items", "modelValue"]), _createElementVNode("button", {
                  type: "submit",
                  disabled: !selectedTeam.value || isGranting.value,
                  class: "px-3 py-2 font-mono text-xs text-bg bg-fg rounded transition-all duration-200 hover:bg-fg/90 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-accent/70"
                }, _toDisplayString(isGranting.value ? '…' : _ctx.$t('package.access.grant_button')), 9 /* TEXT, PROPS */, ["disabled"]), _createElementVNode("button", {
                  type: "button",
                  class: "p-1.5 text-fg-subtle hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                  "aria-label": _ctx.$t('package.access.cancel_grant'),
                  onClick: _cache[3] || (_cache[3] = ($event: any) => (showGrantAccess.value = false))
                }, [ _hoisted_5 ], 8 /* PROPS */, ["aria-label"]) ]) ], 32 /* NEED_HYDRATION */) ])) : (_openBlock(), _createElementBlock("button", {
            key: 1,
            type: "button",
            class: "w-full px-3 py-1.5 font-mono text-xs text-fg-muted bg-bg-subtle border border-border rounded transition-colors duration-200 hover:text-fg hover:border-border-hover focus-visible:outline-accent/70",
            onClick: _cache[4] || (_cache[4] = ($event: any) => (showGrantAccess.value = true))
          }, _toDisplayString(_ctx.$t('package.access.grant_access')), 1 /* TEXT */)) ]))
      : _createCommentVNode("v-if", true)
}
}

})
