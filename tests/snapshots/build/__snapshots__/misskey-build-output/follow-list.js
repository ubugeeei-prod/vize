import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { computed, markRaw } from "vue";
import MkUserInfo from "@/components/MkUserInfo.vue";
import MkPagination from "@/components/MkPagination.vue";
import { Paginator } from "@/utility/paginator.js";
export default {
  __name: "follow-list",
  props: {
    user: {
      type: null,
      required: true
    },
    type: {
      type: String,
      required: true
    }
  },
  setup(__props) {
    const props = __props;
    const followingPaginator = markRaw(new Paginator("users/following", {
      limit: 20,
      computedParams: computed(() => ({ userId: props.user.id }))
    }));
    const followersPaginator = markRaw(new Paginator("users/followers", {
      limit: 20,
      computedParams: computed(() => ({ userId: props.user.id }))
    }));
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", null, [_createVNode(MkPagination, {
        paginator: __props.type === "following" ? _unref(followingPaginator) : _unref(followersPaginator),
        withControl: ""
      }, {
        default: _withCtx(({ items }) => [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.users) },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(items.map((x) => __props.type === "following" ? x.followee : x.follower), (user) => {
              return _openBlock(), _createBlock(MkUserInfo, {
                key: user.id,
                user
              }, null, 8, ["user"]);
            }),
            128
            /* KEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        )]),
        _: 1
      }, 8, ["paginator"])]);
    };
  }
};
