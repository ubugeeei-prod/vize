import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { computed, watch, useTemplateRef, ref, onMounted, onActivated } from "vue";
import MkStreamingNotesTimeline from "@/components/MkStreamingNotesTimeline.vue";
import MkPostForm from "@/components/MkPostForm.vue";
import * as os from "@/os.js";
import { store } from "@/store.js";
import { i18n } from "@/i18n.js";
import { $i } from "@/i.js";
import { definePage } from "@/page.js";
import { antennasCache, userListsCache, favoritedChannelsCache } from "@/cache.js";
import { deviceKind } from "@/utility/device-kind.js";
import { deepMerge } from "@/utility/merge.js";
import { miLocalStorage } from "@/local-storage.js";
import { availableBasicTimelines, hasWithReplies, isAvailableBasicTimeline, isBasicTimeline, basicTimelineIconClass } from "@/timelines.js";
import { prefer } from "@/preferences.js";
export default {
  __name: "timeline",
  setup(__props) {
    const tlComponent = useTemplateRef("tlComponent");
    const srcWhenNotSignin = ref(isAvailableBasicTimeline("local") ? "local" : "global");
    const src = computed({
      get: () => $i ? store.r.tl.value.src : srcWhenNotSignin.value,
      set: (x) => saveSrc(x)
    });
    const withRenotes = computed({
      get: () => store.r.tl.value.filter.withRenotes,
      set: (x) => saveTlFilter("withRenotes", x)
    });
    // computed内での無限ループを防ぐためのフラグ
    const localSocialTLFilterSwitchStore = ref(store.r.tl.value.filter.withReplies ? "withReplies" : store.r.tl.value.filter.onlyFiles ? "onlyFiles" : false);
    const withReplies = computed({
      get: () => {
        if (!$i) return false;
        if (["local", "social"].includes(src.value) && localSocialTLFilterSwitchStore.value === "onlyFiles") {
          return false;
        } else {
          return store.r.tl.value.filter.withReplies;
        }
      },
      set: (x) => saveTlFilter("withReplies", x)
    });
    const onlyFiles = computed({
      get: () => {
        if (["local", "social"].includes(src.value) && localSocialTLFilterSwitchStore.value === "withReplies") {
          return false;
        } else {
          return store.r.tl.value.filter.onlyFiles;
        }
      },
      set: (x) => saveTlFilter("onlyFiles", x)
    });
    watch([withReplies, onlyFiles], ([withRepliesTo, onlyFilesTo]) => {
      if (withRepliesTo) {
        localSocialTLFilterSwitchStore.value = "withReplies";
      } else if (onlyFilesTo) {
        localSocialTLFilterSwitchStore.value = "onlyFiles";
      } else {
        localSocialTLFilterSwitchStore.value = false;
      }
    });
    const withSensitive = computed({
      get: () => store.r.tl.value.filter.withSensitive,
      set: (x) => saveTlFilter("withSensitive", x)
    });
    const showFixedPostForm = prefer.model("showFixedPostForm");
    async function chooseList(ev) {
      const lists = await userListsCache.fetch();
      const items = [
        ...lists.map((list) => ({
          type: "link",
          text: list.name,
          to: `/timeline/list/${list.id}`
        })),
        lists.length === 0 ? undefined : { type: "divider" },
        {
          type: "link",
          icon: "ti ti-plus",
          text: i18n.ts.createNew,
          to: "/my/lists"
        }
      ];
      os.popupMenu(items.filter((i) => i != null), ev.currentTarget ?? ev.target);
    }
    async function chooseAntenna(ev) {
      const antennas = await antennasCache.fetch();
      const items = [
        ...antennas.map((antenna) => ({
          type: "link",
          text: antenna.name,
          indicate: antenna.hasUnreadNote,
          to: `/timeline/antenna/${antenna.id}`
        })),
        antennas.length === 0 ? undefined : { type: "divider" },
        {
          type: "link",
          icon: "ti ti-plus",
          text: i18n.ts.createNew,
          to: "/my/antennas"
        }
      ];
      os.popupMenu(items.filter((i) => i != null), ev.currentTarget ?? ev.target);
    }
    async function chooseChannel(ev) {
      const channels = await favoritedChannelsCache.fetch();
      const items = [
        ...channels.map((channel) => {
          const lastReadedAt = miLocalStorage.getItemAsJson(`channelLastReadedAt:${channel.id}`) ?? null;
          const hasUnreadNote = lastReadedAt && channel.lastNotedAt ? Date.parse(channel.lastNotedAt) > lastReadedAt : !!(!lastReadedAt && channel.lastNotedAt);
          return {
            type: "link",
            text: channel.name,
            indicate: hasUnreadNote,
            to: `/channels/${channel.id}`
          };
        }),
        channels.length === 0 ? undefined : { type: "divider" },
        {
          type: "link",
          icon: "ti ti-plus",
          text: i18n.ts.createNew,
          to: "/channels/new"
        }
      ];
      os.popupMenu(items.filter((i) => i != null), ev.currentTarget ?? ev.target);
    }
    function saveSrc(newSrc) {
      const out = deepMerge({ src: newSrc }, store.s.tl);
      if (newSrc.startsWith("userList:")) {
        const id = newSrc.substring("userList:".length);
        out.userList = prefer.r.pinnedUserLists.value.find((l) => l.id === id) ?? null;
      }
      store.set("tl", out);
      if (["local", "global"].includes(newSrc)) {
        srcWhenNotSignin.value = newSrc;
      }
    }
    function saveTlFilter(key, newValue) {
      if (key !== "withReplies" || $i) {
        const out = deepMerge({ filter: { [key]: newValue } }, store.s.tl);
        store.set("tl", out);
      }
    }
    function switchTlIfNeeded() {
      if (isBasicTimeline(src.value) && !isAvailableBasicTimeline(src.value)) {
        src.value = availableBasicTimelines()[0];
      }
    }
    onMounted(() => {
      switchTlIfNeeded();
    });
    onActivated(() => {
      switchTlIfNeeded();
    });
    const headerActions = computed(() => {
      const items = [{
        icon: "ti ti-dots",
        text: i18n.ts.options,
        handler: (ev) => {
          const menuItems = [];
          menuItems.push({
            type: "switch",
            icon: "ti ti-repeat",
            text: i18n.ts.showRenotes,
            ref: withRenotes
          });
          if (isBasicTimeline(src.value) && hasWithReplies(src.value)) {
            menuItems.push({
              type: "switch",
              icon: "ti ti-messages",
              text: i18n.ts.showRepliesToOthersInTimeline,
              ref: withReplies,
              disabled: onlyFiles
            });
          }
          menuItems.push({
            type: "switch",
            icon: "ti ti-eye-exclamation",
            text: i18n.ts.withSensitive,
            ref: withSensitive
          }, {
            type: "switch",
            icon: "ti ti-photo",
            text: i18n.ts.fileAttachedOnly,
            ref: onlyFiles,
            disabled: isBasicTimeline(src.value) && hasWithReplies(src.value) ? withReplies : false
          }, { type: "divider" }, {
            type: "switch",
            text: i18n.ts.showFixedPostForm,
            ref: showFixedPostForm
          });
          os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
        }
      }];
      if (deviceKind === "desktop") {
        items.unshift({
          icon: "ti ti-refresh",
          text: i18n.ts.reload,
          handler: () => {
            tlComponent.value?.reloadTimeline();
          }
        });
      }
      return items;
    });
    const headerTabs = computed(() => [
      ...prefer.r.pinnedUserLists.value.map((l) => ({
        key: "list:" + l.id,
        title: l.name,
        icon: "ti ti-star",
        iconOnly: true
      })),
      ...availableBasicTimelines().map((tl) => ({
        key: tl,
        title: i18n.ts._timelines[tl],
        icon: basicTimelineIconClass(tl),
        iconOnly: true
      })),
      {
        icon: "ti ti-list",
        title: i18n.ts.lists,
        iconOnly: true,
        onClick: chooseList
      },
      {
        icon: "ti ti-antenna",
        title: i18n.ts.antennas,
        iconOnly: true,
        onClick: chooseAntenna
      },
      {
        icon: "ti ti-device-tv",
        title: i18n.ts.channel,
        iconOnly: true,
        onClick: chooseChannel
      }
    ]);
    const headerTabsWhenNotLogin = computed(() => [...availableBasicTimelines().map((tl) => ({
      key: tl,
      title: i18n.ts._timelines[tl],
      icon: basicTimelineIconClass(tl),
      iconOnly: true
    }))]);
    definePage(() => ({
      title: i18n.ts.timeline,
      icon: isBasicTimeline(src.value) ? basicTimelineIconClass(src.value) : "ti ti-home"
    }));
    return (_ctx, _cache) => {
      const _component_MkTip = _resolveComponent("MkTip");
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: _unref($i) ? headerTabs.value : headerTabsWhenNotLogin.value,
        swipable: true,
        displayMyAvatar: true,
        canOmitTitle: true,
        tab: src.value,
        "onUpdate:tab": _cache[0] || (_cache[0] = ($event) => src.value = $event)
      }, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [
          _unref(isBasicTimeline)(src.value) ? (_openBlock(), _createBlock(_component_MkTip, {
            key: 0,
            k: `tl.${src.value}`,
            style: "margin-bottom: var(--MI-margin);"
          }, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._timelineDescription[src.value]),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["k"])) : _createCommentVNode("v-if", true),
          _unref(prefer).r.showFixedPostForm.value ? (_openBlock(), _createBlock(
            MkPostForm,
            {
              key: 0,
              class: _normalizeClass(["_panel", _ctx.$style.postForm]),
              fixed: "",
              style: "margin-bottom: var(--MI-margin);"
            },
            null,
            2
            /* CLASS */
          )) : _createCommentVNode("v-if", true),
          _createVNode(MkStreamingNotesTimeline, {
            ref_key: "tlComponent",
            ref: tlComponent,
            key: src.value + withRenotes.value + withReplies.value + onlyFiles.value + withSensitive.value,
            class: _normalizeClass(_ctx.$style.tl),
            src: src.value.split(":")[0],
            list: src.value.split(":")[1],
            withRenotes: withRenotes.value,
            withReplies: withReplies.value,
            withSensitive: withSensitive.value,
            onlyFiles: onlyFiles.value,
            sound: true
          }, null, 10, [
            "src",
            "list",
            "withRenotes",
            "withReplies",
            "withSensitive",
            "onlyFiles",
            "sound"
          ])
        ])]),
        _: 1
      }, 8, [
        "actions",
        "tabs",
        "swipable",
        "displayMyAvatar",
        "canOmitTitle",
        "tab"
      ]);
    };
  }
};
