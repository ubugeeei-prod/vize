import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:users w-4 h-4 text-fg-muted", "aria-hidden": "true" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "absolute inset-is-2 top-1/2 -translate-y-1/2 i-lucide:search w-3.5 h-3.5 text-fg-subtle", "aria-hidden": "true" })
const _hoisted_3 = { for: "members-search", class: "sr-only" }
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("span", { class: "i-svg-spinners:ring-resize w-5 h-5 text-fg-muted animate-spin mx-auto", "aria-hidden": "true" })
const _hoisted_5 = { class: "font-mono text-sm text-fg-muted mt-2" }
const _hoisted_6 = { class: "font-mono text-sm text-red-400" }
const _hoisted_7 = { class: "font-mono text-sm text-fg-muted" }
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-4 h-4", "aria-hidden": "true" })
const _hoisted_9 = { class: "font-mono text-sm text-fg-muted" }
const _hoisted_10 = { for: "new-member-username", class: "sr-only" }
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-4 h-4", "aria-hidden": "true" })
import type { NewOperation } from '~/composables/useConnector'
import { buildScopeTeam } from '~/utils/npm/common'

type MemberRole = 'developer' | 'admin' | 'owner'
type MemberRoleFilter = MemberRole | 'all'

export default /*@__PURE__*/_defineComponent({
  __name: 'MembersPanel',
  props: {
    orgName: { type: String, required: true }
  },
  emits: ["select-team"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const {
  isConnected,
  lastExecutionTime,
  listOrgUsers,
  listOrgTeams,
  listTeamUsers,
  addOperation,
  error: connectorError,
} = useConnector()
// Members data: { username: role }
const members = shallowRef<Record<string, MemberRole>>({})
const isLoading = shallowRef(false)
const error = shallowRef<string | null>(null)
// Team membership data: { teamName: [members] }
const teamMembers = ref<Record<string, string[]>>({})
const isLoadingTeams = shallowRef(false)
// Search/filter
const searchQuery = shallowRef('')
const filterRole = shallowRef<MemberRoleFilter>('all')
const filterTeam = shallowRef<string>('')
const sortBy = shallowRef<'name' | 'role'>('name')
const sortOrder = shallowRef<'asc' | 'desc'>('asc')
// Add member form
const showAddMember = shallowRef(false)
const newUsername = shallowRef('')
const newRole = shallowRef<MemberRole>('developer')
const newTeam = shallowRef<string>('') // Empty string means "developers" (default)
const isAddingMember = shallowRef(false)
// Role priority for sorting
const rolePriority = { owner: 0, admin: 1, developer: 2 }
// Get teams a member belongs to
function getMemberTeams(username: string): string[] {
  const teams: string[] = []
  for (const [team, membersList] of Object.entries(teamMembers.value)) {
    if (membersList.includes(username)) {
      teams.push(team)
    }
  }
  return teams.sort()
}
// All team names (for filter dropdown)
const teamNames = computed(() => Object.keys(teamMembers.value).sort())
// Computed member list with teams
const memberList = computed(() => {
  return Object.entries(members.value).map(([name, role]) => ({
    name,
    role,
    teams: getMemberTeams(name),
  }))
})
// Filtered and sorted members
const filteredMembers = computed(() => {
  let result = memberList.value

  // Filter by search
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(m => m.name.toLowerCase().includes(query))
  }

  // Filter by role
  if (filterRole.value !== 'all') {
    result = result.filter(m => m.role === filterRole.value)
  }

  // Filter by team
  if (filterTeam.value) {
    result = result.filter(m => m.teams.includes(filterTeam.value!))
  }

  // Sort
  result = [...result].sort((a, b) => {
    if (sortBy.value === 'name') {
      return sortOrder.value === 'asc' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name)
    } else {
      const diff = rolePriority[a.role] - rolePriority[b.role]
      return sortOrder.value === 'asc' ? diff : -diff
    }
  })

  return result
})
// Role counts
const roleCounts = computed(() => {
  const counts = { developer: 0, admin: 0, owner: 0 }
  for (const role of Object.values(members.value)) {
    counts[role]++
  }
  return counts
})
// Refresh all data
function refreshData() {
  loadMembers()
  loadTeamMemberships()
}
// Load members
async function loadMembers() {
  if (!isConnected.value) return
  isLoading.value = true
  error.value = null
  try {
    const result = await listOrgUsers(props.orgName)
    if (result) {
      members.value = result
    } else {
      error.value = connectorError.value || 'Failed to load members'
    }
  } finally {
    isLoading.value = false
  }
}
// Load all teams and their members
async function loadTeamMemberships() {
  if (!isConnected.value) return
  isLoadingTeams.value = true
  try {
    const teamsResult = await listOrgTeams(props.orgName)
    if (teamsResult) {
      // Teams come as "org:team" format from npm, need @scope:team for API calls
      const teamPromises = teamsResult.map(async (fullTeamName: string) => {
        const teamName = fullTeamName.replace(`${props.orgName}:`, '')
        const membersResult = await listTeamUsers(buildScopeTeam(props.orgName, teamName))
        if (membersResult) {
          teamMembers.value[teamName] = membersResult
        }
      })
      await Promise.all(teamPromises)
    }
  } finally {
    isLoadingTeams.value = false
  }
}
// Add member (with optional team assignment)
async function handleAddMember() {
  if (!newUsername.value.trim()) return
  isAddingMember.value = true
  try {
    const username = newUsername.value.trim().replace(/^@/, '')
    // First operation: add user to org
    const orgOperation: NewOperation = {
      type: 'org:add-user',
      params: {
        org: props.orgName,
        user: username,
        role: newRole.value,
      },
      description: `Add @${username} to @${props.orgName} as ${newRole.value}`,
      command: `npm org set ${props.orgName} ${username} ${newRole.value}`,
    }
    const addedOrgOp = await addOperation(orgOperation)
    // Second operation: add user to team (if a team is selected)
    // This depends on the org operation completing first
    if (newTeam.value && addedOrgOp) {
      const scopeTeam = buildScopeTeam(props.orgName, newTeam.value)
      const teamOperation: NewOperation = {
        type: 'team:add-user',
        params: {
          scopeTeam,
          user: username,
        },
        description: `Add @${username} to team ${newTeam.value}`,
        command: `npm team add ${scopeTeam} ${username}`,
        dependsOn: addedOrgOp.id,
      }
      await addOperation(teamOperation)
    }
    newUsername.value = ''
    newTeam.value = ''
    showAddMember.value = false
  } finally {
    isAddingMember.value = false
  }
}
// Remove member
async function handleRemoveMember(username: string) {
  const operation: NewOperation = {
    type: 'org:rm-user',
    params: {
      org: props.orgName,
      user: username,
    },
    description: `Remove @${username} from @${props.orgName}`,
    command: `npm org rm ${props.orgName} ${username}`,
  }
  await addOperation(operation)
}
// Change role
async function handleChangeRole(username: string, newRoleValue: 'developer' | 'admin' | 'owner') {
  const operation: NewOperation = {
    type: 'org:add-user',
    params: {
      org: props.orgName,
      user: username,
      role: newRoleValue,
    },
    description: `Change @${username} role to ${newRoleValue} in @${props.orgName}`,
    command: `npm org set ${props.orgName} ${username} ${newRoleValue}`,
  }
  await addOperation(operation)
}
// Toggle sort
function toggleSort(field: 'name' | 'role') {
  if (sortBy.value === field) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = field
    sortOrder.value = 'asc'
  }
}
// Role badge color
function getRoleBadgeClass(role: string): string {
  switch (role) {
    case 'owner':
      return 'bg-purple-500/20 text-purple-400 border-purple-500/30'
    case 'admin':
      return 'bg-blue-500/20 text-blue-400 border-blue-500/30'
    default:
      return 'bg-fg-subtle/20 text-fg-muted border-border'
  }
}
const roleLabels = computed(() => ({
  owner: $t('org.members.role.owner'),
  admin: $t('org.members.role.admin'),
  developer: $t('org.members.role.developer'),
  all: $t('org.members.role.all'),
}))
function getRoleLabel(role: MemberRoleFilter): string {
  return roleLabels.value[role]
}
// Click on team badge to switch to teams tab and highlight
function handleTeamClick(teamName: string) {
  emit('select-team', teamName)
}
// Load on mount when connected
watch(
  isConnected,
  connected => {
    if (connected) {
      loadMembers()
      loadTeamMemberships()
    }
  },
  { immediate: true },
)
// Refresh data when operations complete
watch(lastExecutionTime, () => {
  if (isConnected.value) {
    loadMembers()
    loadTeamMemberships()
  }
})

return (_ctx: any,_cache: any) => {
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_SelectField = _resolveComponent("SelectField")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_unref(isConnected))
      ? (_openBlock(), _createElementBlock("section", {
        key: 0,
        class: "bg-bg-subtle border border-border rounded-lg overflow-hidden"
      }, [ _createElementVNode("div", { class: "flex items-center justify-between p-4 border-b border-border" }, [ _createElementVNode("h2", {
            id: "members-heading",
            class: "font-mono text-sm font-medium flex items-center gap-2"
          }, [ _hoisted_1, _createTextVNode("\n        " + _toDisplayString(_ctx.$t('org.members.title')) + "\n        ", 1 /* TEXT */), (memberList.value.length > 0) ? (_openBlock(), _createElementBlock("span", {
                key: 0,
                class: "text-fg-muted"
              }, "(" + _toDisplayString(memberList.value.length) + ")", 1 /* TEXT */)) : _createCommentVNode("v-if", true) ]), _createElementVNode("button", {
            type: "button",
            class: "p-1.5 text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
            "aria-label": _ctx.$t('org.members.refresh'),
            disabled: isLoading.value,
            onClick: refreshData
          }, [ _createElementVNode("span", {
              class: _normalizeClass(["i-lucide:refresh-ccw w-4 h-4", { 'motion-safe:animate-spin': isLoading.value || isLoadingTeams.value }]),
              "aria-hidden": "true"
            }, null, 2 /* CLASS */) ], 8 /* PROPS */, ["aria-label", "disabled"]) ]), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createElementVNode("div", { class: "flex flex-wrap items-center gap-2 p-3 border-b border-border bg-bg" }, [ _createElementVNode("div", { class: "flex-1 min-w-[150px] relative" }, [ _hoisted_2, _createElementVNode("label", _hoisted_3, _toDisplayString(_ctx.$t('org.members.filter_label')), 1 /* TEXT */), _createVNode(_component_InputBase, {
              id: "members-search",
              type: "search",
              name: "members-search",
              placeholder: _ctx.$t('org.members.filter_placeholder'),
              "no-correct": "",
              class: "w-full min-w-25 ps-7",
              size: "small",
              modelValue: searchQuery.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((searchQuery).value = $event))
            }, null, 8 /* PROPS */, ["placeholder", "modelValue"]) ]), _createElementVNode("div", {
            class: "flex items-center gap-1",
            role: "group",
            "aria-label": _ctx.$t('org.members.filter_by_role')
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList([ "all", "owner", "admin", "developer" ], (role) => {
              return (_openBlock(), _createElementBlock("button", {
                key: role,
                type: "button",
                class: _normalizeClass(["px-2 py-1 font-mono text-xs rounded transition-colors duration-200 focus-visible:outline-accent/70", filterRole.value === role ? 'bg-bg-muted text-fg' : 'text-fg-muted hover:text-fg']),
                "aria-pressed": filterRole.value === role,
                onClick: _cache[1] || (_cache[1] = ($event: any) => (filterRole.value = role))
              }, [
                _createTextVNode(_toDisplayString(getRoleLabel(role)) + "\n          ", 1 /* TEXT */),
                (role !== 'all')
                  ? (_openBlock(), _createElementBlock("span", {
                    key: 0,
                    class: "text-fg-subtle"
                  }, "(" + _toDisplayString(roleCounts.value[role]) + ")", 1 /* TEXT */))
                  : _createCommentVNode("v-if", true)
              ], 10 /* CLASS, PROPS */, ["aria-pressed"]))
            }), 128 /* KEYED_FRAGMENT */)) ], 8 /* PROPS */, ["aria-label"]), _createTextVNode("\n      " + "\n      "), (teamNames.value.length > 0) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createVNode(_component_SelectField, {
                label: _ctx.$t('org.members.filter_by_team'),
                "hidden-label": "",
                id: "team-filter",
                name: "team-filter",
                block: "",
                size: "sm",
                items: [{
  	label: _ctx.$t("org.members.all_teams"),
  	value: ""
  }, ...teamNames.value.map((team) => ({
  	label: team,
  	value: team
  }))],
                modelValue: filterTeam.value,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((filterTeam).value = $event))
              }, null, 8 /* PROPS */, ["label", "items", "modelValue"]) ])) : _createCommentVNode("v-if", true), _createElementVNode("div", {
            class: "flex items-center gap-1 text-xs",
            role: "group",
            "aria-label": _ctx.$t('org.members.sort_by')
          }, [ _createElementVNode("button", {
              type: "button",
              class: _normalizeClass(["px-2 py-1 font-mono rounded transition-colors duration-200 focus-visible:outline-accent/70", sortBy.value === 'name' ? 'bg-bg-muted text-fg' : 'text-fg-muted hover:text-fg']),
              "aria-pressed": sortBy.value === 'name',
              onClick: _cache[3] || (_cache[3] = ($event: any) => (toggleSort('name')))
            }, [ _createTextVNode(_toDisplayString(_ctx.$t('common.sort.name')) + "\n          ", 1 /* TEXT */), (sortBy.value === 'name') ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(sortOrder.value === 'asc' ? '↑' : '↓'), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 10 /* CLASS, PROPS */, ["aria-pressed"]), _createElementVNode("button", {
              type: "button",
              class: _normalizeClass(["px-2 py-1 font-mono rounded transition-colors duration-200 focus-visible:outline-accent/70", sortBy.value === 'role' ? 'bg-bg-muted text-fg' : 'text-fg-muted hover:text-fg']),
              "aria-pressed": sortBy.value === 'role',
              onClick: _cache[4] || (_cache[4] = ($event: any) => (toggleSort('role')))
            }, [ _createTextVNode(_toDisplayString(_ctx.$t('common.sort.role')) + "\n          ", 1 /* TEXT */), (sortBy.value === 'role') ? (_openBlock(), _createElementBlock("span", { key: 0 }, _toDisplayString(sortOrder.value === 'asc' ? '↑' : '↓'), 1 /* TEXT */)) : _createCommentVNode("v-if", true) ], 10 /* CLASS, PROPS */, ["aria-pressed"]) ], 8 /* PROPS */, ["aria-label"]) ]), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (isLoading.value && memberList.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "p-8 text-center"
          }, [ _hoisted_4, _createElementVNode("p", _hoisted_5, _toDisplayString(_ctx.$t('org.members.loading')), 1 /* TEXT */) ])) : (error.value) ? (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: "p-4 text-center",
              role: "alert"
            }, [ _createElementVNode("p", _hoisted_6, _toDisplayString(error.value), 1 /* TEXT */), _createElementVNode("button", {
                type: "button",
                class: "mt-2 font-mono text-xs text-fg-muted hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                onClick: loadMembers
              }, _toDisplayString(_ctx.$t('common.try_again')), 1 /* TEXT */) ])) : (memberList.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
              key: 2,
              class: "p-8 text-center"
            }, [ _createElementVNode("p", _hoisted_7, _toDisplayString(_ctx.$t('org.members.no_members')), 1 /* TEXT */) ])) : (_openBlock(), _createElementBlock("ul", {
            key: 3,
            class: "divide-y divide-border max-h-[400px] overflow-y-auto",
            "aria-label": _ctx.$t('org.members.list_label')
          }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(filteredMembers.value, (member) => {
              return (_openBlock(), _createElementBlock("li", {
                key: member.name,
                class: "flex flex-col gap-2 p-3 bg-bg hover:bg-bg-subtle transition-colors duration-200"
              }, [
                _createElementVNode("div", { class: "flex items-center justify-between" }, [
                  _createElementVNode("div", { class: "flex items-center gap-3" }, [
                    _createVNode(_component_NuxtLink, {
                      to: { name: '~username', params: { username: member.name } },
                      class: "font-mono text-sm text-fg hover:text-fg transition-colors duration-200"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("\n              ~"),
                        _createTextVNode(_toDisplayString(member.name), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to"]),
                    _createElementVNode("span", {
                      class: _normalizeClass(["px-1.5 py-0.5 font-mono text-xs border rounded", getRoleBadgeClass(member.role)])
                    }, _toDisplayString(getRoleLabel(member.role)), 3 /* TEXT, CLASS */)
                  ]),
                  _createElementVNode("div", { class: "flex items-center gap-1" }, [
                    _createElementVNode("label", {
                      for: `role-${member.name}`,
                      class: "sr-only"
                    }, _toDisplayString(_ctx.$t('org.members.change_role_for', { name: member.name })), 9 /* TEXT, PROPS */, ["for"]),
                    _createVNode(_component_SelectField, {
                      label: _ctx.$t('org.members.change_role_for', { name: member.name }),
                      "hidden-label": "",
                      id: `role-${member.name}`,
                      "model-value": member.role,
                      name: `role-${member.name}`,
                      block: "",
                      size: "sm",
                      items: [
                  { label: getRoleLabel('developer'), value: 'developer' },
                  { label: getRoleLabel('admin'), value: 'admin' },
                  { label: getRoleLabel('owner'), value: 'owner' },
                ],
                      value: member.role,
                      "onUpdate:modelValue": _cache[5] || (_cache[5] = (value) => handleChangeRole(member.name, value))
                    }, null, 8 /* PROPS */, ["label", "id", "model-value", "name", "items", "value"]),
                    _createTextVNode("\n            " + "\n            "),
                    _createElementVNode("button", {
                      type: "button",
                      class: "p-1 text-fg-subtle hover:text-red-400 transition-colors duration-200 rounded focus-visible:outline-accent/70",
                      "aria-label": _ctx.$t('org.members.remove_from_org', { name: member.name }),
                      onClick: _cache[6] || (_cache[6] = ($event: any) => (handleRemoveMember(member.name)))
                    }, [
                      _hoisted_8
                    ], 8 /* PROPS */, ["aria-label"])
                  ])
                ]),
                _createTextVNode("\n        " + "\n        "),
                (member.teams.length > 0)
                  ? (_openBlock(), _createElementBlock("div", {
                    key: 0,
                    class: "flex flex-wrap gap-1 ps-0"
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(member.teams, (team) => {
                      return (_openBlock(), _createElementBlock("button", {
                        key: team,
                        type: "button",
                        class: "inline-flex items-center gap-1 px-1.5 py-0.5 font-mono text-xs text-fg-muted border border-border rounded hover:text-fg hover:border-border-hover transition-colors duration-200 focus-visible:outline-accent/70",
                        "aria-label": _ctx.$t('org.members.view_team', { team }),
                        onClick: _cache[7] || (_cache[7] = ($event: any) => (handleTeamClick(team)))
                      }, _toDisplayString(team), 9 /* TEXT, PROPS */, ["aria-label"]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]))
                  : _createCommentVNode("v-if", true)
              ]))
            }), 128 /* KEYED_FRAGMENT */)) ])), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createTextVNode("\n\n    "), _createTextVNode("\n    "), (memberList.value.length > 0 && filteredMembers.value.length === 0) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: "p-4 text-center"
          }, [ _createElementVNode("p", _hoisted_9, _toDisplayString(_ctx.$t('org.members.no_match')), 1 /* TEXT */) ])) : _createCommentVNode("v-if", true), _createTextVNode("\n\n    "), _createTextVNode("\n    "), _createElementVNode("div", { class: "p-3 border-t border-border" }, [ (showAddMember.value) ? (_openBlock(), _createElementBlock("div", { key: 0 }, [ _createElementVNode("form", {
                class: "space-y-2",
                onSubmit: _withModifiers(handleAddMember, ["prevent"])
              }, [ _createElementVNode("label", _hoisted_10, _toDisplayString(_ctx.$t('org.members.username_label')), 1 /* TEXT */), _createVNode(_component_InputBase, {
                  id: "new-member-username",
                  type: "text",
                  name: "new-member-username",
                  placeholder: _ctx.$t('org.members.username_placeholder'),
                  "no-correct": "",
                  class: "w-full min-w-25",
                  size: "small",
                  modelValue: newUsername.value,
                  "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((newUsername).value = $event))
                }, null, 8 /* PROPS */, ["placeholder", "modelValue"]), _createElementVNode("div", { class: "flex items-center gap-2" }, [ _createVNode(_component_SelectField, {
                    label: _ctx.$t('org.members.role_label'),
                    "hidden-label": "",
                    id: "new-member-role",
                    name: "new-member-role",
                    block: "",
                    class: "flex-1",
                    size: "sm",
                    items: [
                  { label: _ctx.$t('org.members.role.developer'), value: 'developer' },
                  { label: _ctx.$t('org.members.role.admin'), value: 'admin' },
                  { label: _ctx.$t('org.members.role.owner'), value: 'owner' },
                ],
                    modelValue: newRole.value,
                    "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((newRole).value = $event))
                  }, null, 8 /* PROPS */, ["label", "items", "modelValue"]), _createTextVNode("\n            " + "\n            "), _createVNode(_component_SelectField, {
                    label: _ctx.$t('org.members.team_label'),
                    "hidden-label": "",
                    id: "new-member-team",
                    name: "new-member-team",
                    block: "",
                    class: "flex-1",
                    size: "sm",
                    items: [{
  	label: _ctx.$t("org.members.no_team"),
  	value: ""
  }, ...teamNames.value.map((team) => ({
  	label: team,
  	value: team
  }))],
                    modelValue: newTeam.value,
                    "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((newTeam).value = $event))
                  }, null, 8 /* PROPS */, ["label", "items", "modelValue"]), _createElementVNode("button", {
                    type: "submit",
                    disabled: !newUsername.value.trim() || isAddingMember.value,
                    class: "px-3 py-2 font-mono text-xs text-bg bg-fg rounded transition-all duration-200 hover:bg-fg/90 disabled:opacity-50 disabled:cursor-not-allowed focus-visible:outline-accent/70"
                  }, _toDisplayString(isAddingMember.value ? '…' : _ctx.$t('org.members.add_button')), 9 /* TEXT, PROPS */, ["disabled"]), _createElementVNode("button", {
                    type: "button",
                    class: "p-1.5 text-fg-subtle hover:text-fg transition-colors duration-200 rounded focus-visible:outline-accent/70",
                    "aria-label": _ctx.$t('org.members.cancel_add'),
                    onClick: _cache[11] || (_cache[11] = ($event: any) => (showAddMember.value = false))
                  }, [ _hoisted_11 ], 8 /* PROPS */, ["aria-label"]) ]) ], 32 /* NEED_HYDRATION */) ])) : (_openBlock(), _createElementBlock("button", {
              key: 1,
              type: "button",
              class: "w-full px-3 py-2 font-mono text-sm text-fg-muted bg-bg border border-border rounded transition-colors duration-200 hover:text-fg hover:border-border-hover focus-visible:outline-accent/70",
              onClick: _cache[12] || (_cache[12] = ($event: any) => (showAddMember.value = true))
            }, _toDisplayString(_ctx.$t('org.members.add_member')), 1 /* TEXT */)) ]) ]))
      : _createCommentVNode("v-if", true)
}
}

})
