# Security Policy

## Supported Versions

Vize is preparing for a v1 alpha release. Until a stable v1 line exists, security support covers:

- the latest commit on `main`
- the latest published prerelease for each package

Older prereleases are not supported unless a maintainer explicitly marks them as supported in a release note.

## Reporting a Vulnerability

Please do not open a public tracker entry for a suspected vulnerability.

Use GitHub's private vulnerability reporting flow for this repository. If that is unavailable, contact the maintainer through their GitHub profile and ask for a private channel.

Include as much of the following as you can:

- affected package, CLI command, or integration
- affected version or commit SHA
- operating system, architecture, Node.js version, and package manager
- minimal reproduction steps
- impact, such as code execution, filesystem access, data exposure, denial of service, or supply-chain risk
- whether the report is already public

## Response Expectations

Maintainers aim to acknowledge credible reports within 7 days. Confirmed vulnerabilities are handled privately until a fix or mitigation is available, then disclosed in release notes with credit when the reporter wants attribution.

## Scope

Security-sensitive areas include native bindings, WASM artifacts, package publication, CLI filesystem behavior, compiler input handling, editor integrations, and dev-server middleware. Reports outside those areas are still welcome when they affect users of Vize packages.
