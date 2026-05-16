import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, createSlots as _createSlots, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-settings" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-bolt" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("br");
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-player-play" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-player-track-next" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-notes" });
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-trash" });
const _hoisted_8 = /* @__PURE__ */ _createElementVNode("hr");
const _hoisted_9 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-edit" });
const _hoisted_10 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-bell" });
const _hoisted_11 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-messages" });
const _hoisted_12 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-accessible" });
const _hoisted_13 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-battery-vertical-eco" });
const _hoisted_14 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-antenna-bars-3" });
const _hoisted_15 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-settings-cog" });
const _hoisted_16 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-trash" });
const _hoisted_17 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-download" });
const _hoisted_18 = /* @__PURE__ */ _createElementVNode("hr");
const _hoisted_19 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-list" });
const _hoisted_20 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-list" });
const _hoisted_21 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-columns" });
const _hoisted_22 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-code" });
import { computed, ref, watch } from "vue";
import { langs } from "@@/js/config.js";
import MkSwitch from "@/components/MkSwitch.vue";
import MkSelect from "@/components/MkSelect.vue";
import MkRadios from "@/components/MkRadios.vue";
import MkRange from "@/components/MkRange.vue";
import MkFolder from "@/components/MkFolder.vue";
import MkButton from "@/components/MkButton.vue";
import MkDisableSection from "@/components/MkDisableSection.vue";
import FormLink from "@/components/form/link.vue";
import MkLink from "@/components/MkLink.vue";
import MkInfo from "@/components/MkInfo.vue";
import { store } from "@/store.js";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import { miLocalStorage } from "@/local-storage.js";
import { prefer } from "@/preferences.js";
import MkPreferenceContainer from "@/components/MkPreferenceContainer.vue";
import MkFeatureBanner from "@/components/MkFeatureBanner.vue";
import { globalEvents } from "@/events.js";
import { claimAchievement } from "@/utility/achievements.js";
import { instance } from "@/instance.js";
import { ensureSignin } from "@/i.js";
import { genId } from "@/utility/id.js";
import { suggestReload } from "@/utility/reload-suggest.js";
export default {
  __name: "preferences",
  setup(__props) {
    const $i = ensureSignin();
    const lang = ref(miLocalStorage.getItem("lang"));
    const dataSaver = ref(prefer.s.dataSaver);
    const realtimeMode = store.model("realtimeMode");
    const overridedDeviceKind = prefer.model("overridedDeviceKind");
    const pollingInterval = prefer.model("pollingInterval");
    const showTitlebar = prefer.model("showTitlebar");
    const keepCw = prefer.model("keepCw");
    const serverDisconnectedBehavior = prefer.model("serverDisconnectedBehavior");
    const hemisphere = prefer.model("hemisphere");
    const showNoteActionsOnlyHover = prefer.model("showNoteActionsOnlyHover");
    const showClipButtonInNoteFooter = prefer.model("showClipButtonInNoteFooter");
    const collapseRenotes = prefer.model("collapseRenotes");
    const advancedMfm = prefer.model("advancedMfm");
    const showReactionsCount = prefer.model("showReactionsCount");
    const enableQuickAddMfmFunction = prefer.model("enableQuickAddMfmFunction");
    const forceShowAds = prefer.model("forceShowAds");
    const loadRawImages = prefer.model("loadRawImages");
    const imageNewTab = prefer.model("imageNewTab");
    const showFixedPostForm = prefer.model("showFixedPostForm");
    const showFixedPostFormInChannel = prefer.model("showFixedPostFormInChannel");
    const numberOfPageCache = prefer.model("numberOfPageCache");
    const enableInfiniteScroll = prefer.model("enableInfiniteScroll");
    const useReactionPickerForContextMenu = prefer.model("useReactionPickerForContextMenu");
    const showAvailableReactionsFirstInNote = prefer.model("showAvailableReactionsFirstInNote");
    const useGroupedNotifications = prefer.model("useGroupedNotifications");
    const alwaysConfirmFollow = prefer.model("alwaysConfirmFollow");
    const confirmWhenRevealingSensitiveMedia = prefer.model("confirmWhenRevealingSensitiveMedia");
    const confirmOnReact = prefer.model("confirmOnReact");
    const defaultNoteVisibility = prefer.model("defaultNoteVisibility");
    const defaultNoteLocalOnly = prefer.model("defaultNoteLocalOnly");
    const rememberNoteVisibility = prefer.model("rememberNoteVisibility");
    const notificationPosition = prefer.model("notificationPosition");
    const notificationStackAxis = prefer.model("notificationStackAxis");
    const instanceTicker = prefer.model("instanceTicker");
    const highlightSensitiveMedia = prefer.model("highlightSensitiveMedia");
    const mediaListWithOneImageAppearance = prefer.model("mediaListWithOneImageAppearance");
    const showMediaListByGridInWideArea = prefer.model("showMediaListByGridInWideArea");
    const reactionsDisplaySize = prefer.model("reactionsDisplaySize");
    const limitWidthOfReaction = prefer.model("limitWidthOfReaction");
    const squareAvatars = prefer.model("squareAvatars");
    const enableSeasonalScreenEffect = prefer.model("enableSeasonalScreenEffect");
    const showAvatarDecorations = prefer.model("showAvatarDecorations");
    const nsfw = prefer.model("nsfw");
    const emojiStyle = prefer.model("emojiStyle");
    const useBlurEffectForModal = prefer.model("useBlurEffectForModal");
    const useBlurEffect = prefer.model("useBlurEffect");
    const defaultFollowWithReplies = prefer.model("defaultFollowWithReplies");
    const chatShowSenderName = prefer.model("chat.showSenderName");
    const chatSendOnEnter = prefer.model("chat.sendOnEnter");
    const useStickyIcons = prefer.model("useStickyIcons");
    const enableHighQualityImagePlaceholders = prefer.model("enableHighQualityImagePlaceholders");
    const reduceAnimation = prefer.model("animation", (v) => !v, (v) => !v);
    const animatedMfm = prefer.model("animatedMfm");
    const disableShowingAnimatedImages = prefer.model("disableShowingAnimatedImages");
    const keepScreenOn = prefer.model("keepScreenOn");
    const enableHorizontalSwipe = prefer.model("enableHorizontalSwipe");
    const showPageTabBarBottom = prefer.model("showPageTabBarBottom");
    const enablePullToRefresh = prefer.model("enablePullToRefresh");
    const useNativeUiForVideoAudioPlayer = prefer.model("useNativeUiForVideoAudioPlayer");
    const contextMenu = prefer.model("contextMenu");
    const menuStyle = prefer.model("menuStyle");
    const makeEveryTextElementsSelectable = prefer.model("makeEveryTextElementsSelectable");
    const fontSize = ref(miLocalStorage.getItem("fontSize"));
    const useSystemFont = ref(miLocalStorage.getItem("useSystemFont") != null);
    watch(lang, () => {
      miLocalStorage.setItem("lang", lang.value);
    });
    watch(fontSize, () => {
      if (fontSize.value == null) {
        miLocalStorage.removeItem("fontSize");
      } else {
        miLocalStorage.setItem("fontSize", fontSize.value);
      }
    });
    watch(useSystemFont, () => {
      if (useSystemFont.value) {
        miLocalStorage.setItem("useSystemFont", "t");
      } else {
        miLocalStorage.removeItem("useSystemFont");
      }
    });
    watch([
      hemisphere,
      lang,
      realtimeMode,
      pollingInterval,
      enableInfiniteScroll,
      showNoteActionsOnlyHover,
      overridedDeviceKind,
      alwaysConfirmFollow,
      confirmWhenRevealingSensitiveMedia,
      mediaListWithOneImageAppearance,
      reactionsDisplaySize,
      limitWidthOfReaction,
      mediaListWithOneImageAppearance,
      limitWidthOfReaction,
      instanceTicker,
      squareAvatars,
      highlightSensitiveMedia,
      enableSeasonalScreenEffect,
      chatShowSenderName,
      useStickyIcons,
      enableHighQualityImagePlaceholders,
      disableShowingAnimatedImages,
      keepScreenOn,
      contextMenu,
      fontSize,
      useSystemFont,
      makeEveryTextElementsSelectable,
      enableHorizontalSwipe,
      showPageTabBarBottom,
      enablePullToRefresh,
      reduceAnimation,
      showAvailableReactionsFirstInNote,
      animatedMfm,
      advancedMfm
    ], () => {
      suggestReload();
    });
    const emojiIndexLangs = [
      "en-US",
      "ja-JP",
      "ja-JP_hira"
    ];
    function getEmojiIndexLangName(targetLang) {
      if (langs.find((x) => x[0] === targetLang)) {
        return langs.find((x) => x[0] === targetLang)[1];
      } else {
        // 絵文字辞書限定の言語定義
        switch (targetLang) {
          case "ja-JP_hira": return "ひらがな";
          default: return targetLang;
        }
      }
    }
    function downloadEmojiIndex(lang) {
      async function main() {
        const currentIndexes = store.s.additionalUnicodeEmojiIndexes;
        function download() {
          switch (lang) {
            case "en-US": return import("../../unicode-emoji-indexes/en-US.json").then((x) => x.default);
            case "ja-JP": return import("../../unicode-emoji-indexes/ja-JP.json").then((x) => x.default);
            case "ja-JP_hira": return import("../../unicode-emoji-indexes/ja-JP_hira.json").then((x) => x.default);
            default: throw new Error("unrecognized lang: " + lang);
          }
        }
        currentIndexes[lang] = await download();
        await store.set("additionalUnicodeEmojiIndexes", currentIndexes);
      }
      os.promiseDialog(main());
    }
    function removeEmojiIndex(lang) {
      async function main() {
        const currentIndexes = store.s.additionalUnicodeEmojiIndexes;
        delete currentIndexes[lang];
        await store.set("additionalUnicodeEmojiIndexes", currentIndexes);
      }
      os.promiseDialog(main());
    }
    async function setPinnedList() {
      const lists = await misskeyApi("users/lists/list");
      const { canceled, result: listId } = await os.select({
        title: i18n.ts.selectList,
        items: lists.map((x) => ({
          value: x.id,
          label: x.name
        }))
      });
      if (canceled || listId == null) return;
      prefer.commit("pinnedUserLists", [lists.find((x) => x.id === listId)]);
    }
    function removePinnedList() {
      prefer.commit("pinnedUserLists", []);
    }
    function enableAllDataSaver() {
      const g = { ...prefer.s.dataSaver };
      Object.keys(g).forEach((key) => {
        g[key] = true;
      });
      dataSaver.value = g;
    }
    function disableAllDataSaver() {
      const g = { ...prefer.s.dataSaver };
      Object.keys(g).forEach((key) => {
        g[key] = false;
      });
      dataSaver.value = g;
    }
    watch(dataSaver, (to) => {
      prefer.commit("dataSaver", to);
    }, { deep: true });
    let smashCount = 0;
    let smashTimer = null;
    function testNotification() {
      const notification = {
        id: genId(),
        createdAt: new Date().toUTCString(),
        type: "test"
      };
      globalEvents.emit("clientNotification", notification);
      // セルフ通知破壊 実績関連
      smashCount++;
      if (smashCount >= 10) {
        claimAchievement("smashTestNotificationButton");
        smashCount = 0;
      }
      if (smashTimer) {
        window.clearTimeout(smashTimer);
      }
      smashTimer = window.setTimeout(() => {
        smashCount = 0;
      }, 300);
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.general,
      icon: "ti ti-adjustments"
    }));
    return (_ctx, _cache) => {
      const _component_SearchText = _resolveComponent("SearchText");
      const _component_SearchLabel = _resolveComponent("SearchLabel");
      const _component_SearchIcon = _resolveComponent("SearchIcon");
      const _component_I18n = _resolveComponent("I18n");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      const _component_Mfm = _resolveComponent("Mfm");
      return _openBlock(), _createBlock(_component_SearchMarker, {
        path: "/settings/preferences",
        label: _unref(i18n).ts.preferences,
        keywords: ["general", "preferences"],
        icon: "ti ti-adjustments"
      }, {
        default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/gear_3d.png",
            color: "#00ff9d"
          }, {
            default: _withCtx(() => [_createVNode(_component_SearchText, null, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._settings.preferencesBanner),
                1
                /* TEXT */
              )]),
              _: 1
            })]),
            _: 1
          }),
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(_component_SearchMarker, { keywords: ["general"] }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.general),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_1]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
                  _createVNode(_component_SearchMarker, { keywords: ["language"] }, {
                    default: _withCtx(() => [_createVNode(MkSelect, {
                      items: _unref(langs).map((x) => ({
                        label: x[1],
                        value: x[0]
                      })),
                      modelValue: lang.value,
                      "onUpdate:modelValue": ($event) => lang.value = $event
                    }, {
                      label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts.uiLanguage),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      })]),
                      caption: _withCtx(() => [_createVNode(_component_I18n, {
                        src: _unref(i18n).ts.i18nInfo,
                        tag: "span"
                      }, {
                        link: _withCtx(() => [_createVNode(MkLink, { url: "https://crowdin.com/project/misskey" }, {
                          default: _withCtx(() => [_createTextVNode("Crowdin")]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["src"])]),
                      _: 1
                    }, 8, [
                      "items",
                      "modelValue",
                      "onUpdate:modelValue"
                    ])]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "device",
                    "type",
                    "kind",
                    "smartphone",
                    "tablet",
                    "desktop"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkRadios, {
                      options: [
                        {
                          value: null,
                          label: _unref(i18n).ts.auto
                        },
                        {
                          value: "smartphone",
                          label: _unref(i18n).ts.smartphone,
                          icon: "ti ti-device-mobile"
                        },
                        {
                          value: "tablet",
                          label: _unref(i18n).ts.tablet,
                          icon: "ti ti-device-tablet"
                        },
                        {
                          value: "desktop",
                          label: _unref(i18n).ts.desktop,
                          icon: "ti ti-device-desktop"
                        }
                      ],
                      modelValue: _unref(overridedDeviceKind),
                      "onUpdate:modelValue": ($event) => overridedDeviceKind.value = $event
                    }, {
                      label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts.overridedDeviceKind),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, [
                      "options",
                      "modelValue",
                      "onUpdate:modelValue"
                    ])]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: ["realtimemode"] }, {
                    default: _withCtx(() => [_createVNode(MkSwitch, {
                      modelValue: _unref(realtimeMode),
                      "onUpdate:modelValue": ($event) => realtimeMode.value = $event
                    }, {
                      label: _withCtx(() => [
                        _hoisted_2,
                        _createTextVNode(" "),
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.realtimeMode),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })
                      ]),
                      caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                        default: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts._settings.realtimeMode_description),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["modelValue", "onUpdate:modelValue"])]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(MkDisableSection, { disabled: _unref(realtimeMode) }, {
                    default: _withCtx(() => [_createVNode(_component_SearchMarker, { keywords: ["polling", "interval"] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "pollingInterval" }, {
                        default: _withCtx(() => [_createVNode(MkRange, {
                          min: 1,
                          max: 3,
                          step: 1,
                          easing: "",
                          showTicks: true,
                          textConverter: (v) => v === 1 ? _unref(i18n).ts.low : v === 2 ? _unref(i18n).ts.middle : v === 3 ? _unref(i18n).ts.high : "",
                          modelValue: _unref(pollingInterval),
                          "onUpdate:modelValue": ($event) => pollingInterval.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts._settings.contentsUpdateFrequency),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          caption: _withCtx(() => [
                            _createVNode(_component_SearchText, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts._settings.contentsUpdateFrequency_description),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            }),
                            _hoisted_3,
                            _createVNode(_component_SearchText, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts._settings.contentsUpdateFrequency_description2),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })
                          ]),
                          prefix: _withCtx(() => [_hoisted_4]),
                          suffix: _withCtx(() => [_hoisted_5]),
                          _: 1
                        }, 8, [
                          "min",
                          "max",
                          "step",
                          "showTicks",
                          "textConverter",
                          "modelValue",
                          "onUpdate:modelValue"
                        ])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"])]),
                    _: 1
                  }, 8, ["disabled"]),
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    _createVNode(_component_SearchMarker, { keywords: ["titlebar", "show"] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showTitlebar" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(showTitlebar),
                          "onUpdate:modelValue": ($event) => showTitlebar.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.showTitlebar),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "avatar",
                      "icon",
                      "decoration",
                      "show"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showAvatarDecorations" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(showAvatarDecorations),
                          "onUpdate:modelValue": ($event) => showAvatarDecorations.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.showAvatarDecorations),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "follow",
                      "confirm",
                      "always"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "alwaysConfirmFollow" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(alwaysConfirmFollow),
                          "onUpdate:modelValue": ($event) => alwaysConfirmFollow.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.alwaysConfirmFollow),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "highlight",
                      "sensitive",
                      "nsfw",
                      "image",
                      "photo",
                      "picture",
                      "media",
                      "thumbnail"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "highlightSensitiveMedia" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(highlightSensitiveMedia),
                          "onUpdate:modelValue": ($event) => highlightSensitiveMedia.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.highlightSensitiveMedia),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "sensitive",
                      "nsfw",
                      "media",
                      "image",
                      "photo",
                      "picture",
                      "attachment",
                      "confirm"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "confirmWhenRevealingSensitiveMedia" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(confirmWhenRevealingSensitiveMedia),
                          "onUpdate:modelValue": ($event) => confirmWhenRevealingSensitiveMedia.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.confirmWhenRevealingSensitiveMedia),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "mfm",
                      "enable",
                      "show",
                      "advanced"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "advancedMfm" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(advancedMfm),
                          "onUpdate:modelValue": ($event) => advancedMfm.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.enableAdvancedMfm),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "auto",
                      "load",
                      "auto",
                      "more",
                      "scroll"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "enableInfiniteScroll" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(enableInfiniteScroll),
                          "onUpdate:modelValue": ($event) => enableInfiniteScroll.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.enableInfiniteScroll),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"])
                  ]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "emoji",
                    "style",
                    "native",
                    "system",
                    "fluent",
                    "twemoji"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "emojiStyle" }, {
                      default: _withCtx(() => [_createElementVNode("div", null, [_createVNode(MkRadios, {
                        options: [
                          {
                            value: "native",
                            label: _unref(i18n).ts.native
                          },
                          {
                            value: "fluentEmoji",
                            label: "Fluent Emoji"
                          },
                          {
                            value: "twemoji",
                            label: "Twemoji"
                          }
                        ],
                        modelValue: _unref(emojiStyle),
                        "onUpdate:modelValue": ($event) => emojiStyle.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.emojiStyle),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, [
                        "options",
                        "modelValue",
                        "onUpdate:modelValue"
                      ]), _createElementVNode("div", { style: "margin: 8px 0 0 0; font-size: 1.5em;" }, [_createVNode(_component_Mfm, {
                        key: _unref(emojiStyle),
                        text: "🍮🍦🍭🍩🍰🍫🍬🥞🍪"
                      })])])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"])
                ])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["timeline", "note"] }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._settings.timelineAndNote),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_6]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    _createVNode(_component_SearchMarker, { keywords: [
                      "post",
                      "form",
                      "timeline"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showFixedPostForm" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(showFixedPostForm),
                          "onUpdate:modelValue": ($event) => showFixedPostForm.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.showFixedPostForm),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "post",
                      "form",
                      "timeline",
                      "channel"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showFixedPostFormInChannel" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(showFixedPostFormInChannel),
                          "onUpdate:modelValue": ($event) => showFixedPostFormInChannel.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.showFixedPostFormInChannel),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ["renote"] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "collapseRenotes" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(collapseRenotes),
                          "onUpdate:modelValue": ($event) => collapseRenotes.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.collapseRenotes),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.collapseRenotesDescription),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ["pinned", "list"] }, {
                      default: _withCtx(() => [_createVNode(
                        MkFolder,
                        null,
                        {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.pinnedList),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          default: _withCtx(() => [_unref(prefer).r.pinnedUserLists.value.length === 0 ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            onClick: ($event) => setPinnedList()
                          }, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.add),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          }, 8, ["onClick"])) : (_openBlock(), _createBlock(MkButton, {
                            key: 1,
                            danger: "",
                            onClick: ($event) => removePinnedList()
                          }, {
                            default: _withCtx(() => [
                              _hoisted_7,
                              _createTextVNode(" "),
                              _createTextVNode(
                                _toDisplayString(_unref(i18n).ts.remove),
                                1
                                /* TEXT */
                              )
                            ]),
                            _: 1
                          }, 8, ["onClick"]))]),
                          _: 2
                        },
                        1024
                        /* DYNAMIC_SLOTS */
                      )]),
                      _: 1
                    }, 8, ["keywords"])
                  ]),
                  _hoisted_8,
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      _createVNode(_component_SearchMarker, { keywords: [
                        "hover",
                        "show",
                        "footer",
                        "action"
                      ] }, {
                        default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showNoteActionsOnlyHover" }, {
                          default: _withCtx(() => [_createVNode(MkSwitch, {
                            modelValue: _unref(showNoteActionsOnlyHover),
                            "onUpdate:modelValue": ($event) => showNoteActionsOnlyHover.value = $event
                          }, {
                            label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts.showNoteActionsOnlyHover),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })]),
                            _: 1
                          }, 8, ["modelValue", "onUpdate:modelValue"])]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["keywords"]),
                      _createVNode(_component_SearchMarker, { keywords: [
                        "footer",
                        "action",
                        "clip",
                        "show"
                      ] }, {
                        default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showClipButtonInNoteFooter" }, {
                          default: _withCtx(() => [_createVNode(MkSwitch, {
                            modelValue: _unref(showClipButtonInNoteFooter),
                            "onUpdate:modelValue": ($event) => showClipButtonInNoteFooter.value = $event
                          }, {
                            label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts.showClipButtonInNoteFooter),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })]),
                            _: 1
                          }, 8, ["modelValue", "onUpdate:modelValue"])]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["keywords"]),
                      _createVNode(_component_SearchMarker, { keywords: [
                        "reaction",
                        "count",
                        "show"
                      ] }, {
                        default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showReactionsCount" }, {
                          default: _withCtx(() => [_createVNode(MkSwitch, {
                            modelValue: _unref(showReactionsCount),
                            "onUpdate:modelValue": ($event) => showReactionsCount.value = $event
                          }, {
                            label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts.showReactionsCount),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })]),
                            _: 1
                          }, 8, ["modelValue", "onUpdate:modelValue"])]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["keywords"]),
                      _createVNode(_component_SearchMarker, { keywords: ["reaction", "confirm"] }, {
                        default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "confirmOnReact" }, {
                          default: _withCtx(() => [_createVNode(MkSwitch, {
                            modelValue: _unref(confirmOnReact),
                            "onUpdate:modelValue": ($event) => confirmOnReact.value = $event
                          }, {
                            label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts.confirmOnReact),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })]),
                            _: 1
                          }, 8, ["modelValue", "onUpdate:modelValue"])]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["keywords"]),
                      _createVNode(_component_SearchMarker, { keywords: [
                        "image",
                        "photo",
                        "picture",
                        "media",
                        "thumbnail",
                        "quality",
                        "raw",
                        "attachment"
                      ] }, {
                        default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "loadRawImages" }, {
                          default: _withCtx(() => [_createVNode(MkSwitch, {
                            modelValue: _unref(loadRawImages),
                            "onUpdate:modelValue": ($event) => loadRawImages.value = $event
                          }, {
                            label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts.loadRawImages),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })]),
                            _: 1
                          }, 8, ["modelValue", "onUpdate:modelValue"])]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["keywords"]),
                      _createVNode(_component_SearchMarker, { keywords: [
                        "reaction",
                        "picker",
                        "contextmenu",
                        "open"
                      ] }, {
                        default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "useReactionPickerForContextMenu" }, {
                          default: _withCtx(() => [_createVNode(MkSwitch, {
                            modelValue: _unref(useReactionPickerForContextMenu),
                            "onUpdate:modelValue": ($event) => useReactionPickerForContextMenu.value = $event
                          }, {
                            label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts.useReactionPickerForContextMenu),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })]),
                            _: 1
                          }, 8, ["modelValue", "onUpdate:modelValue"])]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["keywords"]),
                      _createVNode(_component_SearchMarker, { keywords: ["reaction", "order"] }, {
                        default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showAvailableReactionsFirstInNote" }, {
                          default: _withCtx(() => [_createVNode(MkSwitch, {
                            modelValue: _unref(showAvailableReactionsFirstInNote),
                            "onUpdate:modelValue": ($event) => showAvailableReactionsFirstInNote.value = $event
                          }, {
                            label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts._settings.showAvailableReactionsFirstInNote),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })]),
                            _: 1
                          }, 8, ["modelValue", "onUpdate:modelValue"])]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["keywords"])
                    ]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "reaction",
                      "size",
                      "scale",
                      "display"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "reactionsDisplaySize" }, {
                        default: _withCtx(() => [_createVNode(MkRadios, {
                          options: [
                            {
                              value: "small",
                              label: _unref(i18n).ts.small
                            },
                            {
                              value: "medium",
                              label: _unref(i18n).ts.medium
                            },
                            {
                              value: "large",
                              label: _unref(i18n).ts.large
                            }
                          ],
                          modelValue: _unref(reactionsDisplaySize),
                          "onUpdate:modelValue": ($event) => reactionsDisplaySize.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.reactionsDisplaySize),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, [
                          "options",
                          "modelValue",
                          "onUpdate:modelValue"
                        ])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "reaction",
                      "size",
                      "scale",
                      "display",
                      "width",
                      "limit"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "limitWidthOfReaction" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(limitWidthOfReaction),
                          "onUpdate:modelValue": ($event) => limitWidthOfReaction.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.limitWidthOfReaction),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "attachment",
                      "image",
                      "photo",
                      "picture",
                      "media",
                      "thumbnail",
                      "list",
                      "size",
                      "height"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "mediaListWithOneImageAppearance" }, {
                        default: _withCtx(() => [_createVNode(MkRadios, {
                          options: [
                            {
                              value: "expand",
                              label: _unref(i18n).ts.default
                            },
                            {
                              value: "16_9",
                              label: _unref(i18n).tsx.limitTo({ x: "16:9" })
                            },
                            {
                              value: "1_1",
                              label: _unref(i18n).tsx.limitTo({ x: "1:1" })
                            },
                            {
                              value: "2_3",
                              label: _unref(i18n).tsx.limitTo({ x: "2:3" })
                            }
                          ],
                          modelValue: _unref(mediaListWithOneImageAppearance),
                          "onUpdate:modelValue": ($event) => mediaListWithOneImageAppearance.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.mediaListWithOneImageAppearance),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, [
                          "options",
                          "modelValue",
                          "onUpdate:modelValue"
                        ])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "attachment",
                      "image",
                      "photo",
                      "picture",
                      "media",
                      "thumbnail",
                      "grid",
                      "wide",
                      "area"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showMediaListByGridInWideArea" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(showMediaListByGridInWideArea),
                          "onUpdate:modelValue": ($event) => showMediaListByGridInWideArea.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.showMediaListByGridInWideArea),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "ticker",
                      "information",
                      "label",
                      "instance",
                      "server",
                      "host",
                      "federation"
                    ] }, {
                      default: _withCtx(() => [_createVNode(
                        MkPreferenceContainer,
                        { k: "instanceTicker" },
                        {
                          default: _withCtx(() => [_unref(instance).federation !== "none" ? (_openBlock(), _createBlock(MkSelect, {
                            key: 0,
                            items: [
                              {
                                label: _unref(i18n).ts._instanceTicker.none,
                                value: "none"
                              },
                              {
                                label: _unref(i18n).ts._instanceTicker.remote,
                                value: "remote"
                              },
                              {
                                label: _unref(i18n).ts._instanceTicker.always,
                                value: "always"
                              }
                            ],
                            modelValue: _unref(instanceTicker),
                            "onUpdate:modelValue": ($event) => instanceTicker.value = $event
                          }, {
                            label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [_createTextVNode(
                                _toDisplayString(_unref(i18n).ts.instanceTicker),
                                1
                                /* TEXT */
                              )]),
                              _: 1
                            })]),
                            _: 1
                          }, 8, [
                            "items",
                            "modelValue",
                            "onUpdate:modelValue"
                          ])) : _createCommentVNode("v-if", true)]),
                          _: 2
                        },
                        1024
                        /* DYNAMIC_SLOTS */
                      )]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "attachment",
                      "image",
                      "photo",
                      "picture",
                      "media",
                      "thumbnail",
                      "nsfw",
                      "sensitive",
                      "display",
                      "show",
                      "hide",
                      "visibility"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "nsfw" }, {
                        default: _withCtx(() => [_createVNode(MkSelect, {
                          items: [
                            {
                              label: _unref(i18n).ts._displayOfSensitiveMedia.respect,
                              value: "respect"
                            },
                            {
                              label: _unref(i18n).ts._displayOfSensitiveMedia.ignore,
                              value: "ignore"
                            },
                            {
                              label: _unref(i18n).ts._displayOfSensitiveMedia.force,
                              value: "force"
                            }
                          ],
                          modelValue: _unref(nsfw),
                          "onUpdate:modelValue": ($event) => nsfw.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.displayOfSensitiveMedia),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, [
                          "items",
                          "modelValue",
                          "onUpdate:modelValue"
                        ])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"])
                  ])
                ])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["post", "form"] }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.postForm),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_9]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [_createElementVNode("div", { class: "_gaps_s" }, [
                  _createVNode(_component_SearchMarker, { keywords: [
                    "remember",
                    "keep",
                    "note",
                    "cw"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "keepCw" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(keepCw),
                        "onUpdate:modelValue": ($event) => keepCw.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.keepCw),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "remember",
                    "keep",
                    "note",
                    "visibility"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "rememberNoteVisibility" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(rememberNoteVisibility),
                        "onUpdate:modelValue": ($event) => rememberNoteVisibility.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.rememberNoteVisibility),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "mfm",
                    "enable",
                    "show",
                    "advanced",
                    "picker",
                    "form",
                    "function",
                    "fn"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "enableQuickAddMfmFunction" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(enableQuickAddMfmFunction),
                        "onUpdate:modelValue": ($event) => enableQuickAddMfmFunction.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.enableQuickAddMfmFunction),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"])
                ]), _createVNode(_component_SearchMarker, { keywords: [
                  "default",
                  "note",
                  "visibility"
                ] }, {
                  default: _withCtx(() => [_createVNode(MkDisableSection, { disabled: _unref(rememberNoteVisibility) }, {
                    default: _withCtx(() => [_createVNode(
                      MkFolder,
                      null,
                      _createSlots({ _: 2 }, [{
                        name: "label",
                        fn: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.defaultNoteVisibility),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })])
                      }, _unref(defaultNoteVisibility) === "public" ? {
                        name: "suffix",
                        fn: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts._visibility.public),
                          1
                          /* TEXT */
                        )]),
                        key: "0"
                      } : _unref(defaultNoteVisibility) === "home" ? {
                        name: "suffix",
                        fn: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts._visibility.home),
                          1
                          /* TEXT */
                        )]),
                        key: "1"
                      } : _unref(defaultNoteVisibility) === "followers" ? {
                        name: "suffix",
                        fn: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts._visibility.followers),
                          1
                          /* TEXT */
                        )]),
                        key: "2"
                      } : _unref(defaultNoteVisibility) === "specified" ? {
                        name: "suffix",
                        fn: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts._visibility.specified),
                          1
                          /* TEXT */
                        )]),
                        key: "3"
                      } : undefined]),
                      1024
                      /* DYNAMIC_SLOTS */
                    )]),
                    _: 1
                  }, 8, ["disabled"])]),
                  _: 1
                }, 8, ["keywords"])])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["notification"] }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.notifications),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_10]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
                  _createVNode(_component_SearchMarker, { keywords: ["group"] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "useGroupedNotifications" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(useGroupedNotifications),
                        "onUpdate:modelValue": ($event) => useGroupedNotifications.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.useGroupedNotifications),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: ["position"] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "notificationPosition" }, {
                      default: _withCtx(() => [_createVNode(MkRadios, {
                        options: [
                          {
                            value: "leftTop",
                            label: _unref(i18n).ts.leftTop,
                            icon: "ti ti-align-box-left-top"
                          },
                          {
                            value: "rightTop",
                            label: _unref(i18n).ts.rightTop,
                            icon: "ti ti-align-box-right-top"
                          },
                          {
                            value: "leftBottom",
                            label: _unref(i18n).ts.leftBottom,
                            icon: "ti ti-align-box-left-bottom"
                          },
                          {
                            value: "rightBottom",
                            label: _unref(i18n).ts.rightBottom,
                            icon: "ti ti-align-box-right-bottom"
                          }
                        ],
                        modelValue: _unref(notificationPosition),
                        "onUpdate:modelValue": ($event) => notificationPosition.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.position),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, [
                        "options",
                        "modelValue",
                        "onUpdate:modelValue"
                      ])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "stack",
                    "axis",
                    "direction"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "notificationStackAxis" }, {
                      default: _withCtx(() => [_createVNode(MkRadios, {
                        options: [{
                          value: "vertical",
                          label: _unref(i18n).ts.vertical,
                          icon: "ti ti-carousel-vertical"
                        }, {
                          value: "horizontal",
                          label: _unref(i18n).ts.horizontal,
                          icon: "ti ti-carousel-horizontal"
                        }],
                        modelValue: _unref(notificationStackAxis),
                        "onUpdate:modelValue": ($event) => notificationStackAxis.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.stackAxis),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, [
                        "options",
                        "modelValue",
                        "onUpdate:modelValue"
                      ])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(MkButton, { onClick: testNotification }, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts._notification.checkNotificationBehavior),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })
                ])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"]),
            _unref($i).policies.chatAvailability !== "unavailable" ? (_openBlock(), _createBlock(_component_SearchMarker, {
              key: 0,
              keywords: ["chat", "messaging"]
            }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.directMessage),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_11]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [_createVNode(_component_SearchMarker, { keywords: [
                  "show",
                  "sender",
                  "name"
                ] }, {
                  default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "chat.showSenderName" }, {
                    default: _withCtx(() => [_createVNode(MkSwitch, {
                      modelValue: _unref(chatShowSenderName),
                      "onUpdate:modelValue": ($event) => chatShowSenderName.value = $event
                    }, {
                      label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts._settings._chat.showSenderName),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["modelValue", "onUpdate:modelValue"])]),
                    _: 1
                  })]),
                  _: 1
                }, 8, ["keywords"]), _createVNode(_component_SearchMarker, { keywords: [
                  "send",
                  "enter",
                  "newline"
                ] }, {
                  default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "chat.sendOnEnter" }, {
                    default: _withCtx(() => [_createVNode(MkSwitch, {
                      modelValue: _unref(chatSendOnEnter),
                      "onUpdate:modelValue": ($event) => chatSendOnEnter.value = $event
                    }, {
                      label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts._settings._chat.sendOnEnter),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      })]),
                      caption: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [_createElementVNode("div", null, [
                        _createElementVNode(
                          "b",
                          null,
                          _toDisplayString(_unref(i18n).ts._settings.ifOn) + ":",
                          1
                          /* TEXT */
                        ),
                        _createElementVNode(
                          "div",
                          null,
                          _toDisplayString(_unref(i18n).ts._chat.send) + ": Enter",
                          1
                          /* TEXT */
                        ),
                        _createElementVNode(
                          "div",
                          null,
                          _toDisplayString(_unref(i18n).ts._chat.newline) + ": Shift + Enter",
                          1
                          /* TEXT */
                        )
                      ]), _createElementVNode("div", null, [
                        _createElementVNode(
                          "b",
                          null,
                          _toDisplayString(_unref(i18n).ts._settings.ifOff) + ":",
                          1
                          /* TEXT */
                        ),
                        _createElementVNode(
                          "div",
                          null,
                          _toDisplayString(_unref(i18n).ts._chat.send) + ": Ctrl + Enter",
                          1
                          /* TEXT */
                        ),
                        _createElementVNode(
                          "div",
                          null,
                          _toDisplayString(_unref(i18n).ts._chat.newline) + ": Enter",
                          1
                          /* TEXT */
                        )
                      ])])]),
                      _: 1
                    }, 8, ["modelValue", "onUpdate:modelValue"])]),
                    _: 1
                  })]),
                  _: 1
                }, 8, ["keywords"])])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"])) : _createCommentVNode("v-if", true),
            _createVNode(_component_SearchMarker, { keywords: ["accessibility"] }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.accessibility),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_12]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
                  _createVNode(MkFeatureBanner, {
                    icon: "/client-assets/mens_room_3d.png",
                    color: "#0011ff"
                  }, {
                    default: _withCtx(() => [_createVNode(_component_SearchText, null, {
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._settings.accessibilityBanner),
                        1
                        /* TEXT */
                      )]),
                      _: 1
                    })]),
                    _: 1
                  }),
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    _createVNode(_component_SearchMarker, { keywords: [
                      "animation",
                      "motion",
                      "reduce"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "animation" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(reduceAnimation),
                          "onUpdate:modelValue": ($event) => reduceAnimation.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.reduceUiAnimation),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "disable",
                      "animation",
                      "image",
                      "photo",
                      "picture",
                      "media",
                      "thumbnail",
                      "gif"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "disableShowingAnimatedImages" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(disableShowingAnimatedImages),
                          "onUpdate:modelValue": ($event) => disableShowingAnimatedImages.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.disableShowingAnimatedImages),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          caption: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.disableShowingAnimatedImages_caption),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "mfm",
                      "enable",
                      "show",
                      "animated"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "animatedMfm" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(animatedMfm),
                          "onUpdate:modelValue": ($event) => animatedMfm.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.enableAnimatedMfm),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "tabs",
                      "tabbar",
                      "bottom",
                      "under"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "showPageTabBarBottom" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(showPageTabBarBottom),
                          "onUpdate:modelValue": ($event) => showPageTabBarBottom.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts._settings.showPageTabBarBottom),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "swipe",
                      "horizontal",
                      "tab"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "enableHorizontalSwipe" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(enableHorizontalSwipe),
                          "onUpdate:modelValue": ($event) => enableHorizontalSwipe.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.enableHorizontalSwipe),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "swipe",
                      "pull",
                      "refresh"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "enablePullToRefresh" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(enablePullToRefresh),
                          "onUpdate:modelValue": ($event) => enablePullToRefresh.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts._settings.enablePullToRefresh),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts._settings.enablePullToRefresh_description),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "keep",
                      "screen",
                      "display",
                      "on"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "keepScreenOn" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(keepScreenOn),
                          "onUpdate:modelValue": ($event) => keepScreenOn.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.keepScreenOn),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "native",
                      "system",
                      "video",
                      "audio",
                      "player",
                      "media"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "useNativeUiForVideoAudioPlayer" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(useNativeUiForVideoAudioPlayer),
                          "onUpdate:modelValue": ($event) => useNativeUiForVideoAudioPlayer.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.useNativeUIForVideoAudioPlayer),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ["text", "selectable"] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "makeEveryTextElementsSelectable" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(makeEveryTextElementsSelectable),
                          "onUpdate:modelValue": ($event) => makeEveryTextElementsSelectable.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts._settings.makeEveryTextElementsSelectable),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          caption: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts._settings.makeEveryTextElementsSelectable_description),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"])
                  ]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "menu",
                    "style",
                    "popup",
                    "drawer"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "menuStyle" }, {
                      default: _withCtx(() => [_createVNode(MkSelect, {
                        items: [
                          {
                            label: _unref(i18n).ts.auto,
                            value: "auto"
                          },
                          {
                            label: _unref(i18n).ts.popup,
                            value: "popup"
                          },
                          {
                            label: _unref(i18n).ts.drawer,
                            value: "drawer"
                          }
                        ],
                        modelValue: _unref(menuStyle),
                        "onUpdate:modelValue": ($event) => menuStyle.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.menuStyle),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, [
                        "items",
                        "modelValue",
                        "onUpdate:modelValue"
                      ])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "contextmenu",
                    "system",
                    "native"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "contextMenu" }, {
                      default: _withCtx(() => [_createVNode(MkSelect, {
                        items: [
                          {
                            label: _unref(i18n).ts._contextMenu.app,
                            value: "app"
                          },
                          {
                            label: _unref(i18n).ts._contextMenu.appWithShift,
                            value: "appWithShift"
                          },
                          {
                            label: _unref(i18n).ts._contextMenu.native,
                            value: "native"
                          }
                        ],
                        modelValue: _unref(contextMenu),
                        "onUpdate:modelValue": ($event) => contextMenu.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts._contextMenu.title),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, [
                        "items",
                        "modelValue",
                        "onUpdate:modelValue"
                      ])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: ["font", "size"] }, {
                    default: _withCtx(() => [_createVNode(MkRadios, {
                      options: [
                        {
                          value: null,
                          label: "Aa",
                          labelStyle: "font-size: 14px;"
                        },
                        {
                          value: "1",
                          label: "Aa",
                          labelStyle: "font-size: 15px;"
                        },
                        {
                          value: "2",
                          label: "Aa",
                          labelStyle: "font-size: 16px;"
                        },
                        {
                          value: "3",
                          label: "Aa",
                          labelStyle: "font-size: 17px;"
                        }
                      ],
                      modelValue: fontSize.value,
                      "onUpdate:modelValue": ($event) => fontSize.value = $event
                    }, {
                      label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts.fontSize),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, [
                      "options",
                      "modelValue",
                      "onUpdate:modelValue"
                    ])]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "font",
                    "system",
                    "native"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkSwitch, {
                      modelValue: useSystemFont.value,
                      "onUpdate:modelValue": ($event) => useSystemFont.value = $event
                    }, {
                      label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts.useSystemFont),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["modelValue", "onUpdate:modelValue"])]),
                    _: 1
                  }, 8, ["keywords"])
                ])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["performance"] }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.performance),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_13]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [
                  _createVNode(_component_SearchMarker, { keywords: [
                    "animation",
                    "motion",
                    "reduce"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "animation" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: !_unref(reduceAnimation),
                        "onUpdate:modelValue": (v) => reduceAnimation.value = !v
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts._settings.uiAnimations),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "animation",
                    "image",
                    "photo",
                    "picture",
                    "media",
                    "thumbnail",
                    "gif"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "disableShowingAnimatedImages" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: !_unref(disableShowingAnimatedImages),
                        "onUpdate:modelValue": (v) => disableShowingAnimatedImages.value = !v
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts._settings.enableAnimatedImages),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        }), _createElementVNode(
                          "div",
                          null,
                          _toDisplayString(_unref(i18n).ts.disableShowingAnimatedImages_caption),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: ["blur"] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "useBlurEffect" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(useBlurEffect),
                        "onUpdate:modelValue": ($event) => useBlurEffect.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.useBlurEffect),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: ["blur", "modal"] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "useBlurEffectForModal" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(useBlurEffectForModal),
                        "onUpdate:modelValue": ($event) => useBlurEffectForModal.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.useBlurEffectForModal),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "blurhash",
                    "image",
                    "photo",
                    "picture",
                    "thumbnail",
                    "placeholder"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "enableHighQualityImagePlaceholders" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(enableHighQualityImagePlaceholders),
                        "onUpdate:modelValue": ($event) => enableHighQualityImagePlaceholders.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts._settings.enableHighQualityImagePlaceholders),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: ["sticky"] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "useStickyIcons" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(useStickyIcons),
                        "onUpdate:modelValue": ($event) => useStickyIcons.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts._settings.useStickyIcons),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.turnOffToImprovePerformance),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(MkInfo, null, {
                    default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [
                      _createElementVNode(
                        "div",
                        null,
                        _toDisplayString(_unref(i18n).ts._clientPerformanceIssueTip.title) + ":",
                        1
                        /* TEXT */
                      ),
                      _createElementVNode("div", null, [_createElementVNode("div", null, [_createElementVNode(
                        "b",
                        null,
                        _toDisplayString(_unref(i18n).ts._clientPerformanceIssueTip.makeSureDisabledAdBlocker),
                        1
                        /* TEXT */
                      )]), _createElementVNode(
                        "div",
                        null,
                        _toDisplayString(_unref(i18n).ts._clientPerformanceIssueTip.makeSureDisabledAdBlocker_description),
                        1
                        /* TEXT */
                      )]),
                      _createElementVNode("div", null, [_createElementVNode("div", null, [_createElementVNode(
                        "b",
                        null,
                        _toDisplayString(_unref(i18n).ts._clientPerformanceIssueTip.makeSureDisabledCustomCss),
                        1
                        /* TEXT */
                      )]), _createElementVNode(
                        "div",
                        null,
                        _toDisplayString(_unref(i18n).ts._clientPerformanceIssueTip.makeSureDisabledCustomCss_description),
                        1
                        /* TEXT */
                      )]),
                      _createElementVNode("div", null, [_createElementVNode("div", null, [_createElementVNode(
                        "b",
                        null,
                        _toDisplayString(_unref(i18n).ts._clientPerformanceIssueTip.makeSureDisabledAddons),
                        1
                        /* TEXT */
                      )]), _createElementVNode(
                        "div",
                        null,
                        _toDisplayString(_unref(i18n).ts._clientPerformanceIssueTip.makeSureDisabledAddons_description),
                        1
                        /* TEXT */
                      )])
                    ])]),
                    _: 1
                  })
                ])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["datasaver"] }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.dataSaver),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_14]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
                  _createVNode(MkInfo, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.reloadRequiredToApplySettings),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  }),
                  _createElementVNode("div", { class: "_buttons" }, [_createVNode(MkButton, {
                    inline: "",
                    onClick: enableAllDataSaver
                  }, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.enableAll),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  }), _createVNode(MkButton, {
                    inline: "",
                    onClick: disableAllDataSaver
                  }, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.disableAll),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(MkSwitch, {
                      modelValue: dataSaver.value.media,
                      "onUpdate:modelValue": ($event) => dataSaver.value.media = $event
                    }, {
                      caption: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._media.description),
                        1
                        /* TEXT */
                      )]),
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._media.title),
                        1
                        /* TEXT */
                      ), _createTextVNode(" ")]),
                      _: 1
                    }, 8, ["modelValue", "onUpdate:modelValue"]),
                    _createVNode(MkSwitch, {
                      modelValue: dataSaver.value.avatar,
                      "onUpdate:modelValue": ($event) => dataSaver.value.avatar = $event
                    }, {
                      caption: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._avatar.description),
                        1
                        /* TEXT */
                      )]),
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._avatar.title),
                        1
                        /* TEXT */
                      ), _createTextVNode(" ")]),
                      _: 1
                    }, 8, ["modelValue", "onUpdate:modelValue"]),
                    _createVNode(MkSwitch, {
                      disabled: !_unref(instance).enableUrlPreview,
                      modelValue: dataSaver.value.disableUrlPreview,
                      "onUpdate:modelValue": ($event) => dataSaver.value.disableUrlPreview = $event
                    }, {
                      caption: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._disableUrlPreview.description),
                        1
                        /* TEXT */
                      )]),
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._disableUrlPreview.title),
                        1
                        /* TEXT */
                      ), _createTextVNode(" ")]),
                      _: 1
                    }, 8, [
                      "disabled",
                      "modelValue",
                      "onUpdate:modelValue"
                    ]),
                    _createVNode(MkSwitch, {
                      disabled: !_unref(instance).enableUrlPreview || dataSaver.value.disableUrlPreview,
                      modelValue: dataSaver.value.urlPreviewThumbnail,
                      "onUpdate:modelValue": ($event) => dataSaver.value.urlPreviewThumbnail = $event
                    }, {
                      caption: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._urlPreviewThumbnail.description),
                        1
                        /* TEXT */
                      )]),
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._urlPreviewThumbnail.title),
                        1
                        /* TEXT */
                      ), _createTextVNode(" ")]),
                      _: 1
                    }, 8, [
                      "disabled",
                      "modelValue",
                      "onUpdate:modelValue"
                    ]),
                    _createVNode(MkSwitch, {
                      modelValue: dataSaver.value.code,
                      "onUpdate:modelValue": ($event) => dataSaver.value.code = $event
                    }, {
                      caption: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._code.description),
                        1
                        /* TEXT */
                      )]),
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(_unref(i18n).ts._dataSaver._code.title),
                        1
                        /* TEXT */
                      ), _createTextVNode(" ")]),
                      _: 1
                    }, 8, ["modelValue", "onUpdate:modelValue"])
                  ])
                ])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["other"] }, {
              default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.other),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                icon: _withCtx(() => [_createVNode(_component_SearchIcon, null, {
                  default: _withCtx(() => [_hoisted_15]),
                  _: 1
                })]),
                default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    _createVNode(_component_SearchMarker, { keywords: [
                      "avatar",
                      "icon",
                      "square"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "squareAvatars" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(squareAvatars),
                          "onUpdate:modelValue": ($event) => squareAvatars.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.squareAvatars),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ["effect", "show"] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "enableSeasonalScreenEffect" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(enableSeasonalScreenEffect),
                          "onUpdate:modelValue": ($event) => enableSeasonalScreenEffect.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.seasonalScreenEffect),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: [
                      "image",
                      "photo",
                      "picture",
                      "media",
                      "thumbnail",
                      "new",
                      "tab"
                    ] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "imageNewTab" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(imageNewTab),
                          "onUpdate:modelValue": ($event) => imageNewTab.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.openImageInNewTab),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ["follow", "replies"] }, {
                      default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "defaultFollowWithReplies" }, {
                        default: _withCtx(() => [_createVNode(MkSwitch, {
                          modelValue: _unref(defaultFollowWithReplies),
                          "onUpdate:modelValue": ($event) => defaultFollowWithReplies.value = $event
                        }, {
                          label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                            default: _withCtx(() => [_createTextVNode(
                              _toDisplayString(_unref(i18n).ts.withRepliesByDefaultForNewlyFollowed),
                              1
                              /* TEXT */
                            )]),
                            _: 1
                          })]),
                          _: 1
                        }, 8, ["modelValue", "onUpdate:modelValue"])]),
                        _: 1
                      })]),
                      _: 1
                    }, 8, ["keywords"])
                  ]),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "server",
                    "disconnect",
                    "reconnect",
                    "reload",
                    "streaming"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "serverDisconnectedBehavior" }, {
                      default: _withCtx(() => [_createVNode(MkSelect, {
                        items: [
                          {
                            label: _unref(i18n).ts._serverDisconnectedBehavior.reload,
                            value: "reload"
                          },
                          {
                            label: _unref(i18n).ts._serverDisconnectedBehavior.dialog,
                            value: "dialog"
                          },
                          {
                            label: _unref(i18n).ts._serverDisconnectedBehavior.quiet,
                            value: "quiet"
                          }
                        ],
                        modelValue: _unref(serverDisconnectedBehavior),
                        "onUpdate:modelValue": ($event) => serverDisconnectedBehavior.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.whenServerDisconnected),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, [
                        "items",
                        "modelValue",
                        "onUpdate:modelValue"
                      ])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: ["cache", "page"] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "numberOfPageCache" }, {
                      default: _withCtx(() => [_createVNode(MkRange, {
                        min: 1,
                        max: 10,
                        step: 1,
                        easing: "",
                        modelValue: _unref(numberOfPageCache),
                        "onUpdate:modelValue": ($event) => numberOfPageCache.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.numberOfPageCache),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        caption: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts.numberOfPageCacheDescription),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      }, 8, [
                        "min",
                        "max",
                        "step",
                        "modelValue",
                        "onUpdate:modelValue"
                      ])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, { keywords: ["ad", "show"] }, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "forceShowAds" }, {
                      default: _withCtx(() => [_createVNode(MkSwitch, {
                        modelValue: _unref(forceShowAds),
                        "onUpdate:modelValue": ($event) => forceShowAds.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.forceShowAds),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        _: 1
                      }, 8, ["modelValue", "onUpdate:modelValue"])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"]),
                  _createVNode(_component_SearchMarker, null, {
                    default: _withCtx(() => [_createVNode(MkPreferenceContainer, { k: "hemisphere" }, {
                      default: _withCtx(() => [_createVNode(MkRadios, {
                        options: [{
                          value: "N",
                          label: _unref(i18n).ts._hemisphere.N
                        }, {
                          value: "S",
                          label: _unref(i18n).ts._hemisphere.S
                        }],
                        modelValue: _unref(hemisphere),
                        "onUpdate:modelValue": ($event) => hemisphere.value = $event
                      }, {
                        label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [_createTextVNode(
                            _toDisplayString(_unref(i18n).ts.hemisphere),
                            1
                            /* TEXT */
                          )]),
                          _: 1
                        })]),
                        caption: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts._hemisphere.caption),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      }, 8, [
                        "options",
                        "modelValue",
                        "onUpdate:modelValue"
                      ])]),
                      _: 1
                    })]),
                    _: 1
                  }),
                  _createVNode(_component_SearchMarker, { keywords: [
                    "emoji",
                    "dictionary",
                    "additional",
                    "extra"
                  ] }, {
                    default: _withCtx(() => [_createVNode(MkFolder, null, {
                      label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts.additionalEmojiDictionary),
                          1
                          /* TEXT */
                        )]),
                        _: 1
                      })]),
                      default: _withCtx(() => [_createElementVNode("div", { class: "_buttons" }, [(_openBlock(true), _createElementBlock(
                        _Fragment,
                        null,
                        _renderList(_unref(emojiIndexLangs), (lang) => {
                          return _openBlock(), _createElementBlock(
                            _Fragment,
                            { key: lang.value },
                            [_unref(store).r.additionalUnicodeEmojiIndexes.value[lang.value] ? (_openBlock(), _createBlock(MkButton, {
                              key: 0,
                              danger: "",
                              onClick: ($event) => removeEmojiIndex(lang.value)
                            }, {
                              default: _withCtx(() => [
                                _hoisted_16,
                                _createTextVNode(" "),
                                _createTextVNode(
                                  _toDisplayString(_unref(i18n).ts.remove),
                                  1
                                  /* TEXT */
                                ),
                                _createTextVNode(" ("),
                                _createTextVNode(
                                  _toDisplayString(getEmojiIndexLangName(lang.value)),
                                  1
                                  /* TEXT */
                                ),
                                _createTextVNode(")")
                              ]),
                              _: 2
                            }, 8, ["onClick"])) : (_openBlock(), _createBlock(MkButton, {
                              key: 1,
                              onClick: ($event) => downloadEmojiIndex(lang.value)
                            }, {
                              default: _withCtx(() => [
                                _hoisted_17,
                                _createTextVNode(" "),
                                _createTextVNode(
                                  _toDisplayString(getEmojiIndexLangName(lang.value)),
                                  1
                                  /* TEXT */
                                ),
                                _createTextVNode(
                                  _toDisplayString(_unref(store).r.additionalUnicodeEmojiIndexes.value[lang.value] ? ` (${_unref(i18n).ts.installed})` : ""),
                                  1
                                  /* TEXT */
                                )
                              ]),
                              _: 2
                            }, 8, ["onClick"]))],
                            64
                            /* STABLE_FRAGMENT */
                          );
                        }),
                        128
                        /* KEYED_FRAGMENT */
                      ))])]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["keywords"])
                ])]),
                _: 1
              }, 8, ["defaultOpen"])]),
              _: 1
            }, 8, ["keywords"])
          ]),
          _hoisted_18,
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(FormLink, { to: "/settings/navbar" }, {
              icon: _withCtx(() => [_hoisted_19]),
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.navbar),
                1
                /* TEXT */
              )]),
              _: 1
            }),
            _createVNode(FormLink, { to: "/settings/statusbar" }, {
              icon: _withCtx(() => [_hoisted_20]),
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.statusbar),
                1
                /* TEXT */
              )]),
              _: 1
            }),
            _createVNode(FormLink, { to: "/settings/deck" }, {
              icon: _withCtx(() => [_hoisted_21]),
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.deck),
                1
                /* TEXT */
              )]),
              _: 1
            }),
            _createVNode(FormLink, { to: "/settings/custom-css" }, {
              icon: _withCtx(() => [_hoisted_22]),
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.customCss),
                1
                /* TEXT */
              )]),
              _: 1
            })
          ])
        ])]),
        _: 1
      }, 8, ["label", "keywords"]);
    };
  }
};
