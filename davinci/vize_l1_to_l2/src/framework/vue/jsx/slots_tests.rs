use vize_l2::module::Exec;

use super::JsxRefusal;
use super::tests::lower;

#[test]
fn component_children_are_control_flow_or_slot_templates() {
    assert_eq!(
        lower(
            "import Card from './c';\nconst A = <Card>{a ? <i /> : <b />}</Card>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=5 <C:component :is> if[ a? <i> else: <b> ] ")
    );
    assert_eq!(
        lower(
            "import Card from './c';\nconst A = <Card>\n  {(item) => <li>{item}</li>}\n</Card>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok(
            "ops=6 <C:component :is> <template #Some(Static(\"default\"))(Some(\"item\"))> <li> {item} scope@3=[\"item\" ] "
        )
    );
    assert_eq!(
        lower(
            "const A = <Card>{{ header: () => <h1>T</h1>, 'footer': (p) => <><i>{p}</i><b /></>, body() { return ok ? <i /> : <b />; } }}</Card>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=15 <C:Card> <template #Some(Static(\"header\"))(None)> <h1> \"T\" <template #Some(Static(\"footer\"))(Some(\"p\"))> <i> {p} <b> <template #Some(Static(\"body\"))(None)> if[ ok? <i> else: <b> ] scope@6=[\"p\" ] ")
    );
    for source in [
        "const A = <Card>{() => 'x'}</Card>;",
        "const A = <Card>{() => a || <i />}</Card>;",
        "const A = <Card>{(a, b) => <i />}</Card>;",
        "const A = <Card>{(...xs) => <i />}</Card>;",
        "const A = <Card>{() => { const x = 1; return <i />; }}</Card>;",
        "const A = <Card>{() => { return; }}</Card>;",
        "const A = <Card>{{ ...rest }}</Card>;",
        "const A = <Card>{{ header }}</Card>;",
        "const A = <Card>{{ [key]: () => <i /> }}</Card>;",
        "const A = <Card>{{ 1: () => <i /> }}</Card>;",
        "const A = <Card>{x}</Card>;",
    ] {
        assert_eq!(
            lower(source, Exec::ReRender),
            Err(JsxRefusal::Child),
            "{source}"
        );
    }
    assert_eq!(
        lower(
            "const A = <Card v-slots={slots} title=\"t\" x={1}>text</Card>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=4 <C:Card :x slots=slots> \"text\" ")
    );
    assert_eq!(
        lower("const A = <Card vSlots={(own.slots)} />;", Exec::ReRender).as_deref(),
        Ok("ops=2 <C:Card slots=own.slots> ")
    );
    assert_eq!(
        lower(
            "const A = <Card v-slots={{ header: (p) => <h1>{p}</h1> }}>text</Card>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok(
            "ops=6 <C:Card> \"text\" <template #Some(Static(\"header\"))(Some(\"p\"))> <h1> {p} scope@3=[\"p\" ] "
        )
    );
    for source in [
        "const A = <Card v-slots={s} vSlots={t} />;",
        "const A = <Card v-slots={() => <i />} />;",
        "const A = <Card v-slots={1} />;",
        "const A = <Card v-slots={[s]} />;",
        "const A = <Card v-slots={make(<i />)} />;",
        "const A = <Card v-slots />;",
        "const A = <x-card v-slots={s} />;",
        "const A = <Card v-slots={{ ...s }} />;",
        "const A = <Card v-slots={{ a: s.a }} />;",
    ] {
        assert_eq!(
            lower(source, Exec::ReRender),
            Err(JsxRefusal::Attribute),
            "{source}"
        );
    }
}
