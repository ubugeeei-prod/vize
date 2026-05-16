import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-search" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-server" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-user" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-x" });
import { computed, markRaw, ref, shallowRef, toRef } from "vue";
import { host as localHost } from "@@/js/config.js";
import { $i } from "@/i.js";
import { i18n } from "@/i18n.js";
import { instance } from "@/instance.js";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { apLookup } from "@/utility/lookup.js";
import { useRouter } from "@/router.js";
import MkButton from "@/components/MkButton.vue";
import MkFoldableSection from "@/components/MkFoldableSection.vue";
import MkInput from "@/components/MkInput.vue";
import MkNotesTimeline from "@/components/MkNotesTimeline.vue";
import MkRadios from "@/components/MkRadios.vue";
import MkUserCardMini from "@/components/MkUserCardMini.vue";
import { Paginator } from "@/utility/paginator.js";
export default {
  __name: "search.note",
  props: {
    query: {
      type: String,
      required: false,
      default: ""
    },
    userId: {
      type: String,
      required: false,
      default: undefined
    },
    username: {
      type: String,
      required: false,
      default: undefined
    },
    host: {
      type: [String, null],
      required: false,
      default: ""
    }
  },
  async setup(__props) {
    let __temp, __restore;
    const props = __props;
    const router = useRouter();
    const key = ref(0);
    const paginator = shallowRef(null);
    const searchQuery = ref(toRef(props, "query").value);
    const hostInput = ref(toRef(props, "host").value);
    const user = shallowRef(null);
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    const noteSearchableScope = instance.noteSearchableScope ?? "local";
    //#region set user
    let fetchedUser = null;
    if (props.userId) {
      fetchedUser = await misskeyApi("users/show", { userId: props.userId }).catch(() => null);
    }
    if (props.username && fetchedUser == null) {
      fetchedUser = await misskeyApi("users/show", {
        username: props.username,
        ...props.host ? { host: props.host } : {}
      }).catch(() => null);
    }
    if (fetchedUser != null) {
      if (!(noteSearchableScope === "local" && fetchedUser.host != null)) {
        user.value = fetchedUser;
      }
    }
    //#endregion
    const searchScope = ref((() => {
      if (user.value != null) return "user";
      if (noteSearchableScope === "local") return "local";
      if (hostInput.value) return "server";
      return "all";
    })());
    const searchScopeDef = computed(() => {
      const options = [];
      if (instance.federation !== "none" && noteSearchableScope === "global") {
        options.push({
          value: "all",
          label: i18n.ts._search.searchScopeAll
        });
      }
      options.push({
        value: "local",
        label: instance.federation === "none" ? i18n.ts._search.searchScopeAll : i18n.ts._search.searchScopeLocal
      });
      if (instance.federation !== "none" && noteSearchableScope === "global") {
        options.push({
          value: "server",
          label: i18n.ts._search.searchScopeServer
        });
      }
      options.push({
        value: "user",
        label: i18n.ts._search.searchScopeUser
      });
      return options;
    });
    const fixHostIfLocal = (target) => {
      if (!target || target === localHost) return ".";
      return target;
    };
    const searchParams = computed(() => {
      const trimmedQuery = searchQuery.value.trim();
      if (!trimmedQuery) return null;
      if (searchScope.value === "user") {
        if (user.value == null) return null;
        return {
          query: trimmedQuery,
          host: fixHostIfLocal(user.value.host),
          userId: user.value.id
        };
      }
      if (instance.federation !== "none" && searchScope.value === "server") {
        let trimmedHost = hostInput.value?.trim();
        if (!trimmedHost) return null;
        if (trimmedHost.startsWith("https://") || trimmedHost.startsWith("http://")) {
          try {
            trimmedHost = new URL(trimmedHost).host;
          } catch (err) {}
        }
        return {
          query: trimmedQuery,
          host: fixHostIfLocal(trimmedHost)
        };
      }
      if (instance.federation === "none" || searchScope.value === "local") {
        return {
          query: trimmedQuery,
          host: "."
        };
      }
      return { query: trimmedQuery };
    });
    function selectUser() {
      os.selectUser({
        includeSelf: true,
        localOnly: instance.noteSearchableScope === "local"
      }).then((_user) => {
        user.value = _user;
      });
    }
    function selectSelf() {
      user.value = $i;
    }
    function removeUser() {
      user.value = null;
    }
    async function search() {
      if (searchParams.value == null) return;
      //#region AP lookup
      if (searchParams.value.query.startsWith("https://") && !searchParams.value.query.includes(" ")) {
        const confirm = await os.confirm({
          type: "info",
          text: i18n.ts.lookupConfirm
        });
        if (!confirm.canceled) {
          const res = await apLookup(searchParams.value.query);
          if (res.type === "User") {
            router.push("/@:acct/:page?", { params: { acct: `${res.object.username}@${res.object.host}` } });
          } else if (res.type === "Note") {
            router.push("/notes/:noteId/:initialTab?", { params: { noteId: res.object.id } });
          }
          return;
        }
      }
      //#endregion
      if (searchParams.value.query.length > 1 && !searchParams.value.query.includes(" ")) {
        if (searchParams.value.query.startsWith("@")) {
          const confirm = await os.confirm({
            type: "info",
            text: i18n.ts.lookupConfirm
          });
          if (!confirm.canceled) {
            router.pushByPath(`/${searchParams.value.query}`);
            return;
          }
        }
        if (searchParams.value.query.startsWith("#")) {
          const confirm = await os.confirm({
            type: "info",
            text: i18n.ts.openTagPageConfirm
          });
          if (!confirm.canceled) {
            router.push("/tags/:tag", { params: { tag: searchParams.value.query.substring(1) } });
            return;
          }
        }
      }
      paginator.value = markRaw(new Paginator("notes/search", {
        limit: 10,
        params: { ...searchParams.value }
      }));
      key.value++;
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", { class: "_gaps" }, [_createElementVNode("div", { class: "_gaps" }, [
        _createVNode(MkInput, {
          large: "",
          autofocus: "",
          type: "search",
          onEnter: _withModifiers(search, ["prevent"]),
          modelValue: searchQuery.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => searchQuery.value = $event)
        }, {
          prefix: _withCtx(() => [_hoisted_1]),
          _: 1
        }, 8, ["modelValue"]),
        _createVNode(MkFoldableSection, { expanded: "" }, {
          header: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.options),
            1
            /* TEXT */
          )]),
          default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(MkRadios, {
              options: searchScopeDef.value,
              modelValue: searchScope.value,
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => searchScope.value = $event)
            }, null, 8, ["options", "modelValue"]),
            _unref(instance).federation !== "none" && searchScope.value === "server" ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.subOptionRoot)
              },
              [_createVNode(MkInput, {
                placeholder: _unref(i18n).ts._search.serverHostPlaceholder,
                onEnter: _withModifiers(search, ["prevent"]),
                modelValue: hostInput.value,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => hostInput.value = $event)
              }, {
                label: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._search.pleaseEnterServerHost),
                  1
                  /* TEXT */
                )]),
                prefix: _withCtx(() => [_hoisted_2]),
                _: 1
              }, 8, ["placeholder", "modelValue"])],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true),
            searchScope.value === "user" ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.subOptionRoot)
              },
              [_createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.userSelectLabel) },
                _toDisplayString(_unref(i18n).ts._search.pleaseSelectUser),
                3
                /* TEXT, CLASS */
              ), _createElementVNode("div", { class: "_gaps" }, [user.value == null ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.userSelectButtons)
                },
                [_unref($i) != null ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createVNode(
                  MkButton,
                  {
                    transparent: "",
                    class: _normalizeClass(_ctx.$style.userSelectButton),
                    onClick: selectSelf
                  },
                  {
                    default: _withCtx(() => [_createElementVNode(
                      "div",
                      { class: _normalizeClass(_ctx.$style.userSelectButtonInner) },
                      [_createElementVNode("span", null, [_hoisted_3, _hoisted_4]), _createElementVNode(
                        "span",
                        null,
                        _toDisplayString(_unref(i18n).ts.selectSelf),
                        1
                        /* TEXT */
                      )],
                      2
                      /* CLASS */
                    )]),
                    _: 1
                  },
                  2
                  /* CLASS */
                )])) : _createCommentVNode("v-if", true), _createElementVNode(
                  "div",
                  { style: _normalizeStyle(_unref($i) == null ? "grid-column: span 2;" : undefined) },
                  [_createVNode(
                    MkButton,
                    {
                      transparent: "",
                      class: _normalizeClass(_ctx.$style.userSelectButton),
                      onClick: selectUser
                    },
                    {
                      default: _withCtx(() => [_createElementVNode(
                        "div",
                        { class: _normalizeClass(_ctx.$style.userSelectButtonInner) },
                        [_createElementVNode("span", null, [_hoisted_5]), _createElementVNode(
                          "span",
                          null,
                          _toDisplayString(_unref(i18n).ts.selectUser),
                          1
                          /* TEXT */
                        )],
                        2
                        /* CLASS */
                      )]),
                      _: 1
                    },
                    2
                    /* CLASS */
                  )],
                  4
                  /* STYLE */
                )],
                2
                /* CLASS */
              )) : (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 1,
                  class: _normalizeClass(_ctx.$style.userSelectedButtons)
                },
                [_createElementVNode("div", { style: "overflow: hidden;" }, [_createVNode(MkUserCardMini, {
                  user: user.value,
                  withChart: false
                }, null, 8, ["user", "withChart"])]), _createElementVNode("div", null, [_createElementVNode(
                  "button",
                  {
                    class: _normalizeClass(["_button", _ctx.$style.userSelectedRemoveButton]),
                    onClick: removeUser
                  },
                  [_hoisted_6],
                  2
                  /* CLASS */
                )])],
                2
                /* CLASS */
              ))])],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)
          ])]),
          _: 1
        }),
        _createElementVNode("div", null, [_createVNode(MkButton, {
          large: "",
          primary: "",
          gradate: "",
          rounded: "",
          disabled: searchParams.value == null,
          style: "margin: 0 auto;",
          onClick: search
        }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.search),
            1
            /* TEXT */
          )]),
          _: 1
        }, 8, ["disabled"])])
      ]), paginator.value ? (_openBlock(), _createBlock(MkFoldableSection, { key: 0 }, {
        header: _withCtx(() => [_createTextVNode(
          _toDisplayString(_unref(i18n).ts.searchResult),
          1
          /* TEXT */
        )]),
        default: _withCtx(() => [_createVNode(MkNotesTimeline, {
          key: `searchNotes:${key.value}`,
          paginator: paginator.value
        }, null, 8, ["paginator"])]),
        _: 1
      })) : _createCommentVNode("v-if", true)]);
    };
  }
};
