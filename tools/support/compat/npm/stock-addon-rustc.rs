// Compiler adapter for the independent cache-identity control build.
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let mut command = Command::new(args.next().ok_or("expected rustc")?);
    let args: Vec<_> = args.collect();
    if args
        .windows(2)
        .any(|pair| pair[0] == "--crate-name" && pair[1] == "vize_vitrine")
    {
        let actual = env::var("VIZE_PLUGIN_HOST_BUILD_ID")?;
        let base = env::var("VIZE_PROOF_BASE_ID")?;
        if actual != env::var("VIZE_PROOF_PRODUCED_ID")? {
            return Err("unexpected native cache identity from Cargo".into());
        }
        let mut receipt = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(env::var_os("VIZE_PROOF_RECEIPT").ok_or("expected receipt path")?)?;
        writeln!(receipt, "{actual}\n{base}")?;
        command.env("VIZE_PLUGIN_HOST_BUILD_ID", base);
    }
    command.args(args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec().into())
    }
    #[cfg(not(unix))]
    std::process::exit(command.status()?.code().unwrap_or(1))
}
