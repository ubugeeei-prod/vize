use super::{CONTRACT, record};
use corsa::{
    CorsaError,
    api::{
        ApiClient, ApiMode, ApiSpawnConfig, ProjectSession, SignatureResponse, SymbolResponse,
        TypeResponse,
    },
    runtime::block_on,
};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_l0::ToCompactString;

async fn request(
    session: &ProjectSession,
    dir: &Path,
    name: &str,
    frames: &mut Vec<Value>,
    method: &str,
    params: Value,
) -> corsa::Result<Value> {
    let result = session
        .client()
        .raw_json_request(method, params.clone())
        .await;
    let outcome = match &result {
        Ok(response) => json!({"kind":"success","response":response}),
        Err(error) => json!({
            "kind":"error","display":error.to_compact_string(),"debug":format!("{error:?}"),
            "rpcError":match error {
                CorsaError::Rpc(rpc) => Some(json!({"code":rpc.code,"message":rpc.message,"data":rpc.data})),
                _ => None,
            },
            "sdkStaleHandleClassification":ApiClient::is_stale_handle_error(error),
            "serverResponseBytes":null,
            "note":"SDK API error; no server response is fabricated, including client encoding refusals",
        }),
    };
    frames.push(json!({"method":method,"params":params,"outcome":outcome}));
    record(dir, name, &json!({"frames":frames}));
    result
}

pub(super) fn collect(dir: &Path, provider: &Path, variant: &str, prefix: &str) {
    let project = tempfile::tempdir().expect("isolated two-file native project");
    let root = project.path();
    let source = format!("{prefix}const formatter: Formatter = (value) => String(value ?? '');\n");
    fs::write(root.join("consumer.ts"), &source).unwrap();
    fs::write(root.join("contracts.ts"), CONTRACT).unwrap();
    let config = r#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["*.ts"]}"#;
    fs::write(root.join("tsconfig.json"), config).unwrap();
    let name = format!("native-{variant}.json");
    let mut frames = vec![json!({
        "source":{"consumer.ts":source,"contracts.ts":CONTRACT,"tsconfig.json":config},
        "provider":provider,"packetScope":"SDK-adapted complete JSON, not raw transport bytes",
    })];
    record(dir, &name, &json!({"frames":frames}));
    let mode = if vize_carton::corsa_api_mode::uses_async_json_rpc_api(provider) {
        ApiMode::AsyncJsonRpcStdio
    } else {
        ApiMode::SyncMsgpackStdio
    };
    block_on(async {
        let file = root.join("consumer.ts").to_string_lossy().into_owned();
        let session = ProjectSession::spawn(
            ApiSpawnConfig::new(provider).with_mode(mode).with_cwd(root),
            root.join("tsconfig.json").to_string_lossy().into_owned(),
            Some(file.clone().into()),
        )
        .await
        .expect("open exact provider");
        let snapshot = session.snapshot().handle.clone();
        let project_handle = session.project_handle();
        let offset = source.find("formatter:").expect("unique authored query");
        let position = source[..offset].encode_utf16().count();
        frames.push(json!({"snapshot":snapshot,"project":project_handle,"dialect":format!("{:?}",session.client().dialect()),"position":position}));
        let symbols = request(
            &session,
            dir,
            &name,
            &mut frames,
            "getSymbolsAtPositions",
            json!({
                "snapshot":snapshot,"project":project_handle,"file":file,"positions":[position],
            }),
        )
        .await
        .expect("actual symbol response");
        let symbols: Vec<Option<SymbolResponse>> = serde_json::from_value(symbols).unwrap();
        assert_eq!(symbols.len(), 1);
        let symbol = symbols[0]
            .as_ref()
            .expect("local formatter symbol")
            .id
            .clone();
        let types = request(
            &session,
            dir,
            &name,
            &mut frames,
            "getTypesOfSymbols",
            json!({
                "snapshot":snapshot,"project":project_handle,"symbols":[symbol],
            }),
        )
        .await
        .expect("local formatter type");
        let types: Vec<Option<TypeResponse>> = serde_json::from_value(types).unwrap();
        assert_eq!(types.len(), 1);
        let r#type = types[0].as_ref().expect("callable type").id.clone();
        let signatures = request(
            &session,
            dir,
            &name,
            &mut frames,
            "getSignaturesOfType",
            json!({
                "snapshot":snapshot,"project":project_handle,"type":r#type,"kind":0,
            }),
        )
        .await
        .expect("actual signature packet");
        let signatures: Vec<SignatureResponse> = serde_json::from_value(signatures).unwrap();
        assert_eq!(signatures.len(), 1, "finite one-signature input");
        let signature = &signatures[0];
        assert_eq!(signature.parameters.len(), 1);
        let params =
            json!({"snapshot":snapshot,"project":project_handle,"symbols":signature.parameters});
        let cold = request(
            &session,
            dir,
            &name,
            &mut frames,
            "getTypesOfSymbols",
            params.clone(),
        )
        .await;
        let full = request(
            &session,
            dir,
            &name,
            &mut frames,
            "getParametersOfSignature",
            json!({
                "snapshot":snapshot,"project":project_handle,"signature":signature.id,
            }),
        )
        .await
        .expect("full parameter endpoint must execute");
        let full: Vec<SymbolResponse> = serde_json::from_value(full).unwrap();
        let full_ids: Vec<_> = full.iter().map(|symbol| symbol.id.clone()).collect();
        assert_eq!(
            full_ids, signature.parameters,
            "same opaque identities, not replacement handles"
        );
        let warm = request(
            &session,
            dir,
            &name,
            &mut frames,
            "getTypesOfSymbols",
            params,
        )
        .await;
        let classification = match &cold {
            Ok(_) => "cold-query-succeeded; missing-owner cause not reproduced",
            Err(CorsaError::Protocol(message))
                if message.contains("symbol owner is unknown to this connection") =>
            {
                "client-missing-owner-refusal-before-full-parameter-query"
            }
            Err(error) if ApiClient::is_stale_handle_error(error) => {
                "other-stale-handle-before-full-parameter-query"
            }
            Err(_) => "unexpected-cold-error; collector must fail",
        };
        frames.push(json!({
            "classification":classification,"sameSnapshot":session.snapshot().handle==snapshot,
            "sameProject":session.project_handle()==project_handle,
            "sameClient":"one immutable ProjectSession borrowed for every request",
            "fullParameterIds":full_ids,
            "collectorSuccessIsNotProductQualification":true,
        }));
        record(dir, &name, &json!({"frames":frames}));
        if let Err(error) = &cold {
            assert!(
                ApiClient::is_stale_handle_error(error),
                "unexpected actual cold failure: {error}"
            );
        }
        let warm_raw = warm.expect("same request after full symbols must succeed");
        let warm: Vec<Option<TypeResponse>> = serde_json::from_value(warm_raw.clone()).unwrap();
        assert_eq!(warm.len(), 1);
        assert!(warm[0].is_some());
        if let Ok(cold) = cold {
            assert_eq!(
                cold, warm_raw,
                "already valid queries retain their complete SDK response"
            );
        }
        assert_eq!(session.snapshot().handle, snapshot);
        assert_eq!(session.project_handle(), project_handle);
        let client = session.client().clone();
        drop(session);
        client
            .close()
            .await
            .expect("close the same client after dropping its owned snapshot");
    });
    assert_eq!(
        fs::read_to_string(root.join("consumer.ts")).unwrap(),
        source
    );
    assert_eq!(
        fs::read_to_string(root.join("contracts.ts")).unwrap(),
        CONTRACT
    );
}
