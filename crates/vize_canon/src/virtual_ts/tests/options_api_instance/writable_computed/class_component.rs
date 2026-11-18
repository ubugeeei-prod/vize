use super::options_api_declarations;

/// A class component's `@PropSync`, `@ModelSync` and `@VModel` fields are
/// computed get/set pairs over a prop (#13335), and a `get`/`set` accessor
/// pair is a writable computed, so a template assignment to any of them is
/// valid. A getter-only accessor and a method stay read-only, and a plain
/// field is reactive data.
#[test]
fn class_component_sync_fields_and_setters_emit_var() {
    let script = r#"import { Options, Vue } from 'vue-class-component'
import { Prop, PropSync, ModelSync, VModel } from 'vue-property-decorator'

@Options({})
export default class Panel extends Vue {
  @Prop({ required: true }) readonly title!: string
  @PropSync('open', { type: Boolean }) syncedOpen!: boolean
  @ModelSync('checked', 'change', { type: Boolean }) checkedValue!: boolean
  @VModel({ type: String }) text!: string
  count = 0
  get label(): string { return this.title }
  get ratio(): number { return this.count }
  set ratio(value: number) { this.count = value }
  toggle() { this.syncedOpen = !this.syncedOpen }
}
"#;
    let declarations = options_api_declarations(
        script,
        r#"<div @click="syncedOpen = !syncedOpen; checkedValue = true; text = 'a'; ratio = 2; count++">{{ label }} {{ toggle }}</div>"#,
    );
    assert_eq!(
        declarations,
        [
            "  var checkedValue: $OptionsBinding<typeof __default__, \"checkedValue\"> = (void 0) as any;",
            "  var count: $OptionsBinding<typeof __default__, \"count\"> = (void 0) as any;",
            "  const label: $OptionsBinding<typeof __default__, \"label\"> = (void 0) as any;",
            "  var ratio: $OptionsBinding<typeof __default__, \"ratio\"> = (void 0) as any;",
            "  var syncedOpen: $OptionsBinding<typeof __default__, \"syncedOpen\"> = (void 0) as any;",
            "  var text: $OptionsBinding<typeof __default__, \"text\"> = (void 0) as any;",
            "  const toggle: $OptionsBinding<typeof __default__, \"toggle\"> = (void 0) as any;",
        ]
    );
}
