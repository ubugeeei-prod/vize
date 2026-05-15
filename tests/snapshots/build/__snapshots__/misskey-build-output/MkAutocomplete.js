import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref, withModifiers as _withModifiers } from "vue";
const _hoisted_1 = { class: "name" };
import { markRaw, ref, useTemplateRef, computed, onUpdated, onMounted, onBeforeUnmount, nextTick, watch } from "vue";
import sanitizeHtml from "sanitize-html";
import { emojilist, getEmojiName } from "@@/js/emojilist.js";
import { char2twemojiFilePath, char2fluentEmojiFilePath } from "@@/js/emoji-base.js";
import { MFM_TAGS, MFM_PARAMS } from "@@/js/const.js";
import { elementContains } from "@/utility/element-contains.js";
import { acct } from "@/filters/user.js";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { store } from "@/store.js";
import { i18n } from "@/i18n.js";
import { miLocalStorage } from "@/local-storage.js";
import { customEmojis } from "@/custom-emojis.js";
import { searchEmoji, searchEmojiExact } from "@/utility/search-emoji.js";
import { prefer } from "@/preferences.js";
const lib = emojilist.filter((x) => x.category !== "flags");
const unicodeEmojiDB = computed(() => {
  //#region Unicode Emoji
  const char2path = prefer.r.emojiStyle.value === "twemoji" ? char2twemojiFilePath : char2fluentEmojiFilePath;
  const unicodeEmojiDB = lib.map((x) => ({
    emoji: x.char,
    name: x.name,
    url: char2path(x.char)
  }));
  for (const index of Object.values(store.s.additionalUnicodeEmojiIndexes)) {
    for (const [emoji, keywords] of Object.entries(index)) {
      for (const k of keywords) {
        unicodeEmojiDB.push({
          emoji,
          name: k,
          aliasOf: getEmojiName(emoji),
          url: char2path(emoji)
        });
      }
    }
  }
  unicodeEmojiDB.sort((a, b) => a.name.length - b.name.length);
  return unicodeEmojiDB;
});
const emojiDb = computed(() => {
  //#region Unicode Emoji
  //#endregion
  //#region Custom Emoji
  const customEmojiDB = [];
  for (const x of customEmojis.value) {
    customEmojiDB.push({
      name: x.name,
      emoji: `:${x.name}:`,
      isCustomEmoji: true
    });
    if (x.aliases) {
      for (const alias of x.aliases) {
        customEmojiDB.push({
          name: alias,
          aliasOf: x.name,
          emoji: `:${x.name}:`,
          isCustomEmoji: true
        });
      }
    }
  }
  customEmojiDB.sort((a, b) => a.name.length - b.name.length);
  //#endregion
  return markRaw([...customEmojiDB, ...unicodeEmojiDB.value]);
});
const __default__ = {
  emojiDb,
  emojilist
};
export default /* @__PURE__ */ Object.assign(__default__, {
  __name: "MkAutocomplete",
  props: {
    type: {
      type: null,
      required: true
    },
    q: {
      type: null,
      required: true
    },
    textarea: {
      type: null,
      required: true
    },
    close: {
      type: Function,
      required: true
    },
    x: {
      type: Number,
      required: true
    },
    y: {
      type: Number,
      required: true
    }
  },
  emits: [
    "done",
    "payload",
    "closed"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    //const props = defineProps<PropsType<keyof CompleteInfo>>();
    // ↑と同じだけど↓にしないとdiscriminated unionにならない。
    // https://www.typescriptlang.org/docs/handbook/typescript-in-5-minutes-func.html#discriminated-unions
    const suggests = ref();
    const rootEl = useTemplateRef("rootEl");
    const fetching = ref(true);
    const users = ref([]);
    const hashtags = ref([]);
    const emojis = ref([]);
    const items = ref([]);
    const mfmTags = ref([]);
    const mfmParams = ref([]);
    const select = ref(-1);
    const zIndex = os.claimZIndex("high");
    function completeMfmParam(param) {
      if (props.type !== "mfmParam") throw new Error("Invalid type");
      complete("mfmParam", props.q.params.toSpliced(-1, 1, param).join(","));
    }
    function complete(type, value) {
      emit("done", {
        type,
        value
      });
      emit("closed");
      if (type === "emoji" || type === "emojiComplete") {
        let recents = store.s.recentlyUsedEmojis;
        recents = recents.filter((emoji) => emoji !== value);
        recents.unshift(value);
        store.set("recentlyUsedEmojis", recents.splice(0, 32));
      }
    }
    function setPosition() {
      if (!rootEl.value) return;
      if (props.x + rootEl.value.offsetWidth > window.innerWidth) {
        rootEl.value.style.left = window.innerWidth - rootEl.value.offsetWidth + "px";
      } else {
        rootEl.value.style.left = `${props.x}px`;
      }
      if (props.y + rootEl.value.offsetHeight > window.innerHeight) {
        rootEl.value.style.top = props.y - rootEl.value.offsetHeight + "px";
        rootEl.value.style.marginTop = "0";
      } else {
        rootEl.value.style.top = props.y + "px";
        rootEl.value.style.marginTop = "calc(1em + 8px)";
      }
    }
    function exec() {
      select.value = -1;
      if (suggests.value) {
        for (const el of Array.from(items.value)) {
          el.removeAttribute("data-selected");
        }
      }
      if (props.type === "user") {
        if (!props.q) {
          users.value = [];
          fetching.value = false;
          return;
        }
        const cacheKey = `autocomplete:user:${props.q}`;
        const cache = sessionStorage.getItem(cacheKey);
        if (cache) {
          users.value = JSON.parse(cache);
          fetching.value = false;
        } else {
          const [username, host] = props.q.toString().split("@");
          misskeyApi("users/search-by-username-and-host", {
            username,
            host,
            limit: 10,
            detail: false
          }).then((searchedUsers) => {
            users.value = searchedUsers;
            fetching.value = false;
            // キャッシュ
            sessionStorage.setItem(cacheKey, JSON.stringify(searchedUsers));
          });
        }
      } else if (props.type === "hashtag") {
        if (!props.q || props.q === "") {
          hashtags.value = JSON.parse(miLocalStorage.getItem("hashtags") ?? "[]");
          fetching.value = false;
        } else {
          const cacheKey = `autocomplete:hashtag:${props.q}`;
          const cache = sessionStorage.getItem(cacheKey);
          if (cache) {
            const hashtags = JSON.parse(cache);
            hashtags.value = hashtags;
            fetching.value = false;
          } else {
            misskeyApi("hashtags/search", {
              query: props.q,
              limit: 30
            }).then((searchedHashtags) => {
              hashtags.value = searchedHashtags;
              fetching.value = false;
              // キャッシュ
              sessionStorage.setItem(cacheKey, JSON.stringify(searchedHashtags));
            });
          }
        }
      } else if (props.type === "emoji") {
        if (!props.q || props.q === "") {
          // 最近使った絵文字をサジェスト
          emojis.value = store.s.recentlyUsedEmojis.map((emoji) => emojiDb.value.find((dbEmoji) => dbEmoji.emoji === emoji)).filter((x) => x);
          return;
        }
        emojis.value = searchEmoji(props.q, emojiDb.value);
      } else if (props.type === "emojiComplete") {
        emojis.value = searchEmojiExact(props.q, unicodeEmojiDB.value);
      } else if (props.type === "mfmTag") {
        if (!props.q || props.q === "") {
          mfmTags.value = MFM_TAGS;
          return;
        }
        mfmTags.value = MFM_TAGS.filter((tag) => tag.startsWith(props.q ?? ""));
      } else if (props.type === "mfmParam") {
        if (props.q.params.at(-1) === "") {
          mfmParams.value = MFM_PARAMS[props.q.tag] ?? [];
          return;
        }
        mfmParams.value = MFM_PARAMS[props.q.tag].filter((param) => param.startsWith(props.q.params.at(-1) ?? ""));
      }
    }
    function onMousedown(event) {
      if (!elementContains(rootEl.value, event.target) && rootEl.value !== event.target) props.close();
    }
    function onKeydown(event) {
      const cancel = () => {
        event.preventDefault();
        event.stopPropagation();
      };
      switch (event.key) {
        case "Enter":
          if (select.value !== -1) {
            cancel();
            items.value[select.value].click();
          } else {
            props.close();
          }
          break;
        case "Escape":
          cancel();
          props.close();
          break;
        case "ArrowUp":
          if (select.value !== -1) {
            cancel();
            selectPrev();
          } else {
            props.close();
          }
          break;
        case "ArrowDown":
          cancel();
          selectNext();
          break;
        case "Tab":
          if (event.shiftKey) {
            if (select.value !== -1) {
              cancel();
              selectPrev();
            } else {
              props.close();
            }
          } else {
            cancel();
            selectNext();
          }
          break;
        default:
          event.stopPropagation();
          props.textarea.focus();
      }
    }
    function selectNext() {
      if (++select.value >= items.value.length) select.value = 0;
      if (items.value.length === 0) select.value = -1;
      applySelect();
    }
    function selectPrev() {
      if (--select.value < 0) select.value = items.value.length - 1;
      applySelect();
    }
    function applySelect() {
      for (const el of Array.from(items.value)) {
        el.removeAttribute("data-selected");
      }
      if (select.value !== -1) {
        items.value[select.value].setAttribute("data-selected", "true");
        items.value[select.value].focus();
      }
    }
    function chooseUser() {
      props.close();
      os.selectUser({ includeSelf: true }).then((user) => {
        complete("user", user);
        props.textarea.focus();
      });
    }
    onUpdated(() => {
      setPosition();
      items.value = suggests.value?.children ?? [];
    });
    onMounted(() => {
      setPosition();
      props.textarea.addEventListener("keydown", onKeydown);
      window.document.body.addEventListener("mousedown", onMousedown);
      nextTick(() => {
        exec();
        watch(() => props.q, () => {
          nextTick(() => {
            exec();
          });
        });
      });
    });
    onBeforeUnmount(() => {
      props.textarea.removeEventListener("keydown", onKeydown);
      window.document.body.removeEventListener("mousedown", onMousedown);
    });
    return (_ctx, _cache) => {
      const _component_MkUserName = _resolveComponent("MkUserName");
      const _component_MkCustomEmoji = _resolveComponent("MkCustomEmoji");
      const _component_MkEmoji = _resolveComponent("MkEmoji");
      return _openBlock(), _createElementBlock(
        "div",
        {
          ref_key: "rootEl",
          ref: rootEl,
          class: _normalizeClass(["_popup _shadow", _ctx.$style.root]),
          style: _normalizeStyle({ zIndex: _unref(zIndex) }),
          onContextmenu: _cache[0] || (_cache[0] = _withModifiers(() => {}, ["prevent"]))
        },
        [__props.type === "user" ? (_openBlock(), _createElementBlock(
          "ol",
          {
            key: 0,
            ref_key: "suggests",
            ref: suggests,
            class: _normalizeClass(_ctx.$style.list)
          },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(users.value, (user) => {
              return _openBlock(), _createElementBlock("li", {
                tabindex: "-1",
                class: _normalizeClass(_ctx.$style.item),
                onClick: ($event) => complete(__props.type, user),
                onKeydown
              }, [
                _createElementVNode("img", {
                  class: _normalizeClass(_ctx.$style.avatar),
                  src: user.avatarUrl
                }, null, 10, ["src"]),
                _createElementVNode(
                  "span",
                  { class: _normalizeClass(_ctx.$style.userName) },
                  [_createVNode(_component_MkUserName, {
                    key: user.id,
                    user
                  }, null, 8, ["user"])],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "span",
                  null,
                  "@" + _toDisplayString(acct(user)),
                  1
                  /* TEXT */
                )
              ], 42, ["onClick"]);
            }),
            256
            /* UNKEYED_FRAGMENT */
          )), _createElementVNode(
            "li",
            {
              tabindex: "-1",
              class: _normalizeClass(_ctx.$style.item),
              onClick: _cache[1] || (_cache[1] = ($event) => chooseUser()),
              onKeydown
            },
            _toDisplayString(i18n.ts.selectUser),
            35
            /* TEXT, CLASS, NEED_HYDRATION */
          )],
          2
          /* CLASS */
        )) : __props.type === "hashtag" && hashtags.value.length > 0 ? (_openBlock(), _createElementBlock(
          "ol",
          {
            key: 1,
            ref_key: "suggests",
            ref: suggests,
            class: _normalizeClass(_ctx.$style.list)
          },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(hashtags.value, (hashtag) => {
              return _openBlock(), _createElementBlock("li", {
                tabindex: "-1",
                class: _normalizeClass(_ctx.$style.item),
                onClick: ($event) => complete(__props.type, hashtag),
                onKeydown
              }, [_createElementVNode(
                "span",
                _hoisted_1,
                _toDisplayString(hashtag),
                1
                /* TEXT */
              )], 42, ["onClick"]);
            }),
            256
            /* UNKEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        )) : __props.type === "emoji" || __props.type === "emojiComplete" && emojis.value.length > 0 ? (_openBlock(), _createElementBlock(
          "ol",
          {
            key: 2,
            ref_key: "suggests",
            ref: suggests,
            class: _normalizeClass(_ctx.$style.list)
          },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(emojis.value, (emoji) => {
              return _openBlock(), _createElementBlock("li", {
                key: emoji.emoji,
                class: _normalizeClass(_ctx.$style.item),
                tabindex: "-1",
                onClick: ($event) => complete(__props.type, emoji.emoji),
                onKeydown
              }, [
                "isCustomEmoji" in emoji && emoji.isCustomEmoji ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
                  key: 0,
                  name: emoji.emoji,
                  class: _normalizeClass(_ctx.$style.emoji),
                  fallbackToImage: true
                }, null, 10, ["name", "fallbackToImage"])) : (_openBlock(), _createBlock(_component_MkEmoji, {
                  key: 1,
                  emoji: emoji.emoji,
                  class: _normalizeClass(_ctx.$style.emoji)
                }, null, 10, ["emoji"])),
                __props.q != null && typeof __props.q === "string" ? (_openBlock(), _createElementBlock("span", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.emojiName),
                  innerHTML: sanitizeHtml(emoji.name.replace(__props.q, `<b>${__props.q}</b>`))
                }, null, 10, ["innerHTML"])) : (_openBlock(), _createElementBlock("span", {
                  key: 1,
                  textContent: _toDisplayString(emoji.name)
                }, null, 8, ["textContent"])),
                emoji.aliasOf ? (_openBlock(), _createElementBlock(
                  "span",
                  {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.emojiAlias)
                  },
                  "(" + _toDisplayString(emoji.aliasOf) + ")",
                  3
                  /* TEXT, CLASS */
                )) : _createCommentVNode("v-if", true)
              ], 42, ["onClick"]);
            }),
            128
            /* KEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        )) : __props.type === "mfmTag" && mfmTags.value.length > 0 ? (_openBlock(), _createElementBlock(
          "ol",
          {
            key: 3,
            ref_key: "suggests",
            ref: suggests,
            class: _normalizeClass(_ctx.$style.list)
          },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(mfmTags.value, (tag) => {
              return _openBlock(), _createElementBlock("li", {
                tabindex: "-1",
                class: _normalizeClass(_ctx.$style.item),
                onClick: ($event) => complete(__props.type, tag),
                onKeydown
              }, [_createElementVNode(
                "span",
                null,
                _toDisplayString(tag),
                1
                /* TEXT */
              )], 42, ["onClick"]);
            }),
            256
            /* UNKEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        )) : __props.type === "mfmParam" && mfmParams.value.length > 0 ? (_openBlock(), _createElementBlock(
          "ol",
          {
            key: 4,
            ref_key: "suggests",
            ref: suggests,
            class: _normalizeClass(_ctx.$style.list)
          },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(mfmParams.value, (param) => {
              return _openBlock(), _createElementBlock("li", {
                tabindex: "-1",
                class: _normalizeClass(_ctx.$style.item),
                onClick: ($event) => completeMfmParam(param),
                onKeydown
              }, [_createElementVNode(
                "span",
                null,
                _toDisplayString(param),
                1
                /* TEXT */
              )], 42, ["onClick"]);
            }),
            256
            /* UNKEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true)],
        38
        /* CLASS, STYLE, NEED_HYDRATION */
      );
    };
  }
});
