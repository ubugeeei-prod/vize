use crate::{Result, common, jsonrpc};
use base64::Engine as _;
use corsa::jsonrpc::{RawMessage, RequestId, RpcResponseError};
use serde_json::{Value, json};
use std::{
    fs::{OpenOptions, create_dir_all},
    io::{BufReader, BufWriter, Write as _},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// Type handle the mock pretends it has evicted from its snapshot registry, so
/// integration tests can drive the stale-handle degradation path in
/// `getTypeArguments`.
const STALE_TYPE_HANDLE: &str = "t00000000000000ff";
const MAPPED_UTILITY_TYPE_HANDLE: &str = "t00000000000000ee";
const MAPPED_UTILITY_ARGUMENT_SYMBOL: &str = "s00000000000000ee";
const MAPPED_UTILITY_SPARSE_ARGUMENT: &str = "t00000000000000e1";
const MAPPED_UTILITY_STRUCTURAL_ARGUMENT: &str = "t00000000000000e2";
static RELEASE_FAILURE_USED: AtomicBool = AtomicBool::new(false);

pub fn run(cwd: String, callbacks: Vec<String>) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    loop {
        let Some(message) = jsonrpc::read_message(&mut reader)? else {
            return Ok(());
        };
        let method = message.method.unwrap_or_default();
        record_method(method.as_str());
        let id = message.id.clone();
        let params = message.params.unwrap_or(Value::Null);
        record_params(method.as_str(), &params);
        if let (Some(id), Some(error)) = (id.clone(), stale_handle_error(method.as_str(), &params))
        {
            jsonrpc::write_message(&mut writer, &RawMessage::error(id, error))?;
            continue;
        }
        if let (Some(id), Some(error)) = (id.clone(), release_error(method.as_str())) {
            jsonrpc::write_message(&mut writer, &RawMessage::error(id, error))?;
            continue;
        }
        if let (Some(id), Some(error)) = (id.clone(), missing_project_error(&params)) {
            jsonrpc::write_message(&mut writer, &RawMessage::error(id, error))?;
            continue;
        }
        let response = match method.as_str() {
            "initialize" => Some(json!({
                "useCaseSensitiveFileNames": true,
                "currentDirectory": cwd,
            })),
            "describeCapabilities" => Some(common::capabilities()),
            "batchRequests" => Some(batch_requests(params)),
            "parseConfigFile" => Some(parse_config(&mut reader, &mut writer, &callbacks, params)?),
            "updateSnapshot" => Some(common::snapshot_from_update_params(
                "/workspace/tsconfig.json",
                &params,
            )),
            "getDefaultProjectForFile" => Some(common::project("/workspace/tsconfig.json")),
            "getSourceFile" => Some(common::encoded(b"source-file")),
            "getDiagnosticsForSnapshot" => Some(common::snapshot_diagnostics(json!(
                "/workspace/src/index.ts"
            ))),
            "getDiagnosticsForProject" => Some(common::project_diagnostics(json!(
                "/workspace/src/index.ts"
            ))),
            "getDiagnosticsForFile" => Some(common::file_diagnostics(
                params
                    .get("file")
                    .cloned()
                    .unwrap_or_else(|| json!("/workspace/src/index.ts")),
            )),
            "getHoverAtPosition" => Some(common::hover()),
            "getDefinitionAtPosition" => Some(common::definition()),
            "getReferencesAtPosition" => Some(common::references()),
            "getRenameAtPosition" => Some(common::rename(
                params
                    .get("newName")
                    .and_then(Value::as_str)
                    .unwrap_or("renamedValue"),
            )),
            "getCompletionAtPosition" => Some(common::completion()),
            "getSymbolAtPosition" | "getSymbolAtLocation" | "resolveName" => {
                Some(common::symbol("value"))
            }
            "getSymbolsAtPositions" | "getSymbolsAtLocations" => {
                Some(json!([common::symbol("value"), Value::Null]))
            }
            "getDeclaredTypeOfSymbol"
                if symbol_param(&params) == Some(MAPPED_UTILITY_ARGUMENT_SYMBOL) =>
            {
                Some(mapped_utility_argument(MAPPED_UTILITY_STRUCTURAL_ARGUMENT))
            }
            // Checker endpoints that map one type to another.
            "getApparentType"
            | "getNonNullableType"
            | "getWidenedType"
            | "getBaseConstraintOfType"
            | "getFreshTypeOfType"
            | "getRegularTypeOfType"
            | "getTrueTypeOfConditionalType"
            | "getFalseTypeOfConditionalType"
            | "getTypeFromTypeNode"
            | "getParameterType" => Some(common::type_response("t0000000000000001")),
            // Checker predicates. Answering these is what lets the binding stop
            // deciding type shape by parsing rendered type text.
            "isArrayType" | "isTupleType" | "isArrayLikeType" | "isTypeAssignableTo" => {
                Some(json!(true))
            }
            "getAliasTypeArgumentsOfType" | "getTypeParametersOfSignature" => {
                Some(json!([common::type_response("t0000000000000001")]))
            }
            "getResolvedSignature" | "getSignatureFromDeclaration" | "getTargetOfSignature" => {
                Some(common::signature())
            }
            "getAliasSymbolOfType"
            | "getAliasedSymbol"
            | "getImmediateAliasedSymbol"
            | "getPropertyOfType"
            | "getMemberInModuleExports"
            | "getExportSpecifierLocalTargetSymbol" => Some(common::symbol("value")),
            "getExportsOfModule" => Some(json!([common::symbol("value")])),
            "getJsDocTags" => Some(json!([{ "name": "deprecated", "text": "use other" }])),
            "getDocumentationComment" => Some(json!("docs")),
            "getConstantValue" => Some(json!(42)),
            "getWellKnownSymbols" => Some(json!({
                "unknown": "s0000000000000001",
                "undefined": "s0000000000000002",
                "arguments": "s0000000000000003",
            })),
            "getTypeOfSymbol"
            | "getDeclaredTypeOfSymbol"
            | "getTypeAtLocation"
            | "getTypeAtPosition"
            | "getContextualType"
            | "getBaseTypeOfLiteralType"
            | "getTypeOfSymbolAtLocation"
            | "getTargetOfType"
            | "getObjectTypeOfType"
            | "getIndexTypeOfType"
            | "getCheckTypeOfType"
            | "getExtendsTypeOfType"
            | "getBaseTypeOfType"
            | "getConstraintOfType"
            | "getReturnTypeOfSignature"
            | "getRestTypeOfSignature"
            | "getConstraintOfTypeParameter" => Some(common::type_response("t0000000000000001")),
            "getTypesOfSymbols" | "getTypeAtLocations" | "getTypesAtPositions" => {
                let count = params
                    .as_object()
                    .and_then(|value| {
                        value
                            .get("symbols")
                            .or_else(|| value.get("locations"))
                            .or_else(|| value.get("positions"))
                            .and_then(Value::as_array)
                    })
                    .map(Vec::len)
                    .unwrap_or(1);
                Some(Value::Array(
                    (0..count)
                        .map(|_| common::type_response("t0000000000000001"))
                        .collect(),
                ))
            }
            "getBaseTypes" => Some(json!([common::type_response("t0000000000000001")])),
            "getTypeArguments" if type_param(&params) == Some(MAPPED_UTILITY_TYPE_HANDLE) => Some(
                json!([mapped_utility_argument(MAPPED_UTILITY_SPARSE_ARGUMENT,)]),
            ),
            "getTypeArguments"
            | "getTypesOfType"
            | "getTypeParametersOfType"
            | "getOuterTypeParametersOfType"
            | "getLocalTypeParametersOfType" => {
                Some(json!([common::type_response("t0000000000000001")]))
            }
            "getSignaturesOfType" => Some(json!([common::signature()])),
            "getShorthandAssignmentValueSymbol" | "getParentOfSymbol" | "getSymbolOfType" => {
                Some(common::symbol("value"))
            }
            "getMembersOfSymbol" | "getExportsOfSymbol" | "getPropertiesOfType" => {
                Some(json!([common::symbol("value")]))
            }
            // The checker's own answer for a signature's parameters. The
            // binding used to reconstruct these from source text instead.
            "getParametersOfSignature" => Some(json!([
                common::named_symbol("s0000000000000001", "first"),
                common::named_symbol("s0000000000000002", "second"),
            ])),
            "getThisParameterOfSignature" => {
                Some(common::named_symbol("s0000000000000003", "this"))
            }
            "getExportSymbolOfSymbol" => Some(common::symbol("exported")),
            "getTypePredicateOfSignature" => Some(common::type_predicate()),
            "getIndexInfosOfType" => Some(json!([common::index_info()])),
            "getAnyType" | "getStringType" | "getNumberType" | "getBooleanType" | "getVoidType"
            | "getUndefinedType" | "getNullType" | "getNeverType" | "getUnknownType"
            | "getBigIntType" | "getESSymbolType" => {
                Some(common::type_response("t0000000000000010"))
            }
            "typeToTypeNode" => Some(common::encoded(b"type-node")),
            "typeToString" => Some(json!("type:string")),
            "isContextSensitive" => Some(json!(true)),
            "printNode" => Some(print_node(params)?),
            "release" => Some(Value::Null),
            "ping" => Some(json!("pong")),
            "echo" => Some(params),
            _ => None,
        };
        if let Some(id) = id {
            let response = response.unwrap_or(Value::Null);
            jsonrpc::write_message(&mut writer, &RawMessage::response(id, response))?;
        }
    }
}

fn batch_requests(params: Value) -> Value {
    let responses = params
        .get("requests")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|request| {
            let method = request
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let mut response = json!({ "method": method });
            if let Some(error) = missing_project_error(&params) {
                response["error"] = Value::String(error.message.to_string());
                return response;
            }
            response["result"] = batch_request_result(method, &params).unwrap_or(Value::Null);
            response
        })
        .collect::<Vec<_>>();
    json!({ "responses": responses })
}

fn batch_request_result(method: &str, params: &Value) -> Option<Value> {
    match method {
        "getSymbolsAtPositions" | "getSymbolsAtLocations" => {
            Some(repeated_symbol_responses(batch_request_item_count(params)))
        }
        "getTypeOfSymbol"
        | "getDeclaredTypeOfSymbol"
        | "getTypeAtPosition"
        | "getTypeAtLocation"
        | "getConstraintOfType" => Some(common::type_response("t0000000000000001")),
        "getTypeArguments" => Some(repeated_type_responses(1)),
        "getTypesOfSymbols" | "getTypeAtLocations" | "getTypesAtPositions" => {
            Some(repeated_type_responses(batch_request_item_count(params)))
        }
        "getSymbolOfType"
        | "getAliasedSymbol"
        | "getImmediateAliasedSymbol"
        | "getPropertyOfType" => Some(common::symbol("value")),
        "getExportsOfModule" => Some(json!([common::symbol("value")])),
        "isTypeAssignableTo" => Some(json!(true)),
        "typeToString" => Some(json!("type:string")),
        _ => None,
    }
}

fn batch_request_item_count(params: &Value) -> usize {
    params
        .as_object()
        .and_then(|value| {
            value
                .get("symbols")
                .or_else(|| value.get("locations"))
                .or_else(|| value.get("positions"))
                .and_then(Value::as_array)
        })
        .map(Vec::len)
        .unwrap_or(1)
}

fn repeated_type_responses(count: usize) -> Value {
    Value::Array(
        (0..count)
            .map(|_| common::type_response("t0000000000000001"))
            .collect(),
    )
}

fn repeated_symbol_responses(count: usize) -> Value {
    Value::Array((0..count).map(|_| common::symbol("value")).collect())
}

/// Mirrors upstream's per-project handle resolution so project-less lookups
/// fail here exactly as they do against a real TypeScript 7 stable runtime.
fn missing_project_error(params: &Value) -> Option<RpcResponseError> {
    common::missing_project_message(params).map(|message| RpcResponseError {
        code: -32000,
        message: message.into(),
        data: None,
    })
}

fn release_error(method: &str) -> Option<RpcResponseError> {
    if method != "release" || std::env::var_os("CORSA_MOCK_FAIL_RELEASE_ONCE").is_none() {
        return None;
    }
    if RELEASE_FAILURE_USED.swap(true, Ordering::SeqCst) {
        return None;
    }
    Some(RpcResponseError {
        code: -32000,
        message: "mock release failure".into(),
        data: None,
    })
}

fn stale_handle_error(method: &str, params: &Value) -> Option<RpcResponseError> {
    if method != "getTypeArguments" {
        return None;
    }
    let handle = params.get("type").and_then(Value::as_str)?;
    if handle != STALE_TYPE_HANDLE {
        return None;
    }
    Some(RpcResponseError {
        code: -32603,
        message: format!(
            "api: client error: type handle \"{handle}\" not found in snapshot registry"
        )
        .into(),
        data: None,
    })
}

fn type_param(params: &Value) -> Option<&str> {
    params.get("type").and_then(Value::as_str)
}

fn symbol_param(params: &Value) -> Option<&str> {
    params.get("symbol").and_then(Value::as_str)
}

fn mapped_utility_argument(id: &str) -> Value {
    json!({
        "id": id,
        "flags": 524288,
        "objectFlags": 3,
        "symbol": MAPPED_UTILITY_ARGUMENT_SYMBOL,
        "texts": ["Dog"],
    })
}

fn record_params(method: &str, params: &Value) {
    let Ok(dir) = std::env::var("CORSA_MOCK_PARAMS_DIR") else {
        return;
    };
    if create_dir_all(&dir).is_err() {
        return;
    }
    let path = Path::new(&dir).join(format!("{method}.jsonl"));
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", params);
    }
}

fn record_method(method: &str) {
    maybe_delay(method);
    let Ok(dir) = std::env::var("CORSA_MOCK_COUNT_DIR") else {
        return;
    };
    if create_dir_all(&dir).is_err() {
        return;
    }
    let path = Path::new(&dir).join(format!("{method}.count"));
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "1");
    }
}

fn maybe_delay(method: &str) {
    let Ok(delay_ms) = std::env::var("CORSA_MOCK_DELAY_MS") else {
        return;
    };
    if method != "initialize" && method != "describeCapabilities" {
        return;
    }
    if let Ok(delay_ms) = delay_ms.parse::<u64>() {
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
}

fn parse_config<R: std::io::BufRead, W: std::io::Write>(
    reader: &mut R,
    writer: &mut W,
    callbacks: &[String],
    params: Value,
) -> Result<Value> {
    let file = params
        .get("file")
        .and_then(Value::as_str)
        .unwrap_or("/workspace/tsconfig.json");
    let mut options = json!({ "strict": true });
    if file.starts_with("/virtual/") && callbacks.iter().any(|name| name == "readFile") {
        let response = jsonrpc::send_request(
            reader,
            writer,
            RequestId::string("cb-readFile"),
            "readFile",
            Value::String(file.to_owned()),
        )?;
        options["virtual"] = json!(response.get("content").is_some());
    }
    Ok(json!({
        "options": options,
        "fileNames": ["/workspace/src/index.ts"],
    }))
}

fn print_node(params: Value) -> Result<Value> {
    let data = params
        .get("data")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let decoded = base64::engine::general_purpose::STANDARD.decode(data)?;
    Ok(json!(format!(
        "print:{}",
        String::from_utf8_lossy(&decoded)
    )))
}
