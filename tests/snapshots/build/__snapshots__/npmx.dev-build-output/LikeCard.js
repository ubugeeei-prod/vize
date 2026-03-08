import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("p", { class: "transition-transform duration-150 group-hover:rotate-45 pb-1" }, "↗")
import { useAtproto } from '~/composables/atproto/useAtproto'
import { togglePackageLike } from '~/utils/atproto/likes'

export default /*@__PURE__*/_defineComponent({
  __name: 'LikeCard',
  props: {
    packageUrl: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props
const compactNumberFormatter = useCompactNumberFormatter()
function extractPackageFromRef(ref: string) {
  return /https:\/\/npmx.dev\/package\/(?<pkg>.*)/.exec(ref)?.groups?.pkg ?? ref
}
const name = computed(() => extractPackageFromRef(props.packageUrl))
const { user } = useAtproto()
const authModal = useModal('auth-modal')
const { data: likesData } = useFetch(() => `/api/social/likes/${name.value}`, {
  default: () => ({ totalLikes: 0, userHasLiked: false }),
  server: false,
})
const isLikeActionPending = ref(false)
const likeAction = async () => {
  if (user.value?.handle == null) {
    authModal.open()
    return
  }

  if (isLikeActionPending.value) return

  const currentlyLiked = likesData.value?.userHasLiked ?? false
  const currentLikes = likesData.value?.totalLikes ?? 0

  // Optimistic update
  likesData.value = {
    totalLikes: currentlyLiked ? currentLikes - 1 : currentLikes + 1,
    userHasLiked: !currentlyLiked,
  }

  isLikeActionPending.value = true

  try {
    const result = await togglePackageLike(name.value, currentlyLiked, user.value?.handle)

    isLikeActionPending.value = false

    if (result.success) {
      // Update with server response
      likesData.value = result.data
    } else {
      // Revert on error
      likesData.value = {
        totalLikes: currentLikes,
        userHasLiked: currentlyLiked,
      }
    }
  } catch (e) {
    // Revert on error
    likesData.value = {
      totalLikes: currentLikes,
      userHasLiked: currentlyLiked,
    }
    isLikeActionPending.value = false
  }
}

return (_ctx: any,_cache: any) => {
  const _component_TooltipApp = _resolveComponent("TooltipApp")
  const _component_ClientOnly = _resolveComponent("ClientOnly")
  const _component_BaseCard = _resolveComponent("BaseCard")
  const _component_NuxtLink = _resolveComponent("NuxtLink")

  return (_openBlock(), _createBlock(_component_NuxtLink, { to: _ctx.packageRoute(name.value) }, {
      default: _withCtx(() => [
        _createVNode(_component_BaseCard, { class: "font-mono flex justify-between min-w-0" }, {
          default: _withCtx(() => [
            _createElementVNode("span", {
              class: "truncate min-w-0",
              title: name.value
            }, _toDisplayString(name.value), 9 /* TEXT, PROPS */, ["title"]),
            _createElementVNode("div", { class: "flex items-center gap-4 justify-between shrink-0" }, [
              _createVNode(_component_ClientOnly, null, {
                default: _withCtx(() => [
                  _createVNode(_component_TooltipApp, {
                    text: _unref(likesData)?.userHasLiked ? _ctx.$t('package.likes.unlike') : _ctx.$t('package.likes.like'),
                    position: "bottom"
                  }, {
                    default: _withCtx(() => [
                      _createElementVNode("button", {
                        onClick: _withModifiers(likeAction, ["prevent"]),
                        type: "button",
                        title: 
                  _unref(likesData)?.userHasLiked ? _ctx.$t('package.likes.unlike') : _ctx.$t('package.likes.like')
                ,
                        class: "inline-flex items-center gap-1.5 font-mono text-sm text-fg hover:text-fg-muted transition-colors duration-200",
                        "aria-label": 
                  _unref(likesData)?.userHasLiked ? _ctx.$t('package.likes.unlike') : _ctx.$t('package.likes.like')
              
                      }, [
                        _createElementVNode("span", {
                          class: _normalizeClass(["w-4 h-4", 
                    _unref(likesData)?.userHasLiked
                      ? 'i-lucide-heart-minus text-red-500'
                      : 'i-lucide-heart-plus'
                  ]),
                          "aria-hidden": "true"
                        }, null, 2 /* CLASS */),
                        _createElementVNode("span", null, _toDisplayString(_unref(compactNumberFormatter).format(_unref(likesData)?.totalLikes ?? 0)), 1 /* TEXT */)
                      ], 8 /* PROPS */, ["title", "aria-label"])
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["text"])
                ]),
                _: 1 /* STABLE */
              }),
              _hoisted_1
            ])
          ]),
          _: 1 /* STABLE */
        })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["to"]))
}
}

})
