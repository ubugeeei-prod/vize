//! TypeScript content-mapper protocol server.
//!
//! Speaks the TypeScript content-mapper protocol:
//! `initialize` negotiates capabilities and the position encoding, `openProject` and
//! `closeProject` bracket each TypeScript project's mapper options and
//! compiler options, and `transform` projects one Vue SFC into virtual
//! TypeScript for an opened project handle.

use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use clap::Args;
use serde::Deserialize;
use serde_json::{Value, json};
use vize_canon::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};
use vize_carton::{String as CompactString, cstr};

#[path = "content_mapper/project.rs"]
mod project;
use project::{CloseProjectParams, OpenProjectParams, ProjectRegistry, resolve_open_project};

const MAX_MESSAGE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Args, Default)]
pub struct ContentMapperArgs {}

pub fn run(_: ContentMapperArgs) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    if let Err(error) = serve(&mut reader, &mut writer) {
        eprintln!("vize content-mapper: {error}");
        std::process::exit(1);
    }
}

#[derive(Deserialize)]
struct Request {
    #[allow(dead_code)]
    jsonrpc: Option<CompactString>,
    id: Option<Value>,
    method: CompactString,
    #[serde(default)]
    params: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    #[allow(dead_code)]
    locale: Option<CompactString>,
    position_encodings: Vec<CompactString>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformParams {
    file_name: CompactString,
    content: CompactString,
    project_handle: CompactString,
}

fn serve<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let mut initialized = false;
    let mut projects = ProjectRegistry::default();
    while let Some(body) = read_frame(reader)? {
        let request = match serde_json::from_slice::<Request>(&body) {
            Ok(request) => request,
            Err(error) => {
                let message = cstr!("Parse error: {error}");
                write_error(writer, Value::Null, -32700, &message)?;
                continue;
            }
        };
        let id = request.id.unwrap_or(Value::Null);

        match request.method.as_str() {
            "initialize" => {
                let params = match serde_json::from_value::<InitializeParams>(request.params) {
                    Ok(params) => params,
                    Err(error) => {
                        let message = cstr!("Invalid initialize params: {error}");
                        write_error(writer, id, -32602, &message)?;
                        continue;
                    }
                };
                if !params
                    .position_encodings
                    .iter()
                    .any(|encoding| encoding == "utf-8")
                {
                    write_error(
                        writer,
                        id,
                        -32602,
                        "Vize requires the utf-8 position encoding",
                    )?;
                    continue;
                }

                initialized = true;
                let result = json!({
                    "positionEncoding": "utf-8",
                    "diagnosticSource": "vize",
                });
                write_result(writer, id, result)?;
            }
            "openProject" | "closeProject" | "transform" if !initialized => {
                write_error(writer, id, -32002, "Content mapper is not initialized")?;
            }
            "openProject" => {
                let params = match serde_json::from_value::<OpenProjectParams>(request.params) {
                    Ok(params) => params,
                    Err(error) => {
                        let message = cstr!("Invalid openProject params: {error}");
                        write_error(writer, id, -32602, &message)?;
                        continue;
                    }
                };
                let (settings, result) = resolve_open_project(&params);
                projects.open(params.project_handle, settings);
                write_result(writer, id, result)?;
            }
            "closeProject" => {
                let params = match serde_json::from_value::<CloseProjectParams>(request.params) {
                    Ok(params) => params,
                    Err(error) => {
                        let message = cstr!("Invalid closeProject params: {error}");
                        write_error(writer, id, -32602, &message)?;
                        continue;
                    }
                };
                projects.close(params.project_handle.as_str());
                write_result(writer, id, Value::Null)?;
            }
            "transform" => {
                let params = match serde_json::from_value::<TransformParams>(request.params) {
                    Ok(params) => params,
                    Err(error) => {
                        let message = cstr!("Invalid transform params: {error}");
                        write_error(writer, id, -32602, &message)?;
                        continue;
                    }
                };
                let Some(settings) = projects.settings(params.project_handle.as_str()) else {
                    let message =
                        cstr!("Unknown content mapper project: {}", params.project_handle);
                    write_error(writer, id, -32602, &message)?;
                    continue;
                };
                let transform_options = ContentMapperTransformOptions::default()
                    .with_options_api(settings.options_api)
                    .with_preserve_unused_diagnostics(settings.no_unused_locals);
                match generate_vue_content_mapper_transform_with_options(
                    Path::new(params.file_name.as_str()),
                    params.content.as_str(),
                    transform_options,
                ) {
                    Ok(result) => write_result(writer, id, result)?,
                    Err(error) => {
                        let message = cstr!("Transform failed: {error}");
                        write_error(writer, id, -32603, &message)?;
                    }
                }
            }
            _ => {
                let message = cstr!("Method not found: {}", request.method);
                write_error(writer, id, -32601, &message)?;
            }
        }
    }
    Ok(())
}

fn read_frame<R: BufRead>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut content_length = None;
    let mut saw_header = false;
    loop {
        let mut line = Vec::new();
        let bytes = reader.read_until(b'\n', &mut line)?;
        if bytes == 0 {
            if saw_header {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF inside content-mapper headers",
                ));
            }
            return Ok(None);
        }
        saw_header = true;
        if line == b"\r\n" || line == b"\n" {
            break;
        }
        if let Some(separator) = line.iter().position(|byte| *byte == b':')
            && line[..separator].eq_ignore_ascii_case(b"content-length")
        {
            let value = std::str::from_utf8(&line[separator + 1..])
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length"))?
                .trim();
            content_length = Some(value.parse::<usize>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length")
            })?);
        }
    }

    let content_length = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header")
    })?;
    if content_length > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "content-mapper message exceeds 64 MiB",
        ));
    }
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

fn write_result<W: Write>(
    writer: &mut W,
    id: Value,
    result: impl serde::Serialize,
) -> io::Result<()> {
    write_frame(
        writer,
        &json!({"jsonrpc": "2.0", "id": id, "result": result}),
    )
}

fn write_error<W: Write>(writer: &mut W, id: Value, code: i32, message: &str) -> io::Result<()> {
    write_frame(
        writer,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": code, "message": message},
        }),
    )
}

fn write_frame<W: Write>(writer: &mut W, value: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(value).map_err(io::Error::other)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

#[cfg(test)]
#[path = "content_mapper/test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "content_mapper/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "content_mapper/project_tests.rs"]
mod project_tests;
