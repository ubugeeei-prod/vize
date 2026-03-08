import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:users w-4 h-4 text-fg-muted", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "flex-shrink-1 flex-grow-1" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("span", { class: "absolute inset-is-2 top-1/2 -translate-y-1/2 i-lucide:search w-3.5 h-3.5 text-fg-subtle", "aria-hidden": "true" })
const _hoisted_4 = { for: "teams-search", class: "sr-only" }
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-5 h-5 text-fg-muted mx-auto", "aria-hidden": "true" })
const _hoisted_6 = { class: "font-mono text-sm text-fg-muted mt-2" }
const _hoisted_7 = { class: "font-mono text-sm text-red-400" }
const _hoisted_8 = { class: "font-mono text-sm text-fg-muted" }
const _hoisted_9 = { class: "font-mono text-sm text-fg" }
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("span", { "aria-hidden": "true", class: "flex-shrink-1 flex-grow-1" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:trash w-4 h-4", "aria-hidden": "true" })
const _hoisted_12 = { class: "font-mono text-sm text-fg mx-2" }
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-4 h-4", "aria-hidden": "true" })
const _hoisted_15 = { class: "font-mono text-sm text-fg-muted" }
const _hoisted_16 = { class: "px-2 py-3 leading-none font-mono text-sm text-fg-subtle bg-bg border border-ie-0 border-border rounded-is" }
const _hoisted_17 = { for: "new-team-name", class: "sr-only" }
const _hoisted_18 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-4 h-4", "aria-hidden": "true" })
import type { NewOperation } from '~/composables/useConnector'
import { buildScopeTeam } from '~/utils/npm/common'

export default /*@__PURE__*/_defineComponent({
  __name: 'TeamsPanel',
  props: {
    orgName: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const {
  isConnected,
  lastExecutionTime,
  listOrgTeams,
  listOrgUsers,
  listTeamUsers,
  addOperation,
  error: connectorError,
} = useConnector()
// Teams data
const teams = shallowRef<string[]>([])
const teamUsers = ref<Record<string, string[]>>({})
const isLoadingTeams = shallowRef(false)
const isLoadingUsers = ref<Record<string, boolean>>({})
const error = shallowRef<string | null>(null)
// Org members (to check if user needs to be added to org first)
const orgMembers = shallowRef<Record<string, 'developer' | 'admin' | 'owner'>>({})
// Search/filter
const searchQuery = shallowRef('')
const sortBy = shallowRef<'name' | 'members'>('name')
const sortOrder = shallowRef<'asc' | 'desc'>('asc')
// Expanded teams (to show members)
const expandedTeams = ref<Set<string>>(new Set())
// Create team form
const showCreateTeam = shallowRef(false)
const newTeamName = shallowRef('')
const isCreatingTeam = shallowRef(false)
// Add user form (per team)
const showAddUserFor = shallowRef<string | null>(null)
const newUserUsername = shallowRef('')
const isAddingUser = shallowRef(false)
// Filtered and sorted teams
const filteredTeams = computed(() => {
  let result = teams.value

  // Filter by search
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(team => team.toLowerCase().includes(query))
  }

  // Sort
  result = [...result].sort((a, b) => {
    if (sortBy.value === 'name') {
      return sortOrder.value === 'asc' ? a.localeCompare(b) : b.localeCompare(a)
    } else {
      const aCount = teamUsers.value[a]?.length ?? 0
      const bCount = teamUsers.value[b]?.length ?? 0
      return sortOrder.value === 'asc' ? aCount - bCount : bCount - aCount
    }
  })

  return result
})
// Load teams and org members
async function loadTeams() {
  if (!isConnected.value) return
  isLoadingTeams.value = true
  error.value = null
  try {
    // Load teams and org members in parallel
    const [teamsResult, membersResult] = await Promise.all([
      listOrgTeams(props.orgName),
      listOrgUsers(props.orgName),
    ])
    if (teamsResult) {
      // Teams come as "org:team" format, extract just the team name
      teams.value = teamsResult.map((t: string) => t.replace(`${props.orgName}:`, ''))
    } else {
      error.value = connectorError.value || 'Failed to load teams'
    }
    if (membersResult) {
      orgMembers.value = membersResult
    }
  } finally {
    isLoadingTeams.value = false
  }
}
// Load team members
async function loadTeamUsers(teamName: string) {
  if (!isConnected.value) return
  isLoadingUsers.value[teamName] = true
  try {
    const scopeTeam = buildScopeTeam(props.orgName, teamName)
    const result = await listTeamUsers(scopeTeam)
    if (result) {
      teamUsers.value[teamName] = result
    }
  } finally {
    isLoadingUsers.value[teamName] = false
  }
}
// Toggle team expansion
async function toggleTeam(teamName: string) {
  if (expandedTeams.value.has(teamName)) {
    expandedTeams.value.delete(teamName)
  } else {
    expandedTeams.value.add(teamName)
    // Load users if not already loaded
    if (!teamUsers.value[teamName]) {
      await loadTeamUsers(teamName)
    }
  }
  // Force reactivity
  expandedTeams.value = new Set(expandedTeams.value)
}
// Create team
async function handleCreateTeam() {
  if (!newTeamName.value.trim()) return
  isCreatingTeam.value = true
  try {
    const teamName = newTeamName.value.trim()
    const scopeTeam = buildScopeTeam(props.orgName, teamName)
    const operation: NewOperation = {
      type: 'team:create',
      params: { scopeTeam },
      description: `Create team ${scopeTeam}`,
      command: `npm team create ${scopeTeam}`,
    }
    await addOperation(operation)
    newTeamName.value = ''
    showCreateTeam.value = false
  } finally {
    isCreatingTeam.value = false
  }
}
// Destroy team
async function handleDestroyTeam(teamName: string) {
  const scopeTeam = buildScopeTeam(props.orgName, teamName)
  const operation: NewOperation = {
    type: 'team:destroy',
    params: { scopeTeam },
    description: `Destroy team ${scopeTeam}`,
    command: `npm team destroy ${scopeTeam}`,
  }
  await addOperation(operation)
}
// Add user to team (auto-invites to org if needed)
async function handleAddUser(teamName: string) {
  if (!newUserUsername.value.trim()) return
  isAddingUser.value = true
  try {
    const username = newUserUsername.value.trim().replace(/^@/, '')
    const scopeTeam = buildScopeTeam(props.orgName, teamName)
    let dependsOnId: string | undefined
    // If user is not in org, add them first with developer role
    const isInOrg = username in orgMembers.value
    if (!isInOrg) {
      const orgOperation: NewOperation = {
        type: 'org:add-user',
        params: {
          org: props.orgName,
          user: username,
          role: 'developer',
        },
        description: `Add @${username} to @${props.orgName} as developer`,
        command: `npm org set ${props.orgName} ${username} developer`,
      }
      const addedOp = await addOperation(orgOperation)
      if (addedOp) {
        dependsOnId = addedOp.id
      }
    }
    // Then add user to team (depends on org op if user wasn't in org)
    const teamOperation: NewOperation = {
      type: 'team:add-user',
      params: { scopeTeam, user: username },
      description: `Add @${username} to team ${teamName}`,
      command: `npm team add ${scopeTeam} ${username}`,
      dependsOn: dependsOnId,
    }
    await addOperation(teamOperation)
    newUserUsername.value = ''
    showAddUserFor.value = null
  } finally {
    isAddingUser.value = false
  }
}
// Remove user from team
async function handleRemoveUser(teamName: string, username: string) {
  const scopeTeam = buildScopeTeam(props.orgName, teamName)
  const operation: NewOperation = {
    type: 'team:rm-user',
    params: { scopeTeam, user: username },
    description: `Remove @${username} from ${scopeTeam}`,
    command: `npm team rm ${scopeTeam} ${username}`,
  }
  await addOperation(operation)
}
// Toggle sort
function toggleSort(field: 'name' | 'members') {
  if (sortBy.value === field) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = field
    sortOrder.value = 'asc'
  }
}
// Load on mount when connected
watch(
  isConnected,
  connected => {
    if (connected) {
      loadTeams()
    }
  },
  { immediate: true },
)
// Refresh data when operations complete
watch(lastExecutionTime, () => {
  if (isConnected.value) {
    loadTeams()
  }
})

return (_ctx: any,_cache: any) => {
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_unref(isConnected))
      ? (_openBlock(), _createElementBlock("section", {
        key: 0,
        class: "bg-bg-subtle border border-border rounded-lg overflow-hidden"
      }, [ _createElementVNode("div", { class: "flex items-center justify-start p-4 border-b border-border" }, [ _createElementVNode("h2", {
            id: "teams-heading",
            class: "font-mono text-sm font-medium flex items-center gap-2"
          }, [ _hoisted_1, _createTextVNode("\n        " + _toDisplayString(_ctx.$t('org.teams.title')) + "\n        ", 1 /* TEXT */), (teams.value.length > 0) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "text-fg-muted"
              }, "(" + _toDisplayString(teams.value.length) + ")", 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _hoisted_2, _createElementVNode("button", {
            type: "button",
            class: "p-1.5 text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
            "aria-label": _ctx.$t('org.teams.refresh'),
            disabled: isLoadingTeams.value,
            onClick: loadTeams
          }, [ _createElementVNode("span", {
              class: _normalizeClass(["i-lucide:refresh-ccw w-4 h-4", { 'animate-spin': isLoadingTeams.value }]),
              "aria-hidden": "true"
            }, null, 2 /* CLASS */) ], 8 /* PROPS */, ["aria-label", "disabled"]) ]), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createElementVNode("div", { class: "flex items-center gap-2 p-3 border-b border-border bg-bg" }, [ _createElementVNode("div", { class: "flex-1 relative" }, [ _hoisted_3, _createElementVNode("label", _hoisted_4, _toDisplayString(_ctx.$t('org.teams.filter_label')), 1 /* TEXT */), _createVNode(_component_InputBase, {
              id: "teams-search",
              type: "search",
              name: "teams-search",
              placeholder: _ctx.$t('org.teams.filter_placeholder'),
              "no-correct": "",
              class: "w-full min-w-25 ps-7",
              size: "medium",
              modelValue: searchQuery.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((searchQuery).value = $event))
            }, null, 8 /* PROPS */, ["placeholder", "modelValue"]) ]), _createElementVNode("div", {
            class: "flex items-center gap-1 text-xs",
            role: "group",
            "aria-label": _ctx.$t('org.teams.sort_by')
          }, [ _createElementVNode("button", {
              type: "button",
              class: _normalizeClass(["px-2 py-1 font-mono rounded transition-colors duration-200 focus-visible:outline-accent/70", sortBy.value === 'name' ? 'bg-bg-muted text-fg' : 'text-fg-muted hover:text-fg']),
              "aria-pressed": sortBy.value === 'name',
              onClick: _cache[1] || (_cache[1] = ($event: any) => (toggleSort('name')))
            }, [ _createTextVNode(_toDisplayString(_ctx.$t('common.sort.name')) + "\n          ", 1 /* TEXT */), (sortBy.value === 'name') ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(sortOrder.value === 'asc' ? '↑' : '↓'), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 10 /* CLASS, PROPS */, ["aria-pressed"]), _createElementVNode("button", {
              type: "button",
              class: _normalizeClass(["px-2 py-1 font-mono rounded transition-colors duration-200 focus-visible:outline-accent/70", sortBy.value === 'members' ? 'bg-bg-muted text-fg' : 'text-fg-muted hover:text-fg']),
              "aria-pressed": sortBy.value === 'members',
              onClick: _cache[2] || (_cache[2] = ($event: any) => (toggleSort('members')))
            }, [ _createTextVNode(_toDisplayString(_ctx.$t('common.sort.members')) + "\n          ", 1 /* TEXT */), (sortBy.value === 'members') ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(sortOrder.value === 'asc' ? '↑' : '↓'), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 10 /* CLASS, PROPS */, ["aria-pressed"]) ], 8 /* PROPS */, ["aria-label"]) ]), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (isLoadingTeams.value && teams.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "p-8 text-center"
          }, [ _hoisted_5, _createElementVNode("p", _hoisted_6, _toDisplayString(_ctx.$t('org.teams.loading')), 1 /* TEXT */) ])) : (error.value) ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "p-4 text-center",
              role: "alert"
            }, [ _createElementVNode("p", _hoisted_7, _toDisplayString(error.value), 1 /* TEXT */), _createElementVNode("button", {
                type: "button",
                class: "mt-2 font-mono text-xs text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                onClick: loadTeams
              }, _toDisplayString(_ctx.$t('common.try_again')), 1 /* TEXT */) ])) : (teams.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "p-8 text-center"
            }, [ _createElementVNode("p", _hoisted_8, _toDisplayString(_ctx.$t('org.teams.no_teams')), 1 /* TEXT */) ])) : (_openBlock(), _createElementBlock("ul", {
            key: 3,
            class: "divide-y divide-border",
            "aria-label": _ctx.$t('org.teams.list_label')
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(filteredTeams.value, (teamName) => {
              return (_openBlock(), _createElementBlock("li", {
                key: teamName,
                class: "bg-bg"
              }, [
                _createElementVNode("div", { class: "flex items-center justify-start p-3 hover:bg-bg-subtle transition-colors duration-200" }, [
                  _createElementVNode("button", {
                    type: "button",
                    class: "flex-1 flex items-center gap-2 text-start rounded focus-visible:outline-accent/70",
                    "aria-expanded": expandedTeams.value.has(teamName),
                    "aria-controls": `team-${teamName}-members`,
                    onClick: _cache[3] || (_cache[3] = ($event: any) => (toggleTeam(teamName)))
                  }, [
                    _createElementVNode("span", {
                      class: _normalizeClass(["w-4 h-4 transition-transform duration-200 rtl-flip", [
                  expandedTeams.value.has(teamName) ? 'i-lucide:chevron-down' : 'i-lucide:chevron-right',
                  'text-fg-muted',
                ]]),
                      "aria-hidden": "true"
                    }, null, 2 /* CLASS */),
                    _createElementVNode("span", _hoisted_9, _toDisplayString(teamName), 1 /* TEXT */),
                    (teamUsers.value[teamName])
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "font-mono text-xs text-fg-subtle"
                      }, "\n              (" + _toDisplayString(_ctx.$t(
                    'org.teams.member_count',
                    { count: teamUsers.value[teamName].length },
                    teamUsers.value[teamName].length,
                  )) + ")\n            ", 1 /* TEXT */))
                      : _createCommentVNode("v-if", true),
                    (isLoadingUsers.value[teamName])
                      ? (_openBlock(), _createElementBlock("span", {
                        key: 0,
                        class: "i-svg-spinners:ring-resize w-3 h-3 text-fg-muted",
                        "aria-hidden": "true"
                      }))
                      : _createCommentVNode("v-if", true)
                  ], 8 /* PROPS */, ["aria-expanded", "aria-controls"]),
                  _hoisted_10,
                  _createElementVNode("button", {
                    type: "button",
                    class: "p-1 text-fg-subtle hover:text-red-400 transition-colors duration-200 rounded focus-visible:outline-accent/70",
                    "aria-label": _ctx.$t('org.teams.delete_team', { name: teamName }),
                    onClick: _cache[4] || (_cache[4] = _withModifiers(($event: any) => (handleDestroyTeam(teamName)), ["stop"]))
                  }, [
                    _hoisted_11
                  ], 8 /* PROPS */, ["aria-label"])
                ]),
                _createTextVNode("\n\n        " + "\n        "),
                (expandedTeams.value.has(teamName))
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    id: `team-${teamName}-members`,
                    class: "ps-9 pe-3 pb-3"
                  }, [
                    (teamUsers.value[teamName]?.length)
                      ? (_openBlock(), _createElementBlock("ul", {
                        key: 0,
                        class: "space-y-1 mb-2",
                        "aria-label": _ctx.$t('org.teams.members_of', { team: teamName })
                      }, [
                        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(teamUsers.value[teamName], (user) => {
                          return (_openBlock(), _createElementBlock("li", {
                            key: user,
                            class: "flex items-center justify-start py-1 ps-2 pe-1 rounded hover:bg-bg-subtle transition-colors duration-200"
                          }, [
                            _createVNode(_component_NuxtLink, {
                              to: { name: '~username', params: { username: user } },
                              class: "font-mono text-sm text-fg-muted hover:text-fg transition-colors duration-200"
                            }, {
                              default: _withCtx(() => [
                                _createTextVNode("\n                ~"),
                                _createTextVNode(_toDisplayString(user), 1 /* TEXT */)
                              ]),
                              _: 2 /* DYNAMIC */
                            }, 8 /* PROPS */, ["to"]),
                            _createElementVNode("span", _hoisted_12, _toDisplayString(teamName), 1 /* TEXT */),
                            _createElementVNode("button", {
                              type: "button",
                              class: "p-1 text-fg-subtle hover:text-red-400 transition-colors duration-200 rounded focus-visible:outline-accent/70",
                              "aria-label": _ctx.$t('org.teams.remove_user', { user }),
                              onClick: _cache[5] || (_cache[5] = ($event: any) => (handleRemoveUser(teamName, user)))
                            }, [
                              _hoisted_13
                            ], 8 /* PROPS */, ["aria-label"])
                          ]))
                        }), 128 /* KEYED_FRAGMENT */))
                      ]))
                      : (!isLoadingUsers.value[teamName])
                        ? (_openBlock(), _createElementBlock("p", {
                          key: 1,
                          class: "font-mono text-xs text-fg-subtle py-1"
                        }, _toDisplayString(_ctx.$t('org.teams.no_members')), 1 /* TEXT */))
                      : _createCommentVNode("v-if", true),
                    _createTextVNode("\n\n          "),
                    _createTextVNode("\n          "),
                    (showAddUserFor.value === teamName)
                      ? (_openBlock(), _createElementBlock("div", {
                        key: 0,
                        class: "mt-2"
                      }, [
                        _createElementVNode("form", {
                          class: "flex items-center gap-2",
                          onSubmit: _cache[6] || (_cache[6] = _withModifiers(($event: any) => (handleAddUser(teamName)), ["prevent"]))
                        }, [
                          _createElementVNode("label", {
                            for: `add-user-${teamName}`,
                            class: "sr-only"
                          }, _toDisplayString(_ctx.$t('org.teams.username_to_add', { team: teamName })), 9 /* TEXT, PROPS */, ["for"]),
                          _createVNode(_component_InputBase, {
                            id: `add-user-${teamName}`,
                            type: "text",
                            name: `add-user-${teamName}`,
                            placeholder: _ctx.$t('org.teams.username_placeholder'),
                            "no-correct": "",
                            class: "flex-1 min-w-25",
                            size: "medium",
                            modelValue: newUserUsername.value,
                            "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((newUserUsername).value = $event))
                          }, null, 8 /* PROPS */, ["id", "name", "placeholder", "modelValue"]),
                          _createElementVNode("button", {
                            type: "submit",
                            disabled: !newUserUsername.value.trim() || isAddingUser.value,
                            class: "px-2 py-1 font-mono text-xs text-bg bg-fg rounded transition-all duration-200 hover:bg-fg/90 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-accent/70"
                          }, _toDisplayString(isAddingUser.value ? 'â¦' : _ctx.$t('org.teams.add_button')), 9 /* TEXT, PROPS */, ["disabled"]),
                          _createElementVNode("button", {
                            type: "button",
                            class: "p-1 text-fg-subtle hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                            "aria-label": _ctx.$t('org.teams.cancel_add_user'),
                            onClick: _cache[8] || (_cache[8] = ($event: any) => (showAddUserFor.value = null))
                          }, [
                            _hoisted_14
                          ], 8 /* PROPS */, ["aria-label"])
                        ], 32 /* NEED_HYDRATION */)
                      ]))
                      : (_openBlock(), _createElementBlock("button", {
                        key: 1,
                        type: "button",
                        class: "mt-2 px-2 py-1 font-mono text-xs text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                        onClick: _cache[9] || (_cache[9] = ($event: any) => (showAddUserFor.value = teamName))
                      }, _toDisplayString(_ctx.$t('org.teams.add_member')), 1 /* TEXT */))
                  ]))
                  : _createCommentVNode("v-if", true)
              ]))
            }), 128 /* KEYED_FRAGMENT */)) ])), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (teams.value.length > 0 && filteredTeams.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "p-4 text-center"
          }, [ _createElementVNode("p", _hoisted_15, _toDisplayString(_ctx.$t('org.teams.no_match', { query: searchQuery.value })), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createElementVNode("div", { class: "p-3 border-t border-border" }, [ (showCreateTeam.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("form", {
                class: "flex items-center gap-2",
                onSubmit: _withModifiers(handleCreateTeam, ["prevent"])
              }, [ _createElementVNode("div", { class: "flex-1 flex items-center" }, [ _createElementVNode("span", _hoisted_16, _toDisplayString(__props.orgName) + ":\n            ", 1 /* TEXT */), _createElementVNode("label", _hoisted_17, _toDisplayString(_ctx.$t('org.teams.team_name_label')), 1 /* TEXT */), _createVNode(_component_InputBase, {
                    id: "new-team-name",
                    type: "text",
                    name: "new-team-name",
                    placeholder: _ctx.$t('org.teams.team_name_placeholder'),
                    "no-correct": "",
                    class: "flex-1 min-w-25 rounded-is-none",
                    size: "medium",
                    modelValue: newTeamName.value,
                    "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((newTeamName).value = $event))
                  }, null, 8 /* PROPS */, ["placeholder", "modelValue"]) ]), _createElementVNode("button", {
                  type: "submit",
                  disabled: !newTeamName.value.trim() || isCreatingTeam.value,
                  class: "px-3 py-2 font-mono text-xs text-bg bg-fg rounded transition-all duration-200 hover:bg-fg/90 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-accent/70"
                }, _toDisplayString(isCreatingTeam.value ? '…' : _ctx.$t('org.teams.create_button')), 9 /* TEXT, PROPS */, ["disabled"]), _createElementVNode("button", {
                  type: "button",
                  class: "p-1.5 text-fg-subtle hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                  "aria-label": _ctx.$t('org.teams.cancel_create'),
                  onClick: _cache[11] || (_cache[11] = ($event: any) => (showCreateTeam.value = false))
                }, [ _hoisted_18 ], 8 /* PROPS */, ["aria-label"]) ], 32 /* NEED_HYDRATION */) ])) : (_openBlock(), _createElementBlock("button", {
              key: 1,
              type: "button",
              class: "w-full px-3 py-2 font-mono text-sm text-fg-muted bg-bg border border-border rounded transition-colors duration-200 hover:text-fg hover:border-border-hover focus-visible:outline-accent/70",
              onClick: _cache[12] || (_cache[12] = ($event: any) => (showCreateTeam.value = true))
            }, _toDisplayString(_ctx.$t('org.teams.create_team')), 1 /* TEXT */)) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
