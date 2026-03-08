import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-chevron-down" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-eye" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { computed, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { AISCRIPT_VERSION } from '@syuilo/aiscript'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import MkTextarea from '@/components/MkTextarea.vue'
import MkCodeEditor from '@/components/MkCodeEditor.vue'
import MkInput from '@/components/MkInput.vue'
import MkSelect from '@/components/MkSelect.vue'
import { useMkSelect } from '@/composables/use-mkselect.js'
import { useRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'flash-edit',
  props: {
    id: { type: String, required: false }
  },
  async setup(__props: any) {

let __temp: any, __restore: any

const props = __props
const PRESET_DEFAULT = `/// @ ${AISCRIPT_VERSION}
var name = ""
Ui:render([
	Ui:C:textInput({
		label: "Your name"
		onInput: @(v) { name = v }
	})
	Ui:C:button({
		text: "Hello"
		onClick: @() {
			Mk:dialog(null, \`Hello, {name}!\`)
		}
	})
])
`;
const PRESET_OMIKUJI = `/// @ ${AISCRIPT_VERSION}
// ユーザーごとに日替わりのおみくじのプリセット
// 選択肢
let choices = [
	"ｷﾞｶﾞ吉"
	"大吉"
	"吉"
	"中吉"
	"小吉"
	"末吉"
	"凶"
	"大凶"
]
// シードが「PlayID+ユーザーID+今日の日付」である乱数生成器を用意
let random = Math:gen_rng(\`{THIS_ID}{USER_ID}{Date:year()}{Date:month()}{Date:day()}\`)
// ランダムに選択肢を選ぶ
let chosen = choices[random(0, (choices.len - 1))]
// 結果のテキスト
let result = \`今日のあなたの運勢は **{chosen}** です。\`
// UIを表示
Ui:render([
	Ui:C:container({
		align: 'center'
		children: [
			Ui:C:mfm({ text: result })
			Ui:C:postFormButton({
				text: "投稿する"
				rounded: true
				primary: true
				form: {
					text: \`{result}{Str:lf}{THIS_URL}\`
				}
			})
		]
	})
])
`;
const PRESET_SHUFFLE = `/// @ ${AISCRIPT_VERSION}
// 巻き戻し可能な文字シャッフルのプリセット
let string = "ペペロンチーノ"
let length = string.len
// 過去の結果を保存しておくやつ
var results = []
// どれだけ巻き戻しているか
var cursor = 0
@main() {
	if (cursor != 0) {
		results = results.slice(0, (cursor + 1))
		cursor = 0
	}
	let chars = []
	for (let i, length) {
		let r = Math:rnd(0, (length - 1))
		chars.push(string.pick(r))
	}
	let result = chars.join("")
	results.push(result)
	// UIを表示
	render(result)
}
@back() {
	cursor = cursor + 1
	let result = results[results.len - (cursor + 1)]
	render(result)
}
@forward() {
	cursor = cursor - 1
	let result = results[results.len - (cursor + 1)]
	render(result)
}
@render(result) {
	Ui:render([
		Ui:C:container({
			align: 'center'
			children: [
				Ui:C:mfm({ text: result })
				Ui:C:buttons({
					buttons: [{
						text: "←"
						disabled: !(results.len > 1 && (results.len - cursor) > 1)
						onClick: back
					}, {
						text: "→"
						disabled: !(results.len > 1 && cursor > 0)
						onClick: forward
					}, {
						text: "引き直す"
						onClick: main
					}]
				})
				Ui:C:postFormButton({
					text: "投稿する"
					rounded: true
					primary: true
					form: {
						text: \`{result}{Str:lf}{THIS_URL}\`
					}
				})
			]
		})
	])
}
main()
`;
const PRESET_QUIZ = `/// @ ${AISCRIPT_VERSION}
let title = '地理クイズ'
let qas = [{
	q: 'オーストラリアの首都は？'
	choices: ['シドニー', 'キャンベラ', 'メルボルン']
	a: 'キャンベラ'
	aDescription: '最大の都市はシドニーですが首都はキャンベラです。'
}, {
	q: '国土面積2番目の国は？'
	choices: ['カナダ', 'アメリカ', '中国']
	a: 'カナダ'
	aDescription: '大きい順にロシア、カナダ、アメリカ、中国です。'
}, {
	q: '二重内陸国ではないのは？'
	choices: ['リヒテンシュタイン', 'ウズベキスタン', 'レソト']
	a: 'レソト'
	aDescription: 'レソトは(一重)内陸国です。'
}, {
	q: '閘門がない運河は？'
	choices: ['キール運河', 'スエズ運河', 'パナマ運河']
	a: 'スエズ運河'
	aDescription: 'スエズ運河は高低差がないので閘門はありません。'
}]
let qaEls = [Ui:C:container({
	align: 'center'
	children: [
		Ui:C:text({
			size: 1.5
			bold: true
			text: title
		})
	]
})]
var qn = 0
each (let qa, qas) {
	qn += 1
	qa.id = Util:uuid()
	qaEls.push(Ui:C:container({
		align: 'center'
		bgColor: '#000'
		fgColor: '#fff'
		padding: 16
		rounded: true
		children: [
			Ui:C:text({
				text: \`Q{qn} {qa.q}\`
			})
			Ui:C:select({
				items: qa.choices.map(@(c) {{ text: c, value: c }})
				onChange: @(v) { qa.userAnswer = v }
			})
			Ui:C:container({
				children: []
			}, \`{qa.id}:a\`)
		]
	}, qa.id))
}
@finish() {
	var score = 0
	each (let qa, qas) {
		let correct = qa.userAnswer == qa.a
		if (correct) score += 1
		let el = Ui:get(\`{qa.id}:a\`)
		el.update({
			children: [
				Ui:C:text({
					size: 1.2
					bold: true
					color: if (correct) '#f00' else '#00f'
					text: if (correct) '🎉正解' else '不正解'
				})
				Ui:C:text({
					text: qa.aDescription
				})
			]
		})
	}
	let result = \`{title}の結果は{qas.len}問中{score}問正解でした。\`
	Ui:get('footer').update({
		children: [
			Ui:C:postFormButton({
				text: '結果を共有'
				rounded: true
				primary: true
				form: {
					text: \`{result}{Str:lf}{THIS_URL}\`
				}
			})
		]
	})
}
qaEls.push(Ui:C:container({
	align: 'center'
	children: [
		Ui:C:button({
			text: '答え合わせ'
			primary: true
			rounded: true
			onClick: finish
		})
	]
}, 'footer'))
Ui:render(qaEls)
`;
const PRESET_TIMELINE = `/// @ ${AISCRIPT_VERSION}
// APIリクエストを行いローカルタイムラインを表示するプリセット
@fetch() {
	Ui:render([
		Ui:C:container({
			align: 'center'
			children: [
				Ui:C:text({ text: "読み込み中..." })
			]
		})
	])
	// タイムライン取得
	let notes = Mk:api("notes/local-timeline", {})
	// それぞれのノートごとにUI要素作成
	let noteEls = []
	each (let note, notes) {
		// 表示名を設定していないアカウントはidを表示
		let userName = if Core:type(note.user.name) == "str" note.user.name else note.user.username
		// リノートもしくはメディア・投票のみで本文が無いノートに代替表示文を設定
		let noteText = if Core:type(note.text) == "str" note.text else "（リノートもしくはメディア・投票のみのノート）"
		let el = Ui:C:container({
			bgColor: "#444"
			fgColor: "#fff"
			padding: 10
			rounded: true
			children: [
				Ui:C:mfm({
					text: userName
					bold: true
				})
				Ui:C:mfm({
					text: noteText
				})
			]
		})
		noteEls.push(el)
	}
	// UIを表示
	Ui:render([
		Ui:C:text({ text: "ローカル タイムライン" })
		Ui:C:button({
			text: "更新"
			onClick: @() {
				fetch()
			}
		})
		Ui:C:container({
			children: noteEls
		})
	])
}
fetch()
`;
const router = useRouter();
const flash = ref<Misskey.entities.Flash | null>(null);
if (props.id) {
	flash.value = await misskeyApi('flash/show', {
		flashId: props.id,
	});
}
const title = ref(flash.value?.title ?? 'New Play');
const summary = ref(flash.value?.summary ?? '');
const permissions = ref([]); // not implemented yet
const {
	model: visibility,
	def: visibilityDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts.public, value: 'public' },
		{ label: i18n.ts.private, value: 'private' },
	],
	initialValue: flash.value?.visibility ?? 'public',
});
const script = ref(flash.value?.script ?? PRESET_DEFAULT);
function selectPreset(ev: PointerEvent) {
	os.popupMenu([{
		text: 'Omikuji',
		action: () => {
			script.value = PRESET_OMIKUJI;
		},
	}, {
		text: 'Shuffle',
		action: () => {
			script.value = PRESET_SHUFFLE;
		},
	}, {
		text: 'Quiz',
		action: () => {
			script.value = PRESET_QUIZ;
		},
	}, {
		text: 'Timeline viewer',
		action: () => {
			script.value = PRESET_TIMELINE;
		},
	}], ev.currentTarget ?? ev.target);
}
async function save() {
	if (flash.value != null) {
		os.apiWithDialog('flash/update', {
			flashId: flash.value.id,
			title: title.value,
			summary: summary.value,
			permissions: permissions.value,
			script: script.value,
			visibility: visibility.value,
		});
	} else {
		const created = await os.apiWithDialog('flash/create', {
			title: title.value,
			summary: summary.value,
			permissions: permissions.value,
			script: script.value,
			visibility: visibility.value,
		});
		router.push('/play/:id/edit', {
			params: {
				id: created.id,
			},
		});
	}
}
function show() {
	if (flash.value == null) {
		os.alert({
			text: 'Please save',
		});
	} else {
		os.pageWindow(`/play/${flash.value.id}`);
	}
}
async function del() {
	if (flash.value == null) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.tsx.deleteAreYouSure({ x: flash.value.title }),
	});
	if (canceled) return;
;(
  ([__temp,__restore] = _withAsyncContext(() => os.apiWithDialog('flash/delete', {)),
  await __temp,
  __restore()
)
		flashId: flash.value.id,
	});
	router.push('/play');
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: flash.value ? `${i18n.ts._play.edit}: ${flash.value.title}` : i18n.ts._play.new,
}));

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      footer: _withCtx(() => [
        _createElementVNode("div", {
          class: _normalizeClass(_ctx.$style.footer)
        }, [
          _createElementVNode("div", { class: "_spacer" }, [
            _createElementVNode("div", { class: "_buttons" }, [
              _createVNode(MkButton, {
                primary: "",
                onClick: save
              }, {
                default: _withCtx(() => [
                  _hoisted_2,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkButton, { onClick: show }, {
                default: _withCtx(() => [
                  _hoisted_3,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.show), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              (flash.value)
                ? (_openBlock(), _createBlock(MkButton, {
                  key: 0,
                  danger: "",
                  onClick: del
                }, {
                  default: _withCtx(() => [
                    _hoisted_4,
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true)
            ])
          ])
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [
          _createElementVNode("div", { class: "_gaps" }, [
            _createVNode(MkInput, {
              modelValue: title.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((title).value = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._play.title), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]),
            _createVNode(MkSelect, {
              items: _unref(visibilityDef),
              modelValue: _unref(visibility),
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((visibility).value = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.visibility), 1 /* TEXT */)
              ]),
              caption: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._play.visibilityDescription), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["items", "modelValue"]),
            _createVNode(MkTextarea, {
              mfmAutocomplete: true,
              mfmPreview: true,
              modelValue: summary.value,
              "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((summary).value = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._play.summary), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["mfmAutocomplete", "mfmPreview", "modelValue"]),
            _createVNode(MkButton, {
              primary: "",
              onClick: selectPreset
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.selectFromPresets), 1 /* TEXT */),
                _hoisted_1
              ]),
              _: 1 /* STABLE */
            }),
            _createVNode(MkCodeEditor, {
              lang: "is",
              modelValue: script.value,
              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((script).value = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts._play.script), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"])
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
