mod api_async;
mod common;
mod jsonrpc;
mod lsp;
mod msgpack;

type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T> = std::result::Result<T, DynError>;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let cwd = args
        .windows(2)
        .find_map(|window| (window[0] == "--cwd").then(|| window[1].clone()))
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|cwd| cwd.display().to_string())
                .unwrap_or_else(|_| ".".into())
        });
    let callbacks = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--callbacks="))
        .map(|list| list.split(',').map(str::to_owned).collect::<Vec<_>>())
        .unwrap_or_default();
    if args.iter().any(|arg| arg == "--lsp") {
        return lsp::run(&args);
    }
    if args.iter().any(|arg| arg == "--api") && args.iter().any(|arg| arg == "--async") {
        return api_async::run(cwd, callbacks);
    }
    if args.iter().any(|arg| arg == "--api") {
        return msgpack::run(cwd, callbacks);
    }
    Err("unsupported invocation".into())
}
