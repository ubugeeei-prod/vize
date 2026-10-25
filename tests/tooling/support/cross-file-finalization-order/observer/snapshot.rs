use serde_json::{Value, json};
use std::collections::BTreeMap;
use vize_croquis::{ScopeData, facts};
use vize_croquis_cf::{CrossFileAnalyzer, DependencyEdge};
use vize_l0::cstr;

pub fn snapshot(analyzer: &mut CrossFileAnalyzer) -> Value {
    let mut result = analyzer.analyze();
    let actual_result = cstr!("{result:#?}");
    // Analysis time is a separately retained observation. No semantic field is
    // removed; complete original Debug data remains alongside this projection.
    let analysis_time_ms = std::mem::replace(&mut result.stats.analysis_time_ms, 0.0);
    let semantic_result = cstr!("{result:#?}");
    let registry: Vec<_> = analyzer.registry().iter().map(|entry| {
        let imports: Vec<_> = entry.analysis.scopes.iter().filter_map(|scope| {
            if let ScopeData::ExternalModule(data) = scope.data() {
                Some(json!({"source": data.source, "typeOnly": data.is_type_only}))
            } else { None }
        }).collect();
        let usage_targets: Vec<_> = vize_croquis_cf::facts::component_usage_targets(analyzer.registry(), entry.id).into_iter().map(|id| id.as_u32()).collect();
        let used_components = facts::used_component_name_list(&entry.analysis);
        let index_lookups: Vec<_> = used_components.iter().map(|name| json!({"name":name,"target":analyzer.graph().find_by_component(name).map(|id|id.as_u32())})).collect();
        json!({
            "id":entry.id.as_u32(), "path":entry.path, "filename":entry.filename,
            "sourceHash":entry.source_hash,"mtime":cstr!("{:?}",entry.mtime),
            "vue":entry.is_vue_sfc,"componentName":entry.component_name,
            "storeFactories":cstr!("{:?}",entry.pinia_stores),"imports":imports,
            "semanticSnapshot":entry.analysis.semantic_snapshot(),"componentTargets":usage_targets,
            "componentIndexLookups":index_lookups,
        })
    }).collect();
    let mut nodes = Vec::new();
    let mut forward = Vec::new();
    let mut reverse = Vec::new();
    let mut imports_by_kind = Vec::new();
    for node in analyzer.graph().nodes() {
        let outgoing: Vec<_> = node
            .imports
            .iter()
            .map(|(id, kind)| (id.as_u32(), kind.display_name()))
            .collect();
        let incoming: Vec<_> = node
            .importers
            .iter()
            .map(|(id, kind)| (id.as_u32(), kind.display_name()))
            .collect();
        let mut grouped: BTreeMap<&str, Vec<u32>> = BTreeMap::new();
        for (id, kind) in &node.importers {
            grouped
                .entry(kind.display_name())
                .or_default()
                .push(id.as_u32());
        }
        for (id, kind) in &node.imports {
            forward.push((node.file_id.as_u32(), id.as_u32(), kind.display_name()));
        }
        for (id, kind) in &node.importers {
            reverse.push((id.as_u32(), node.file_id.as_u32(), kind.display_name()));
        }
        imports_by_kind
            .push(json!({"id":node.file_id.as_u32(),"outgoing":outgoing,"incomingByKind":grouped}));
        nodes.push(json!({"id":node.file_id.as_u32(),"path":node.path,"componentName":node.component_name,"entry":node.is_entry,"exports":node.exports,"outgoing":outgoing,"incoming":incoming}));
    }
    forward.sort();
    reverse.sort();
    let components: Vec<_> = analyzer
        .graph()
        .component_usage()
        .map(|(from, to)| (from.as_u32(), to.as_u32()))
        .collect();
    let circular: Vec<_> = analyzer
        .graph()
        .circular_dependencies()
        .iter()
        .map(|cycle| cycle.iter().map(|id| id.as_u32()).collect::<Vec<_>>())
        .collect();
    json!({
        "registry":registry,"rawNodes":nodes,"allForwardRelations":forward,"allReverseRelations":reverse,
        "orderedPerKind":imports_by_kind,"orderedComponentRelations":components,
        "entries":analyzer.graph().entries().iter().map(|id|id.as_u32()).collect::<Vec<_>>(),
        "cycles":circular,"analysisCompleteRaw":actual_result,"analysisTimeMs":analysis_time_ms,
        "analysisCompleteWithTimeSplit":semantic_result,
    })
}

pub fn comparison(first: &Value, second: &Value) -> Value {
    let same = |name: &str| first.get(name) == second.get(name);
    json!({
        "registry":same("registry"),"rawGraph":same("rawNodes"),
        "allForwardRelations":same("allForwardRelations"),"allReverseRelations":same("allReverseRelations"),
        "orderedPerKind":same("orderedPerKind"),"orderedComponentRelations":same("orderedComponentRelations"),
        "entries":same("entries"),"cycles":same("cycles"),
        "completeSemanticAnalysis":same("analysisCompleteWithTimeSplit"),
    })
}

pub fn stale_import(
    analyzer: &CrossFileAnalyzer,
    from: &std::path::Path,
    target: &std::path::Path,
) -> bool {
    let Some(from) = analyzer.registry().get_id(from) else {
        return false;
    };
    let Some(target) = analyzer.registry().get_id(target) else {
        return false;
    };
    analyzer.graph().dependencies(from).any(|(id, kind)| {
        id == target && matches!(kind, DependencyEdge::Import | DependencyEdge::TypeImport)
    })
}
