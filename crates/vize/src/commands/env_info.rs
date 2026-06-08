use std::process::Command;
use vize_carton::{String, cstr};

pub fn run() {
    println!("{}", collect().join("\n"));
}

fn collect() -> Vec<String> {
    vec![
        cstr!("Vize: {}", env!("CARGO_PKG_VERSION")),
        cstr!("OS: {}", os()),
        cstr!("Architecture: {}", std::env::consts::ARCH),
        cstr!("Node.js: {}", command_version("node", &["--version"])),
        cstr!("Package manager: {}", package_manager()),
        cstr!("Rust: {}", command_version("rustc", &["--version"])),
    ]
}

fn os() -> String {
    let os = std::env::consts::OS;
    let version = match os {
        "macos" => command_version("sw_vers", &["-productVersion"]),
        "linux" => command_version("uname", &["-r"]),
        "windows" => command_version("cmd", &["/C", "ver"]),
        _ => "unknown".into(),
    };

    if version == "not found" || version == "unknown" {
        os.into()
    } else {
        cstr!("{os} {version}")
    }
}

fn package_manager() -> String {
    if let Ok(user_agent) = std::env::var("npm_config_user_agent")
        && !user_agent.trim().is_empty()
    {
        return user_agent.into();
    }

    let candidates = [
        ("aube", ["--version"].as_slice()),
        ("pnpm", ["--version"].as_slice()),
        ("yarn", ["--version"].as_slice()),
        ("npm", ["--version"].as_slice()),
    ];

    for (program, args) in candidates {
        let version = command_version(program, args);
        if version != "not found" && version != "unknown" {
            return cstr!("{program} {version}");
        }
    }

    "not found".into()
}

fn command_version(program: &str, args: &[&str]) -> String {
    let Ok(output) = Command::new(program).args(args).output() else {
        return "not found".into();
    };

    if !output.status.success() {
        return "unknown".into();
    }

    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("");
    let version = stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .find(|line| !line.is_empty());

    version.unwrap_or("unknown").into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn collect_prints_bug_report_fields() {
        let lines = super::collect();

        for prefix in [
            "Vize: ",
            "OS: ",
            "Architecture: ",
            "Node.js: ",
            "Package manager: ",
            "Rust: ",
        ] {
            assert!(
                lines.iter().any(|line| line.starts_with(prefix)),
                "missing {prefix} in {lines:?}"
            );
        }
    }
}
