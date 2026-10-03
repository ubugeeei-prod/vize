use vize_l1_to_l2::native_file::NativeTemplateObservation;
use vize_l3::decision::dom::vue::{NativeVueRenderAnalysis, VueReadKind};
use vize_l4::write::document::EmitDocument;

pub fn check(
    analysis: &NativeVueRenderAnalysis<'_, '_, '_, '_, '_>,
    template: &NativeTemplateObservation<'_>,
    document: &EmitDocument,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut expected_names = Vec::new();
    for embed in template.embeds() {
        let row = analysis
            .expression(embed.node.ok_or("native node")?)
            .ok_or("read row")?;
        let resolution = row.resolution();
        let table = resolution.table().ok_or("complete table")?;
        if row.reads().len() != table.occurrences().len() {
            return Err("incomplete read list".into());
        }
        for (read, occurrence) in row.reads().iter().zip(table.occurrences()) {
            if !core::ptr::eq(read.occurrence(), occurrence)
                || !read.binding().same_owner(
                    resolution
                        .binding(occurrence.binding)
                        .ok_or("file binding")?,
                )
                || read.kind() != VueReadKind::SetupLet
            {
                return Err("foreign or unclassified occurrence".into());
            }
            let authored = table
                .expression()
                .authored_span(occurrence.span)
                .ok_or("authored span")?;
            if source.get(authored.start as usize..authored.end as usize) != Some(occurrence.name) {
                return Err("authored identifier spelling".into());
            }
            let name = occurrence.name;
            let accessor = vize_l0::cstr!("$setup.{name}");
            if document
                .links()
                .iter()
                .filter(|link| {
                    link.authored == authored
                        && link.name.as_deref() == Some(occurrence.name)
                        && document
                            .as_str()
                            .get(link.generated.start as usize..link.generated.end as usize)
                            == Some(accessor.as_str())
                })
                .count()
                != 1
            {
                return Err("complete named accessor link".into());
            }
            if !expected_names.contains(&occurrence.name) {
                expected_names.push(occurrence.name);
            }
            for gap in [
                vize_l0::Span::new(0, occurrence.span.start),
                vize_l0::Span::new(occurrence.span.end, table.expression().source.len() as u32),
            ] {
                if gap.start == gap.end {
                    continue;
                }
                let bytes = table
                    .expression()
                    .source
                    .get(gap.start as usize..gap.end as usize)
                    .ok_or("comment/trivia window")?;
                let authored_gap = table
                    .expression()
                    .authored_span(gap)
                    .ok_or("comment projection")?;
                if document
                    .links()
                    .iter()
                    .filter(|link| {
                        link.name.is_none()
                            && link.authored == authored_gap
                            && document
                                .as_str()
                                .get(link.generated.start as usize..link.generated.end as usize)
                                == Some(bytes)
                    })
                    .count()
                    != 1
                {
                    return Err("complete retained comment/trivia link".into());
                }
            }
        }
    }
    for link in document.links() {
        if source
            .get(link.authored.start as usize..link.authored.end as usize)
            .is_none()
            || document
                .as_str()
                .get(link.generated.start as usize..link.generated.end as usize)
                .is_none()
        {
            return Err("out-of-owner authored/generated link".into());
        }
    }
    let map: serde_json::Value =
        serde_json::from_str(&document.source_map("VueSetupLet.vue", source))?;
    if map.get("names") != Some(&serde_json::json!(expected_names)) {
        return Err("complete native named map".into());
    }
    // Vue's raw reference maps have no setup names. Our complete authored
    // links remain richer, so full upstream-map equivalence is not asserted.
    Ok(())
}
