mod support;

use std::time::{Duration, Instant};

use corsa::{
    api::{
        ApiClient, ApiMode, ApiSpawnConfig, ManagedSnapshot, ProjectHandle, UpdateSnapshotParams,
    },
    runtime::block_on,
};

#[test]
fn real_corsa_msgpack_roundtrip_smoke() {
    block_on(async {
        let Some(config) = support::real_api_config(ApiMode::SyncMsgpackStdio) else {
            return;
        };
        let client = ApiClient::spawn(config).await.unwrap();
        let dataset_wire = support::real_dataset().display().to_string();
        let init = client.initialize().await.unwrap();
        assert!(!init.current_directory.is_empty());
        let config = client
            .parse_config_file(dataset_wire.as_str())
            .await
            .unwrap();
        assert!(!config.file_names.is_empty());
        let primary_file = config
            .file_names
            .iter()
            .find(|file| !file.ends_with(".d.ts"))
            .cloned()
            .unwrap_or_else(|| config.file_names[0].clone());
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some(dataset_wire.clone()),
                file_changes: None,
                overlay_changes: None,
            })
            .await
            .unwrap();
        let project = snapshot.projects[0].id.clone();
        let source = client
            .get_source_file(
                snapshot.handle.clone(),
                project.clone(),
                primary_file.as_str(),
            )
            .await
            .unwrap();
        assert!(source.is_some());
        let string_type = client
            .get_string_type(snapshot.handle.clone(), project.clone())
            .await
            .unwrap();
        let rendered = client
            .type_to_string(
                snapshot.handle.clone(),
                project,
                string_type.id.clone(),
                None,
                None,
            )
            .await
            .unwrap();
        assert!(!rendered.is_empty());
        snapshot.release().await.unwrap();
        client.close().await.unwrap();
    });
}

#[test]
fn real_corsa_jsonrpc_roundtrip_smoke() {
    block_on(async {
        let Some(config) = support::real_api_config(ApiMode::AsyncJsonRpcStdio) else {
            return;
        };
        let client = ApiClient::spawn(config).await.unwrap();
        let dataset_wire = support::real_dataset().display().to_string();
        let _ = client.initialize().await.unwrap();
        let config = client
            .parse_config_file(dataset_wire.as_str())
            .await
            .unwrap();
        assert!(!config.file_names.is_empty());
        let snapshot = client
            .update_snapshot(UpdateSnapshotParams {
                open_project: Some(dataset_wire),
                file_changes: None,
                overlay_changes: None,
            })
            .await
            .unwrap();
        assert!(!snapshot.projects.is_empty());
        snapshot.release().await.unwrap();
        client.close().await.unwrap();
    });
}

#[test]
fn real_corsa_msgpack_hot_path_stays_ahead_of_jsonrpc() {
    block_on(async {
        let Some(msgpack_config) = support::real_api_config(ApiMode::SyncMsgpackStdio) else {
            return;
        };
        let Some(jsonrpc_config) = support::real_api_config(ApiMode::AsyncJsonRpcStdio) else {
            return;
        };
        let dataset_wire = support::real_dataset().display().to_string();

        let msgpack = prepare_session(msgpack_config, dataset_wire.as_str()).await;
        let jsonrpc = prepare_session(jsonrpc_config, dataset_wire.as_str()).await;

        let msgpack_elapsed =
            measure_hot_get_string_type(&msgpack.client, &msgpack.snapshot, &msgpack.project).await;
        let jsonrpc_elapsed =
            measure_hot_get_string_type(&jsonrpc.client, &jsonrpc.snapshot, &jsonrpc.project).await;

        assert!(
            msgpack_elapsed <= jsonrpc_elapsed.mul_f32(1.5),
            "msgpack hot path regressed: {:?} vs {:?}",
            msgpack_elapsed,
            jsonrpc_elapsed
        );

        msgpack.snapshot.release().await.unwrap();
        msgpack.client.close().await.unwrap();
        jsonrpc.snapshot.release().await.unwrap();
        jsonrpc.client.close().await.unwrap();
    });
}

struct Session {
    client: ApiClient,
    snapshot: ManagedSnapshot,
    project: ProjectHandle,
}

async fn prepare_session(config: ApiSpawnConfig, dataset_wire: &str) -> Session {
    let client = ApiClient::spawn(config).await.unwrap();
    let snapshot = client
        .update_snapshot(UpdateSnapshotParams {
            open_project: Some(dataset_wire.to_owned()),
            file_changes: None,
            overlay_changes: None,
        })
        .await
        .unwrap();
    let project = snapshot.projects[0].id.clone();
    Session {
        client,
        snapshot,
        project,
    }
}

async fn measure_hot_get_string_type(
    client: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectHandle,
) -> Duration {
    let mut samples = [Duration::ZERO; 24];
    let mut index = 0;
    while index < samples.len() {
        let started = Instant::now();
        let _ = client
            .get_string_type(snapshot.handle.clone(), project.clone())
            .await
            .unwrap();
        samples[index] = started.elapsed();
        index += 1;
    }
    samples.sort_unstable();
    samples[samples.len() / 2]
}
