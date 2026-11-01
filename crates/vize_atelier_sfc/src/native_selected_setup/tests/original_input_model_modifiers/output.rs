//! Complete source-built attempts are retained before the current verdict.

use crate::{NativeSelectedSfcDomOutput, NativeSsrSfcOutput, NativeVaporSetupSfcOutput};
use serde_json::{Value, json};
use std::fmt::Debug;
use vize_l1_to_l2::native_file::NativeSelectedSetupSfcObservation;

pub(super) trait Output {
    fn code(&self) -> &str;
    fn source_map(&self) -> Option<&str>;
    fn links(&self) -> usize;
}
macro_rules! output {
    ($kind:ty) => {
        impl Output for $kind {
            fn code(&self) -> &str {
                self.code()
            }
            fn source_map(&self) -> Option<&str> {
                self.source_map()
            }
            fn links(&self) -> usize {
                self.document().links().len()
            }
        }
    };
}
output!(NativeSelectedSfcDomOutput);
output!(NativeSsrSfcOutput);
output!(NativeVaporSetupSfcOutput);

pub(super) fn observe<T: Output, E: Debug>(result: &Result<&T, E>) -> Value {
    match result {
        Ok(output) => json!({"state":"emitted","code":output.code(),
            "nativeMapRaw":output.source_map(),
            "nativeMap":output.source_map().and_then(|raw|serde_json::from_str::<Value>(raw).ok()),
            "linkCount":output.links()}),
        Err(error) => json!({"state":"refused","error":format!("{error:?}")}),
    }
}

pub(super) struct Captured {
    pub(super) attempt: Value,
    pub(super) result: Result<(Value, Value), String>,
}

pub(super) fn capture<T: Output, E: Debug>(
    fixture: &Value,
    target: &str,
    mode: &str,
    mapped: Result<&T, E>,
    plain: Result<&T, E>,
    default: Option<Result<&T, E>>,
    observation: &NativeSelectedSetupSfcObservation<'_>,
    modifier: &str,
) -> Captured {
    let attempt = json!({"id":fixture["id"],"target":target,"mode":mode,"source":fixture["source"],
        "mapped":observe(&mapped),"plain":observe(&plain),
        "defaultOptions":default.as_ref().map(observe)});
    let result = (|| {
        let id = fixture["id"].as_str().ok_or("original input id")?;
        let source = fixture["source"].as_str().ok_or("original input source")?;
        let output = mapped.map_err(|error| format!("{id}/{target}/{mode}: {error:?}"))?;
        let plain = plain.map_err(|error| format!("{id}/{target}/{mode} plain: {error:?}"))?;
        if output.code() != plain.code() || plain.source_map().is_some() || plain.links() != 0 {
            return Err(format!("{id}/{target}/{mode}: whole NoLinks equality"));
        }
        if let Some(default) = default {
            let default =
                default.map_err(|error| format!("{id}/{target}/{mode} default: {error:?}"))?;
            if default.code() != output.code()
                || default.source_map().is_some()
                || default.links() != 0
            {
                return Err(format!(
                    "{id}/{target}/{mode}: whole default-options equality"
                ));
            }
        }
        let raw = output.source_map().ok_or("complete raw map")?;
        let map: Value = serde_json::from_str(raw).map_err(|error| format!("{error}"))?;
        if map["sourcesContent"] != json!([source]) {
            return Err(format!("{id}/{target}/{mode}: original whole source map"));
        }
        let owner = super::custody::capture(observation, fixture, target, mode, modifier)?;
        Ok((
            json!({"id":id,"target":target,"mode":mode,"source":source,
            "code":output.code(),"nativeMapRaw":raw,"nativeMap":map}),
            owner,
        ))
    })();
    Captured { attempt, result }
}
