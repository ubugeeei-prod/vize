import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("div", { class: "misskey" }, "Misskey")
const _hoisted_2 = { class: "version" }
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("img", { src: "/fluent-emoji/1f3c6.png", class: "treasureImg" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_5 = { href: "https://misskey-hub.net/docs/about-misskey/", target: "_blank", class: "_link" }
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-language-hiragana" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pig-money" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-code" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-download" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("img", { style: "width: 100%;", src: "https://assets.misskey-hub.net/sponsors/masknetwork.png", alt: "Mask Network" })
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("img", { style: "width: 100%;", src: "https://assets.misskey-hub.net/sponsors/xserver.png", alt: "XServer" })
const _hoisted_13 = /*#__PURE__*/ _createElementVNode("img", { style: "width: 100%;", src: "https://assets.misskey-hub.net/sponsors/skeb.svg", alt: "Skeb" })
const _hoisted_14 = /*#__PURE__*/ _createElementVNode("img", { style: "width: 100%;", src: "https://assets.misskey-hub.net/sponsors/gmo_pepabo.svg", alt: "GMO Pepabo" })
const _hoisted_15 = /*#__PURE__*/ _createElementVNode("img", { style: "width: 100%;", src: "https://assets.misskey-hub.net/sponsors/purple-dot-digital.jpg", alt: "Purple Dot Digital" })
const _hoisted_16 = /*#__PURE__*/ _createElementVNode("img", { style: "width: 100%;", src: "https://assets.misskey-hub.net/sponsors/sads-llc.png", alt: "合同会社サッズ" })
import { nextTick, onBeforeUnmount, ref, useTemplateRef, computed } from 'vue'
import { host, version } from '@@/js/config.js'
import FormLink from '@/components/form/link.vue'
import FormSection from '@/components/form/section.vue'
import MkButton from '@/components/MkButton.vue'
import MkInfo from '@/components/MkInfo.vue'
import { physics } from '@/utility/physics.js'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import * as os from '@/os.js'
import { definePage } from '@/page.js'
import { claimAchievement, claimedAchievements } from '@/utility/achievements.js'
import { $i } from '@/i.js'
import { prefer } from '@/preferences.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'about-misskey',
  setup(__props) {

const patronsWithIcon = [{
	name: 'カイヤン',
	icon: 'https://assets.misskey-hub.net/patrons/a2820716883e408cb87773e377ce7c8d.jpg',
}, {
	name: 'だれかさん',
	icon: 'https://assets.misskey-hub.net/patrons/f7409b5e5a88477a9b9d740c408de125.jpg',
}, {
	name: 'narazaka',
	icon: 'https://assets.misskey-hub.net/patrons/e3affff31ffb4877b1196c7360abc3e5.jpg',
}, {
	name: 'ひとぅ',
	icon: 'https://assets.misskey-hub.net/patrons/8cc0d0a0a6d84c88bca1aedabf6ed5ab.jpg',
}, {
	name: 'ぱーこ',
	icon: 'https://assets.misskey-hub.net/patrons/79c6602ffade489e8df2fcf2c2bc5d9d.jpg',
}, {
	name: 'わっほー☆',
	icon: 'https://assets.misskey-hub.net/patrons/d31d5d13924443a082f3da7966318a0a.jpg',
}, {
	name: 'mollinaca',
	icon: 'https://assets.misskey-hub.net/patrons/ceb36b8f66e549bdadb3b90d5da62314.jpg',
}, {
	name: '坂本龍',
	icon: 'https://assets.misskey-hub.net/patrons/a631cf8b490145cf8dbbe4e7508cfbc2.jpg',
}, {
	name: 'takke',
	icon: 'https://assets.misskey-hub.net/patrons/6c3327e626c046f2914fbcd9f7557935.jpg',
}, {
	name: 'ぺんぎん',
	icon: 'https://assets.misskey-hub.net/patrons/6a652e0534ff4cb1836e7ce4968d76a7.jpg',
}, {
	name: 'かみらえっと',
	icon: 'https://assets.misskey-hub.net/patrons/be1326bda7d940a482f3758ffd9ffaf6.jpg',
}, {
	name: 'へてて',
	icon: 'https://assets.misskey-hub.net/patrons/0431eacd7c6843d09de8ea9984307e86.jpg',
}, {
	name: 'spinlock',
	icon: 'https://assets.misskey-hub.net/patrons/6a1cebc819d540a78bf20e9e3115baa8.jpg',
}, {
	name: 'じゅくま',
	icon: 'https://assets.misskey-hub.net/patrons/3e56bdac69dd42f7a06e0f12cf2fc895.jpg',
}, {
	name: '清遊あみ',
	icon: 'https://assets.misskey-hub.net/patrons/de25195b88e940a388388bea2e7637d8.jpg',
}, {
	name: 'Nagi8410',
	icon: 'https://assets.misskey-hub.net/patrons/31b102ab4fc540ed806b0461575d38be.jpg',
}, {
	name: '山岡士郎',
	icon: 'https://assets.misskey-hub.net/patrons/84b9056341684266bb1eda3e680d094d.jpg',
}, {
	name: 'よもやまたろう',
	icon: 'https://assets.misskey-hub.net/patrons/4273c9cce50d445f8f7d0f16113d6d7f.jpg',
}, {
	name: '花咲ももか',
	icon: 'https://assets.misskey-hub.net/patrons/8c9b2b9128cb4fee99f04bb4f86f2efa.jpg',
}, {
	name: 'カガミ',
	icon: 'https://assets.misskey-hub.net/patrons/226ea3a4617749548580ec2d9a263e24.jpg',
}, {
	name: 'フランギ・シュウ',
	icon: 'https://assets.misskey-hub.net/patrons/3016d37e35f3430b90420176c912d304.jpg',
}, {
	name: '百日紅',
	icon: 'https://assets.misskey-hub.net/patrons/302dce2898dd457ba03c3f7dc037900b.jpg',
}, {
	name: 'taichan',
	icon: 'https://assets.misskey-hub.net/patrons/f981ab0159fb4e2c998e05f7263e1cd9.jpg',
}, {
	name: '猫吉よりお',
	icon: 'https://assets.misskey-hub.net/patrons/a11518b3b34b4536a4bdd7178ba76a7b.jpg',
}, {
	name: '有栖かずみ',
	icon: 'https://assets.misskey-hub.net/patrons/9240e8e0ba294a8884143e99ac7ed6a0.jpg',
}, {
	name: 'イカロ(コアラ)',
	icon: 'https://assets.misskey-hub.net/patrons/50b9bdc03735412c80807dbdf32cecb6.jpg',
}, {
	name: 'ハチノス３号',
	icon: 'https://assets.misskey-hub.net/patrons/030347a6f8ce4e82bc5184b5aad09a18.jpg',
}, {
	name: 'Takeno',
	icon: 'https://assets.misskey-hub.net/patrons/6fba81536aea48fe94a30909c502dfa1.jpg',
}, {
	name: 'くびすじ',
	icon: 'https://assets.misskey-hub.net/patrons/aa5789850b2149aeb5b89ebe2e9083db.jpg',
}, {
	name: '古道京紗＠ぷらいべったー',
	icon: 'https://assets.misskey-hub.net/patrons/18346d0519704963a4beabe6abc170af.jpg',
}, {
	name: '越貝鯛丸',
	icon: 'https://assets.misskey-hub.net/patrons/86c7374de37849b882d8ebbc833dc968.jpg',
}, {
	name: '☔あめ🍬(灬˘╰╯˘灬)',
	icon: 'https://assets.misskey-hub.net/patrons/676eea72d4884d3f89aababbb62533fb.jpg',
}, {
	name: '貯水よび',
	icon: 'https://assets.misskey-hub.net/patrons/2974506d53244bbe94a67707b27099e2.jpg',
}, {
	name: 'はるかさ',
	icon: 'https://assets.misskey-hub.net/patrons/26ce2432739a400aa3aa0de0ef67a107.jpg',
}, {
	name: '天鈴のあ',
	icon: 'https://assets.misskey-hub.net/patrons/995cdbb00bd6421184461a883adfe1d9.jpg',
}, {
	name: 'えとゔぁす',
	icon: 'https://assets.misskey-hub.net/patrons/2578f441b82a44cfaa55ba83a318b26e.jpg',
}, {
	name: 'Soli',
	icon: 'https://assets.misskey-hub.net/patrons/448070c81ebd41eda4ea2328291b2efe.jpg',
}, {
	name: 'ささくれりょう',
	icon: 'https://assets.misskey-hub.net/patrons/cf55022cee6c41da8e70a43587aaad9a.jpg',
}, {
	name: 'Macop',
	icon: 'https://assets.misskey-hub.net/patrons/ee052bf550014d36a643ce3dce595640.jpg',
}, {
	name: 'なっかあ',
	icon: 'https://assets.misskey-hub.net/patrons/c2f5f3e394e74a64912284a2f4ca710e.jpg',
}, {
	name: '如月ユカ',
	icon: 'https://assets.misskey-hub.net/patrons/f24a042076a041b6811a2f124eb620ca.jpg',
}, {
	name: 'Yatoigawa',
	icon: 'https://assets.misskey-hub.net/patrons/505e3568885a4a488431a8f22b4553d0.jpg',
}, {
	name: '秋瀬カヲル',
	icon: 'https://assets.misskey-hub.net/patrons/0f22aeb866484f4fa51db6721e3f9847.jpg',
}, {
	name: '新井　治',
	icon: 'https://assets.misskey-hub.net/patrons/d160876f20394674a17963a0e609600a.jpg',
}, {
	name: 'しきいし',
	icon: 'https://assets.misskey-hub.net/patrons/77dd5387db41427ba9cbdc8849e76402.jpg',
}, {
	name: '井上千二十四',
	icon: 'https://assets.misskey-hub.net/patrons/193afa1f039b4c339866039c3dcd74bf.jpg',
}, {
	name: 'NigN',
	icon: 'https://assets.misskey-hub.net/patrons/1ccaef8e73ec4a50b59ff7cd688ceb84.jpg',
}, {
	name: 'しゃどかの',
	icon: 'https://assets.misskey-hub.net/patrons/5bec3c6b402942619e03f7a2ae76d69e.jpg',
}, {
	name: '大賀愛一郎',
	icon: 'https://assets.misskey-hub.net/patrons/c701a797d1df4125970f25d3052250ac.jpg',
}];
const patrons = [
	'まっちゃとーにゅ',
	'mametsuko',
	'noellabo',
	'AureoleArk',
	'Gargron',
	'Nokotaro Takeda',
	'Suji Yan',
	'oi_yekssim',
	'regtan',
	'Hekovic',
	'nenohi',
	'Gitmo Life Services',
	'naga_rus',
	'Efertone',
	'Melilot',
	'motcha',
	'nanami kan',
	'sevvie Rose',
	'Hayato Ishikawa',
	'Puniko',
	'skehmatics',
	'Quinton Macejkovic',
	'YUKIMOCHI',
	'dansup',
	'mewl hayabusa',
	'Emilis',
	'Fristi',
	'makokunsan',
	'chidori ninokura',
	'Peter G.',
	'見当かなみ',
	'natalie',
	'Maronu',
	'Steffen K9',
	'takimura',
	'sikyosyounin',
	'Nesakko',
	'YuzuRyo61',
	'blackskye',
	'sheeta.s',
	'osapon',
	'public_yusuke',
	'CG',
	'吴浥',
	't_w',
	'Jerry',
	'nafuchoco',
	'Takumi Sugita',
	'GLaTAN',
	'mkatze',
	'kabo2468y',
	'mydarkstar',
	'Roujo',
	'DignifiedSilence',
	'uroco @99',
	'totokoro',
	'うし',
	'kiritan',
	'weepjp',
	'Liaizon Wakest',
	'Duponin',
	'Blue',
	'Naoki Hirayama',
	'wara',
	'Wataru Manji (manji0)',
	'みなしま',
	'kanoy',
	'xianon',
	'Denshi',
	'Osushimaru',
	'にょんへら',
	'おのだい',
	'Leni',
	'oss',
	'Weeble',
	'蝉暮せせせ',
	'ThatOneCalculator',
	'pixeldesu',
	'あめ玉',
	'氷月氷華里',
	'Ebise Lutica',
	'巣黒るい@リスケモ男の娘VTuber!',
	'ふぇいぽむ',
	'依古田イコ',
	'戸塚こだま',
	'すー。',
	'秋雨/Slime-hatena.jp',
	'けそ',
	'ずも',
	'binvinyl',
	'渡志郎',
	'ぷーざ',
	'越貝鯛丸',
	'Nick / pprmint.',
	'kino3277',
	'美少女JKぐーちゃん',
	'てば',
	'たっくん',
	'SHO SEKIGUCHI',
	'塩キャベツ',
	'はとぽぷさん',
	'100の人 (エスパー・イーシア)',
	'ケモナーのケシン',
	'こまつぶり',
	'まゆつな空高',
	'asata',
	'ruru',
	'みりめい',
	'東雲 琥珀',
	'ほとラズ',
	'スズカケン',
	'蒼井よみこ',
];
const thereIsTreasure = ref($i && !claimedAchievements.includes('foundTreasure'));
let easterEggReady = false;
const easterEggEmojis = ref<{
	id: string,
	top: number,
	left: number,
	emoji: string
}[]>([]);
const easterEggEngine = ref<{ stop: () => void } | null>(null);
const containerEl = useTemplateRef('containerEl');
function iconLoaded() {
	if (containerEl.value == null) return;
	const emojis = prefer.s.emojiPalettes[0].emojis;
	const containerWidth = containerEl.value.offsetWidth;
	for (let i = 0; i < 32; i++) {
		easterEggEmojis.value.push({
			id: i.toString(),
			top: -(128 + (Math.random() * 256)),
			left: (Math.random() * containerWidth),
			emoji: emojis[Math.floor(Math.random() * emojis.length)],
		});
	}
	nextTick(() => {
		easterEggReady = true;
	});
}
function gravity() {
	if (containerEl.value == null) return;
	if (!easterEggReady) return;
	easterEggReady = false;
	easterEggEngine.value = physics(containerEl.value);
}
function iLoveMisskey() {
	os.post({
		initialText: 'I $[jelly ❤] #Misskey',
		instant: true,
	});
}
function getTreasure() {
	thereIsTreasure.value = false;
	claimAchievement('foundTreasure');
}
onBeforeUnmount(() => {
	if (easterEggEngine.value) {
		easterEggEngine.value.stop();
	}
});
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.aboutMisskey,
	icon: null,
}));

return (_ctx: any,_cache: any) => {
  const _component_MkCustomEmoji = _resolveComponent("MkCustomEmoji")
  const _component_MkEmoji = _resolveComponent("MkEmoji")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { style: "overflow: clip;" }, [
          _createElementVNode("div", {
            class: "_spacer",
            style: "--MI_SPACER-w: 600px; --MI_SPACER-min: 20px;"
          }, [
            _createElementVNode("div", { class: "_gaps_m znqjceqz" }, [
              _createElementVNode("div", { class: "about" }, [
                _createElementVNode("div", {
                  ref_key: "containerEl", ref: containerEl,
                  class: _normalizeClass(["container", { playing: easterEggEngine.value != null }])
                }, [
                  _createElementVNode("img", {
                    src: "/client-assets/about-icon.png",
                    alt: "",
                    class: "icon",
                    draggable: "false",
                    onLoad: iconLoaded,
                    onClick: gravity
                  }, null, 32 /* NEED_HYDRATION */),
                  _hoisted_1,
                  _createElementVNode("div", _hoisted_2, "v" + _toDisplayString(_unref(version)), 1 /* TEXT */),
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(easterEggEmojis.value, (emoji) => {
                    return (_openBlock(), _createElementBlock("span", {
                      key: emoji.id,
                      "data-physics-x": emoji.left,
                      "data-physics-y": emoji.top,
                      class: _normalizeClass(["emoji", { _physics_circle_: !emoji.emoji.startsWith(':') }])
                    }, [
                      (emoji.emoji[0] === ':')
                        ? (_openBlock(), _createBlock(_component_MkCustomEmoji, {
                          key: 0,
                          class: "emoji",
                          name: emoji.emoji,
                          normal: true,
                          noStyle: true,
                          fallbackToImage: true
                        }, null, 8 /* PROPS */, ["name", "normal", "noStyle", "fallbackToImage"]))
                        : (_openBlock(), _createBlock(_component_MkEmoji, {
                          key: 1,
                          class: "emoji",
                          emoji: emoji.emoji,
                          normal: true,
                          noStyle: true
                        }, null, 8 /* PROPS */, ["emoji", "normal", "noStyle"]))
                    ], 10 /* CLASS, PROPS */, ["data-physics-x", "data-physics-y"]))
                  }), 128 /* KEYED_FRAGMENT */))
                ], 2 /* CLASS */),
                (thereIsTreasure.value)
                  ? (_openBlock(), _createElementBlock("button", {
                    key: 0,
                    class: "_button treasure",
                    onClick: getTreasure
                  }, [
                    _hoisted_3
                  ]))
                  : _createCommentVNode("v-if", true)
              ]),
              _createElementVNode("div", { style: "text-align: center;" }, [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.about), 1 /* TEXT */),
                _hoisted_4,
                _createElementVNode("a", _hoisted_5, _toDisplayString(_unref(i18n).ts.learnMore), 1 /* TEXT */)
              ]),
              (_unref($i) != null)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  style: "text-align: center;"
                }, [
                  _createVNode(MkButton, {
                    primary: "",
                    rounded: "",
                    inline: "",
                    onClick: iLoveMisskey
                  }, {
                    default: _withCtx(() => [
                      _createTextVNode("I "),
                      _createVNode(_component_Mfm, { text: "$[jelly ❤]" }),
                      _createTextVNode(" #Misskey")
                    ]),
                    _: 1 /* STABLE */
                  })
                ]))
                : _createCommentVNode("v-if", true),
              _createVNode(FormSection, null, {
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    _createVNode(FormLink, {
                      to: "https://github.com/misskey-dev/misskey",
                      external: ""
                    }, {
                      icon: _withCtx(() => [
                        _hoisted_6
                      ]),
                      suffix: _withCtx(() => [
                        _createTextVNode("GitHub")
                      ]),
                      default: _withCtx(() => [
                        _createTextVNode("\n\t\t\t\t\t\t\t"),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.source), 1 /* TEXT */),
                        _createTextVNode(" ("),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.original), 1 /* TEXT */),
                        _createTextVNode(")\n\t\t\t\t\t\t\t")
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(FormLink, {
                      to: "https://crowdin.com/project/misskey",
                      external: ""
                    }, {
                      icon: _withCtx(() => [
                        _hoisted_7
                      ]),
                      suffix: _withCtx(() => [
                        _createTextVNode("Crowdin")
                      ]),
                      default: _withCtx(() => [
                        _createTextVNode("\n\t\t\t\t\t\t\t"),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.translation), 1 /* TEXT */),
                        _createTextVNode("\n\t\t\t\t\t\t\t")
                      ]),
                      _: 1 /* STABLE */
                    }),
                    _createVNode(FormLink, {
                      to: "https://www.patreon.com/syuilo",
                      external: ""
                    }, {
                      icon: _withCtx(() => [
                        _hoisted_8
                      ]),
                      suffix: _withCtx(() => [
                        _createTextVNode("Patreon")
                      ]),
                      default: _withCtx(() => [
                        _createTextVNode("\n\t\t\t\t\t\t\t"),
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.donate), 1 /* TEXT */),
                        _createTextVNode("\n\t\t\t\t\t\t\t")
                      ]),
                      _: 1 /* STABLE */
                    })
                  ])
                ]),
                _: 1 /* STABLE */
              }),
              (_unref(instance).repositoryUrl !== 'https://github.com/misskey-dev/misskey')
                ? (_openBlock(), _createBlock(FormSection, { key: 0 }, {
                  default: _withCtx(() => [
                    _createElementVNode("div", { class: "_gaps_s" }, [
                      _createVNode(MkInfo, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).tsx._aboutMisskey.thisIsModifiedVersion({ name: _unref(instance).name ?? _unref(host) })), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      }),
                      (_unref(instance).repositoryUrl)
                        ? (_openBlock(), _createBlock(FormLink, {
                          key: 0,
                          to: _unref(instance).repositoryUrl,
                          external: ""
                        }, {
                          icon: _withCtx(() => [
                            _hoisted_9
                          ]),
                          default: _withCtx(() => [
                            _createTextVNode("\n\t\t\t\t\t\t\t"),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.source), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]))
                        : _createCommentVNode("v-if", true),
                      (_unref(instance).providesTarball)
                        ? (_openBlock(), _createBlock(FormLink, {
                          key: 0,
                          to: `/tarball/misskey-${_unref(version)}.tar.gz`,
                          external: ""
                        }, {
                          icon: _withCtx(() => [
                            _hoisted_10
                          ]),
                          suffix: _withCtx(() => [
                            _createTextVNode("Tarball")
                          ]),
                          default: _withCtx(() => [
                            _createTextVNode("\n\t\t\t\t\t\t\t"),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.source), 1 /* TEXT */),
                            _createTextVNode("\n\t\t\t\t\t\t\t")
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["to"]))
                        : _createCommentVNode("v-if", true),
                      (!_unref(instance).repositoryUrl && !_unref(instance).providesTarball)
                        ? (_openBlock(), _createBlock(MkInfo, {
                          key: 0,
                          warn: ""
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.sourceCodeIsNotYetProvided), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : _createCommentVNode("v-if", true)
                    ])
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true),
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.projectMembers), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.contributors)
                  }, [
                    _createElementVNode("a", {
                      href: "https://github.com/syuilo",
                      target: "_blank",
                      class: _normalizeClass(_ctx.$style.contributor)
                    }, [
                      _createElementVNode("img", {
                        src: "https://avatars.githubusercontent.com/u/4439005?v=4",
                        class: _normalizeClass(_ctx.$style.contributorAvatar)
                      }),
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.contributorUsername)
                      }, "@syuilo")
                    ]),
                    _createElementVNode("a", {
                      href: "https://github.com/acid-chicken",
                      target: "_blank",
                      class: _normalizeClass(_ctx.$style.contributor)
                    }, [
                      _createElementVNode("img", {
                        src: "https://avatars.githubusercontent.com/u/20679825?v=4",
                        class: _normalizeClass(_ctx.$style.contributorAvatar)
                      }),
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.contributorUsername)
                      }, "@acid-chicken")
                    ]),
                    _createElementVNode("a", {
                      href: "https://github.com/kakkokari-gtyih",
                      target: "_blank",
                      class: _normalizeClass(_ctx.$style.contributor)
                    }, [
                      _createElementVNode("img", {
                        src: "https://avatars.githubusercontent.com/u/67428053?v=4",
                        class: _normalizeClass(_ctx.$style.contributorAvatar)
                      }),
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.contributorUsername)
                      }, "@kakkokari-gtyih")
                    ]),
                    _createElementVNode("a", {
                      href: "https://github.com/tai-cha",
                      target: "_blank",
                      class: _normalizeClass(_ctx.$style.contributor)
                    }, [
                      _createElementVNode("img", {
                        src: "https://avatars.githubusercontent.com/u/40626578?v=4",
                        class: _normalizeClass(_ctx.$style.contributorAvatar)
                      }),
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.contributorUsername)
                      }, "@tai-cha")
                    ]),
                    _createElementVNode("a", {
                      href: "https://github.com/samunohito",
                      target: "_blank",
                      class: _normalizeClass(_ctx.$style.contributor)
                    }, [
                      _createElementVNode("img", {
                        src: "https://avatars.githubusercontent.com/u/46447427?v=4",
                        class: _normalizeClass(_ctx.$style.contributorAvatar)
                      }),
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.contributorUsername)
                      }, "@samunohito")
                    ]),
                    _createElementVNode("a", {
                      href: "https://github.com/anatawa12",
                      target: "_blank",
                      class: _normalizeClass(_ctx.$style.contributor)
                    }, [
                      _createElementVNode("img", {
                        src: "https://avatars.githubusercontent.com/u/22656849?v=4",
                        class: _normalizeClass(_ctx.$style.contributorAvatar)
                      }),
                      _createElementVNode("span", {
                        class: _normalizeClass(_ctx.$style.contributorUsername)
                      }, "@anatawa12")
                    ])
                  ])
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createTextVNode("Special thanks")
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { style: "display:grid;grid-template-columns:repeat(auto-fill, minmax(130px, 1fr));grid-gap:24px;align-items:center;" }, [
                    _createElementVNode("div", null, [
                      _createElementVNode("a", {
                        style: "display: inline-block;",
                        class: "masknetwork",
                        title: "Mask Network",
                        href: "https://mask.io/",
                        target: "_blank"
                      }, [
                        _hoisted_11
                      ])
                    ]),
                    _createElementVNode("div", null, [
                      _createElementVNode("a", {
                        style: "display: inline-block;",
                        class: "xserver",
                        title: "XServer",
                        href: "https://www.xserver.ne.jp/",
                        target: "_blank"
                      }, [
                        _hoisted_12
                      ])
                    ]),
                    _createElementVNode("div", null, [
                      _createElementVNode("a", {
                        style: "display: inline-block;",
                        class: "skeb",
                        title: "Skeb",
                        href: "https://skeb.jp/",
                        target: "_blank"
                      }, [
                        _hoisted_13
                      ])
                    ]),
                    _createElementVNode("div", null, [
                      _createElementVNode("a", {
                        style: "display: inline-block;",
                        class: "pepabo",
                        title: "GMO Pepabo",
                        href: "https://pepabo.com/",
                        target: "_blank"
                      }, [
                        _hoisted_14
                      ])
                    ]),
                    _createElementVNode("div", null, [
                      _createElementVNode("a", {
                        style: "display: inline-block;",
                        class: "purpledotdigital",
                        title: "Purple Dot Digital",
                        href: "https://purpledotdigital.com/",
                        target: "_blank"
                      }, [
                        _hoisted_15
                      ])
                    ]),
                    _createElementVNode("div", null, [
                      _createElementVNode("a", {
                        style: "display: inline-block;",
                        class: "sads-llc",
                        title: "合同会社サッズ",
                        href: "https://sads-llc.co.jp/",
                        target: "_blank"
                      }, [
                        _hoisted_16
                      ])
                    ])
                  ])
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(FormSection, null, {
                label: _withCtx(() => [
                  _createVNode(_component_Mfm, { text: "$[jelly ❤]" }),
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._aboutMisskey.patrons), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", {
                    class: _normalizeClass(_ctx.$style.patronsWithIcon)
                  }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(patronsWithIcon, (patron) => {
                      return (_openBlock(), _createElementBlock("div", { class: _normalizeClass(_ctx.$style.patronWithIcon) }, [
                        _createElementVNode("img", {
                          src: patron.icon,
                          class: _normalizeClass(_ctx.$style.patronIcon)
                        }, null, 8 /* PROPS */, ["src"]),
                        _createElementVNode("span", {
                          class: _normalizeClass(_ctx.$style.patronName)
                        }, _toDisplayString(patron.name), 1 /* TEXT */)
                      ]))
                    }), 256 /* UNKEYED_FRAGMENT */))
                  ]),
                  _createElementVNode("div", { style: "margin-top: 16px; display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); grid-gap: 12px;" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(patrons, (patron) => {
                      return (_openBlock(), _createElementBlock("div", { key: patron }, _toDisplayString(patron), 1 /* TEXT */))
                    }), 128 /* KEYED_FRAGMENT */))
                  ]),
                  _createElementVNode("p", null, _toDisplayString(_unref(i18n).ts._aboutMisskey.morePatrons), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
