use super::*;

#[test]
fn genuine_embedded_jsx_owner_does_not_authorize_copying_the_outer_module()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<!-- outer -->\n<script>export function render() { return <div/>; }</script>";
    let start = source.find("export").ok_or("start")?;
    let end = source.find("</script>").ok_or("end")?;
    let script = source.get(start..end).ok_or("block")?;
    let block = SourceRoot::new(source)
        .map_err(|_| "root")?
        .block(script, start as u32)
        .map_err(|_| "block")?;
    let observation = Parser::new(&arena, script, SourceType::jsx()).parse_observed();
    let mut producer =
        JsxFileProducer::new(&arena, observation, block, 0).map_err(|_| "producer")?;
    producer.walk().map_err(|_| "walk")?;
    let analysis =
        build_jsx_decisions(producer.finish().map_err(|_| "finish")?).map_err(|_| "L3")?;
    assert_eq!(analysis.owner().file().artifact().source(), source);
    assert_eq!(analysis.owner().file().units()[0].span.start, start as u32);
    assert_eq!(
        emit_js_module::<Recorded>(&analysis).unwrap_err().kind,
        Error::SourceWindow
    );
    Ok(())
}
