import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, normalizeProps as _normalizeProps, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-brand-x" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-player-play" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-picture-in-picture" })
import { defineAsyncComponent, onDeactivated, onUnmounted, ref } from 'vue'
import { url as local } from '@@/js/config.js'
import { versatileLang } from '@@/js/intl-const.js'
import type { summaly } from '@misskey-dev/summaly'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { deviceKind } from '@/utility/device-kind.js'
import MkButton from '@/components/MkButton.vue'
import { transformPlayerUrl } from '@/utility/url-preview.js'
import { store } from '@/store.js'
import { prefer } from '@/preferences.js'
import { maybeMakeRelative } from '@@/js/url.js'

type SummalyResult = Awaited<ReturnType<typeof summaly>>;
const MOBILE_THRESHOLD = 500;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkUrlPreview',
  props: {
    url: { type: String, required: true },
    detail: { type: Boolean, required: false, default: false },
    compact: { type: Boolean, required: false, default: false },
    showActions: { type: Boolean, required: false, default: true }
  },
  setup(__props: any) {

const props = __props
const isMobile = ref(deviceKind === 'smartphone' || window.innerWidth <= MOBILE_THRESHOLD);
const maybeRelativeUrl = maybeMakeRelative(props.url, local);
const self = maybeRelativeUrl !== props.url;
const attr = self ? 'to' : 'href';
const target = self ? null : '_blank';
const fetching = ref(true);
const title = ref<string | null>(null);
const description = ref<string | null>(null);
const thumbnail = ref<string | null>(null);
const icon = ref<string | null>(null);
const sitename = ref<string | null>(null);
const sensitive = ref<boolean>(false);
const player = ref({
	url: null,
	width: null,
	height: null,
} as SummalyResult['player']);
const playerEnabled = ref(false);
const tweetId = ref<string | null>(null);
const tweetExpanded = ref(props.detail);
const embedId = `embed${Math.random().toString().replace(/\D/, '')}`;
const tweetHeight = ref(150);
const unknownUrl = ref(false);
onDeactivated(() => {
	playerEnabled.value = false;
});
const requestUrl = new URL(props.url);
if (!['http:', 'https:'].includes(requestUrl.protocol)) throw new Error('invalid url');
if (requestUrl.hostname === 'twitter.com' || requestUrl.hostname === 'mobile.twitter.com' || requestUrl.hostname === 'x.com' || requestUrl.hostname === 'mobile.x.com') {
	const m = requestUrl.pathname.match(/^\/.+\/status(?:es)?\/(\d+)/);
	if (m) tweetId.value = m[1];
}
if (requestUrl.hostname === 'music.youtube.com' && requestUrl.pathname.match('^/(?:watch|channel)')) {
	requestUrl.hostname = 'www.youtube.com';
}
requestUrl.hash = '';
window.fetch(`/url?url=${encodeURIComponent(requestUrl.href)}&lang=${versatileLang}`)
	.then(res => {
		if (!res.ok) {
			if (_DEV_) {
				console.warn(`[HTTP${res.status}] Failed to fetch url preview`);
			}
			return null;
		}
		return res.json();
	})
	.then((info: SummalyResult | null) => {
		if (!info || info.url == null) {
			fetching.value = false;
			unknownUrl.value = true;
			return;
		}
		fetching.value = false;
		unknownUrl.value = false;
		title.value = info.title;
		description.value = info.description;
		thumbnail.value = info.thumbnail;
		icon.value = info.icon;
		sitename.value = info.sitename;
		player.value = info.player;
		sensitive.value = info.sensitive ?? false;
	});
function adjustTweetHeight(message: MessageEvent) {
	if (message.origin !== 'https://platform.twitter.com') return;
	const embed = message.data?.['twttr.embed'];
	if (embed?.method !== 'twttr.private.resize') return;
	if (embed?.id !== embedId) return;
	const height = embed?.params[0]?.height;
	if (height) tweetHeight.value = height;
}
function openPlayer(): void {
	const { dispose } = os.popup(defineAsyncComponent(() => import('@/components/MkYouTubePlayer.vue')), {
		url: requestUrl.href,
	}, {
		closed: () => {
			dispose();
		},
	});
}
window.addEventListener('message', adjustTweetHeight);
onUnmounted(() => {
	window.removeEventListener('message', adjustTweetHeight);
});

return (_ctx: any,_cache: any) => {
  const _component_MkEllipsis = _resolveComponent("MkEllipsis")

  return (player.value.url && playerEnabled.value)
      ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.player),
          style: _normalizeStyle(player.value.width ? `padding: ${(player.value.height || 0) / player.value.width * 100}% 0 0` : `padding: ${(player.value.height || 0)}px 0 0`)
        }, [ (player.value.url.startsWith('http://') || player.value.url.startsWith('https://')) ? (_openBlock(), _createElementBlock("iframe", {
              key: 0,
              sandbox: "allow-popups allow-popups-to-escape-sandbox allow-scripts allow-storage-access-by-user-activation allow-same-origin",
              scrolling: "no",
              allow: player.value.allow == null ? 'autoplay;encrypted-media;fullscreen' : player.value.allow.filter(x => ['autoplay', 'clipboard-write', 'fullscreen', 'encrypted-media', 'picture-in-picture', 'web-share'].includes(x)).join(';'),
              class: _normalizeClass(_ctx.$style.playerIframe),
              src: _unref(transformPlayerUrl)(player.value.url),
              style: { border: 0 }
            })) : (_openBlock(), _createElementBlock("span", { key: 1 }, "invalid url")) ], 4 /* STYLE */), _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.action)
        }, [ _createVNode(MkButton, {
            small: true,
            inline: "",
            onClick: _cache[0] || (_cache[0] = ($event: any) => (playerEnabled.value = false))
          }, {
            default: _withCtx(() => [
              _hoisted_1,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.disablePlayer), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["small"]) ]) ], 64 /* STABLE_FRAGMENT */))
      : (tweetId.value && tweetExpanded.value)
        ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createElementVNode("div", { ref: "twitter" }, [ _createElementVNode("iframe", {
              ref: "tweet",
              allow: "fullscreen;web-share",
              sandbox: "allow-popups allow-popups-to-escape-sandbox allow-scripts allow-same-origin",
              scrolling: "no",
              style: _normalizeStyle({ position: 'relative', width: '100%', height: `${tweetHeight.value}px`, border: 0 }),
              src: `https://platform.twitter.com/embed/index.html?embedId=${_unref(embedId)}&amp;hideCard=false&amp;hideThread=false&amp;lang=en&amp;theme=${_unref(store).s.darkMode ? 'dark' : 'light'}&amp;id=${tweetId.value}`
            }, null, 12 /* STYLE, PROPS */, ["src"]) ], 512 /* NEED_PATCH */), _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.action)
          }, [ _createVNode(MkButton, {
              small: true,
              inline: "",
              onClick: _cache[1] || (_cache[1] = ($event: any) => (tweetExpanded.value = false))
            }, {
              default: _withCtx(() => [
                _hoisted_2,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.close), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["small"]) ]) ], 64 /* STABLE_FRAGMENT */))
      : (_openBlock(), _createElementBlock("div", { key: 2 }, [ _createVNode(_resolveDynamicComponent(_unref(self) ? 'MkA' : 'a'), _normalizeProps({
          class: _normalizeClass([_ctx.$style.link, { [_ctx.$style.compact]: __props.compact }]),
          [_ctx.attr || ""]: _unref(maybeRelativeUrl),
          rel: "nofollow noopener",
          target: _unref(target),
          title: __props.url
        }), {
          default: _withCtx(() => [
            (thumbnail.value && !sensitive.value)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.thumbnail),
                style: _normalizeStyle(_unref(prefer).s.dataSaver.urlPreviewThumbnail ? '' : { backgroundImage: `url('${thumbnail.value}')` })
              }))
              : _createCommentVNode("v-if", true),
            _createElementVNode("article", {
              class: _normalizeClass(_ctx.$style.body)
            }, [
              _createElementVNode("header", {
                class: _normalizeClass(_ctx.$style.header)
              }, [
                (unknownUrl.value)
                  ? (_openBlock(), _createElementBlock("h1", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.title)
                  }, _toDisplayString(__props.url), 1 /* TEXT */))
                  : (fetching.value)
                    ? (_openBlock(), _createElementBlock("h1", {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.title)
                    }, [
                      _createVNode(_component_MkEllipsis)
                    ]))
                  : (_openBlock(), _createElementBlock("h1", {
                    key: 2,
                    class: _normalizeClass(_ctx.$style.title),
                    title: title.value ?? undefined
                  }, _toDisplayString(title.value), 1 /* TEXT */))
              ]),
              (unknownUrl.value)
                ? (_openBlock(), _createElementBlock("p", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.text)
                }, _toDisplayString(_unref(i18n).ts.failedToPreviewUrl), 1 /* TEXT */))
                : (fetching.value)
                  ? (_openBlock(), _createElementBlock("p", {
                    key: 1,
                    class: _normalizeClass(_ctx.$style.text)
                  }, [
                    _createVNode(_component_MkEllipsis)
                  ]))
                : (description.value)
                  ? (_openBlock(), _createElementBlock("p", {
                    key: 2,
                    class: _normalizeClass(_ctx.$style.text),
                    title: description.value
                  }, _toDisplayString(description.value.length > 85 ? description.value.slice(0, 85) + '…' : description.value), 1 /* TEXT */))
                : _createCommentVNode("v-if", true),
              _createElementVNode("footer", {
                class: _normalizeClass(_ctx.$style.footer)
              }, [
                (icon.value)
                  ? (_openBlock(), _createElementBlock("img", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.siteIcon),
                    src: icon.value
                  }))
                  : _createCommentVNode("v-if", true),
                (unknownUrl.value)
                  ? (_openBlock(), _createElementBlock("p", {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.siteName)
                  }, _toDisplayString(_unref(requestUrl).host), 1 /* TEXT */))
                  : (fetching.value)
                    ? (_openBlock(), _createElementBlock("p", {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.siteName)
                    }, [
                      _createVNode(_component_MkEllipsis)
                    ]))
                  : (_openBlock(), _createElementBlock("p", {
                    key: 2,
                    class: _normalizeClass(_ctx.$style.siteName),
                    title: sitename.value ?? _unref(requestUrl).host
                  }, _toDisplayString(sitename.value ?? _unref(requestUrl).host), 1 /* TEXT */))
              ])
            ])
          ]),
          _: 1 /* STABLE */
        }, 16 /* FULL_PROPS */, ["target", "title"]), (__props.showActions) ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ (tweetId.value) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.action)
              }, [ _createVNode(MkButton, {
                  small: true,
                  inline: "",
                  onClick: _cache[2] || (_cache[2] = ($event: any) => (tweetExpanded.value = true))
                }, {
                  default: _withCtx(() => [
                    _hoisted_3,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.expandTweet), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["small"]) ])) : _createCommentVNode("v-if", true), (!playerEnabled.value && player.value.url) ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.action)
              }, [ _createVNode(MkButton, {
                  small: true,
                  inline: "",
                  onClick: _cache[3] || (_cache[3] = ($event: any) => (playerEnabled.value = true))
                }, {
                  default: _withCtx(() => [
                    _hoisted_4,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.enablePlayer), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["small"]), (!isMobile.value) ? (_openBlock(), _createBlock(MkButton, {
                    key: 0,
                    small: true,
                    inline: "",
                    onClick: _cache[4] || (_cache[4] = ($event: any) => (openPlayer()))
                  }, {
                    default: _withCtx(() => [
                      _hoisted_5,
                      _createTextVNode(" "),
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.openInWindow), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  }, 8 /* PROPS */, ["small"])) : _createCommentVNode("v-if", true) ])) : _createCommentVNode("v-if", true) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true) ]))
}
}

})
