use super::super::super::super::names::NamesConfiguration;
use super::{
    Arc, NavigationRefusal, Position, SOURCE, block_on, enable, linked_ticket, project, ranges, uri,
};
use vize_l0::config::VueDialect;

#[test]
fn ready_actual_names_response_rejects_route_aba_inside_original_document_publication_guard() {
    let (state, project) = project(SOURCE);
    enable(&state);
    let ticket = NamesConfiguration::Linked(linked_ticket(&state));
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let ready = block_on(query.run(|snapshot| async {
        let worker = project.names_worker(snapshot, &ticket)?;
        worker.linked_editing(Position::new(0, 11)).await
    }))
    .unwrap();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({"nativeLinkedEditing":false})));
    enable(&state);
    assert_eq!(
        ready.publish(|response| project
            .source
            .with_names_configuration(&ticket, |_| response)
            .and_then(|response| response)),
        Ok(Err(NavigationRefusal::NamesRouteChanged))
    );
    assert_eq!(
        block_on(project.linked_editing(&uri(), Position::new(0, 11))),
        Ok(ranges((0, 11, 12), (0, 15, 16)))
    );
}

#[test]
fn actual_configuration_writer_waits_for_ready_names_publication_without_recursive_reads() {
    let (state, project) = project(SOURCE);
    let ticket = project.source.capture_names_parser().unwrap();
    let (query, _) = project.source.begin_query(&uri()).unwrap();
    let ready = block_on(query.run(|snapshot| async {
        let worker = project.names_worker(snapshot, &ticket)?;
        worker.linked_editing(Position::new(0, 11)).await
    }))
    .unwrap();
    let (start, started) = std::sync::mpsc::channel();
    let (attempt, attempted) = std::sync::mpsc::channel();
    let (finish, finished) = std::sync::mpsc::channel();
    let writer_state = Arc::clone(&state);
    let writer = std::thread::spawn(move || {
        started.recv().unwrap();
        attempt.send(()).unwrap();
        writer_state.set_dialect_config(Some(VueDialect::PetiteVue));
        finish.send(()).unwrap();
    });
    assert_eq!(
        ready.publish(|response| project
            .source
            .with_names_configuration(&ticket, |_| {
                start.send(()).unwrap();
                attempted.recv().unwrap();
                assert!(matches!(
                    finished.try_recv(),
                    Err(std::sync::mpsc::TryRecvError::Empty)
                ));
                response
            })
            .and_then(|response| response)),
        Ok(Ok(ranges((0, 11, 12), (0, 15, 16))))
    );
    writer.join().unwrap();
    finished.recv().unwrap();
    assert_eq!(
        project.source.with_names_configuration(&ticket, |_| ()),
        Err(NavigationRefusal::ConfigurationChanged)
    );
}
