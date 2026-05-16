import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "i-lucide:x w-3.5 h-3.5", "aria-hidden": "true" })
const _hoisted_2 = { for: "add-owner-username", class: "sr-only" }
import type { NewOperation } from '~/composables/useConnector'
const DEFAULT_VISIBLE_MAINTAINERS = 5

export default /*@__PURE__*/_defineComponent({
  __name: 'Maintainers',
  props: {
    packageName: { type: String, required: true },
    maintainers: { type: Array, required: false }
  },
  setup(__props: any) {

const props = __props
const {
  isConnected,
  lastExecutionTime,
  npmUser,
  addOperation,
  listPackageCollaborators,
  listTeamUsers,
} = useConnector()
const showAddOwner = shallowRef(false)
const newOwnerUsername = shallowRef('')
const isAdding = shallowRef(false)
const showAllMaintainers = shallowRef(false)
// Show admin controls when connected (let npm CLI handle permission errors)
const canManageOwners = computed(() => isConnected.value)
// Computed for visible maintainers with show more/fewer support
const visibleMaintainers = computed(() => {
  if (canManageOwners.value || showAllMaintainers.value) {
    return maintainerAccess.value
  }
  return maintainerAccess.value.slice(0, DEFAULT_VISIBLE_MAINTAINERS)
})
const hiddenMaintainersCount = computed(() =>
  Math.max(0, maintainerAccess.value.length - DEFAULT_VISIBLE_MAINTAINERS),
)
// Extract org name from scoped package
const orgName = computed(() => {
  if (!props.packageName.startsWith('@')) return null
  const match = props.packageName.match(/^@([^/]+)\//)
  return match ? match[1] : null
})
// Access data: who has access and via what
const collaborators = shallowRef<Record<string, 'read-only' | 'read-write'>>({})
const teamMembers = ref<Record<string, string[]>>({}) // team -> members
const isLoadingAccess = shallowRef(false)
// Compute access source for each maintainer
const maintainerAccess = computed(() => {
  if (!props.maintainers) return []

  return props.maintainers.map(maintainer => {
    const name = maintainer.name
    if (!name) return { ...maintainer, accessVia: [] as string[] }

    const accessVia: string[] = []

    // Check if they're a direct owner (in collaborators as a user, not team)
    if (collaborators.value[name]) {
      accessVia.push('owner')
    }

    // Check which teams they're in that have access
    for (const [collab, _perm] of Object.entries(collaborators.value)) {
      // Teams are in format "org:team"
      if (collab.includes(':')) {
        const teamName = collab.split(':')[1]
        const members = teamMembers.value[collab]
        if (members?.includes(name)) {
          accessVia.push(teamName || collab)
        }
      }
    }

    // If no specific access found, they're likely an owner
    if (accessVia.length === 0) {
      accessVia.push('owner')
    }

    return { ...maintainer, accessVia }
  })
})
// Load access information
async function loadAccessInfo() {
  if (!isConnected.value) return
  isLoadingAccess.value = true
  try {
    // Get collaborators (teams and users with access)
    const collabResult = await listPackageCollaborators(props.packageName)
    if (collabResult) {
      collaborators.value = collabResult
      // For each team collaborator, load its members
      const teamPromises: Promise<void>[] = []
      for (const collab of Object.keys(collabResult)) {
        if (collab.includes(':')) {
          teamPromises.push(
            listTeamUsers(collab).then((members: string[] | null) => {
              if (members) {
                teamMembers.value[collab] = members
              }
            }),
          )
        }
      }
      await Promise.all(teamPromises)
    }
  } finally {
    isLoadingAccess.value = false
  }
}
async function handleAddOwner() {
  if (!newOwnerUsername.value.trim()) return
  isAdding.value = true
  try {
    const username = newOwnerUsername.value.trim().replace(/^@/, '')
    const operation: NewOperation = {
      type: 'owner:add',
      params: {
        user: username,
        pkg: props.packageName,
      },
      description: `Add @${username} as owner of ${props.packageName}`,
      command: `npm owner add ${username} ${props.packageName}`,
    }
    await addOperation(operation)
    newOwnerUsername.value = ''
    showAddOwner.value = false
  } finally {
    isAdding.value = false
  }
}
async function handleRemoveOwner(username: string) {
  const operation: NewOperation = {
    type: 'owner:rm',
    params: {
      user: username,
      pkg: props.packageName,
    },
    description: `Remove @${username} from ${props.packageName}`,
    command: `npm owner rm ${username} ${props.packageName}`,
  }
  await addOperation(operation)
}
// Load access info when connected and for scoped packages
watch(
  [isConnected, () => props.packageName, lastExecutionTime],
  ([connected]) => {
    if (connected && orgName.value) {
      loadAccessInfo()
    }
  },
  { immediate: true },
)

return (_ctx: any,_cache: any) => {
  const _component_LinkBase = _resolveComponent("LinkBase")
  const _component_ButtonBase = _resolveComponent("ButtonBase")
  const _component_InputBase = _resolveComponent("InputBase")
  const _component_CollapsibleSection = _resolveComponent("CollapsibleSection")

  return (__props.maintainers?.length)
      ? (_openBlock(), _createBlock(_component_CollapsibleSection, {
        key: 0,
        id: "maintainers",
        title: _ctx.$t('package.maintainers.title')
      }, {
        default: _withCtx(() => [
          _createElementVNode("ul", {
            class: "space-y-2 list-none m-0 p-0 my-1 px-1",
            "aria-label": _ctx.$t('package.maintainers.list_label')
          }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(visibleMaintainers.value, (maintainer) => {
              return (_openBlock(), _createElementBlock("li", {
                key: maintainer.name ?? maintainer.email,
                class: "flex items-center justify-between gap-2"
              }, [
                _createElementVNode("div", { class: "flex items-center gap-2 min-w-0" }, [
                  (maintainer.name)
                    ? (_openBlock(), _createBlock(_component_LinkBase, {
                      key: 0,
                      to: {
                name: '~username',
                params: { username: maintainer.name },
              },
                      class: "link-subtle text-sm shrink-0",
                      dir: "ltr"
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode("\n            ~"),
                        _createTextVNode(_toDisplayString(maintainer.name), 1 /* TEXT */)
                      ]),
                      _: 2 /* DYNAMIC */
                    }, 8 /* PROPS */, ["to"]))
                    : (_openBlock(), _createElementBlock("span", {
                      key: 1,
                      class: "font-mono text-sm text-fg-muted",
                      dir: "ltr"
                    }, _toDisplayString(maintainer.email), 1 /* TEXT */)),
                  _createTextVNode("\n\n          " + "\n          "),
                  (_unref(isConnected) && maintainer.accessVia?.length && !isLoadingAccess.value)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: "text-xs text-fg-subtle truncate"
                    }, _toDisplayString(_ctx.$t('package.maintainers.via', {
                  teams: maintainer.accessVia.join(', '),
                })), 1 /* TEXT */))
                    : _createCommentVNode("v-if", true),
                  (canManageOwners.value && maintainer.name === _unref(npmUser))
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: "text-xs text-fg-subtle shrink-0"
                    }, _toDisplayString(_ctx.$t('package.maintainers.you')), 1 /* TEXT */))
                    : _createCommentVNode("v-if", true)
                ]),
                _createTextVNode("\n\n        " + "\n        "),
                (canManageOwners.value && maintainer.name && maintainer.name !== _unref(npmUser))
                  ? (_openBlock(), _createBlock(_component_ButtonBase, {
                    key: 0,
                    type: "button",
                    class: "hover:text-red-400",
                    "aria-label": 
              _ctx.$t('package.maintainers.remove_owner', {
                name: maintainer.name,
              })
            ,
                    onClick: _cache[0] || (_cache[0] = ($event: any) => (handleRemoveOwner(maintainer.name)))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_1
                    ]),
                    _: 2 /* DYNAMIC */
                  }, 8 /* PROPS */, ["aria-label"]))
                  : _createCommentVNode("v-if", true)
              ]))
            }), 128 /* KEYED_FRAGMENT */))
          ], 8 /* PROPS */, ["aria-label"]),
          _createTextVNode("\n\n    "),
          _createTextVNode("\n    "),
          (!canManageOwners.value && hiddenMaintainersCount.value > 0)
            ? (_openBlock(), _createBlock(_component_ButtonBase, {
              key: 0,
              onClick: _cache[1] || (_cache[1] = ($event: any) => (showAllMaintainers.value = !showAllMaintainers.value))
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(showAllMaintainers.value
            ? _ctx.$t('package.maintainers.show_less')
            : _ctx.$t('package.maintainers.show_more', {
                count: hiddenMaintainersCount.value,
              })), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }))
            : _createCommentVNode("v-if", true),
          _createTextVNode("\n\n    "),
          _createTextVNode("\n    "),
          (canManageOwners.value)
            ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "mt-3"
            }, [
              (showAddOwner.value)
                ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                  _createElementVNode("form", {
                    class: "flex items-center gap-2",
                    onSubmit: _withModifiers(handleAddOwner, ["prevent"])
                  }, [
                    _createElementVNode("label", _hoisted_2, _toDisplayString(_ctx.$t('package.maintainers.username_to_add')), 1 /* TEXT */),
                    _createVNode(_component_InputBase, {
                      id: "add-owner-username",
                      type: "text",
                      name: "add-owner-username",
                      placeholder: _ctx.$t('package.maintainers.username_placeholder'),
                      "no-correct": "",
                      class: "flex-1 min-w-25 m-1",
                      size: "small",
                      modelValue: newOwnerUsername.value,
                      "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((newOwnerUsername).value = $event))
                    }, null, 8 /* PROPS */, ["placeholder", "modelValue"]),
                    _createVNode(_component_ButtonBase, {
                      type: "submit",
                      disabled: !newOwnerUsername.value.trim() || isAdding.value
                    }, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(isAdding.value ? '…' : _ctx.$t('package.maintainers.add_button')), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["disabled"]),
                    _createVNode(_component_ButtonBase, {
                      "aria-label": _ctx.$t('package.maintainers.cancel_add'),
                      onClick: _cache[3] || (_cache[3] = ($event: any) => (showAddOwner.value = false)),
                      classicon: "i-lucide:x"
                    }, null, 8 /* PROPS */, ["aria-label"])
                  ], 32 /* NEED_HYDRATION */)
                ]))
                : (_openBlock(), _createBlock(_component_ButtonBase, {
                  key: 1,
                  type: "button",
                  onClick: _cache[4] || (_cache[4] = ($event: any) => (showAddOwner.value = true))
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_ctx.$t('package.maintainers.add_owner')), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
            ]))
            : _createCommentVNode("v-if", true)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["title"]))
      : _createCommentVNode("v-if", true)
}
}

})
