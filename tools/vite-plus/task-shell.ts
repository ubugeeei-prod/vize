/**
 * Quotes a command fragment for POSIX shell interpolation.
 *
 * Vite+ tasks are command strings, so a few helpers intentionally compose shell
 * snippets. This function keeps that composition centralized and prevents
 * environment-derived paths from breaking commands that are later wrapped in
 * `sh -c`.
 */
export const shellQuote = (command: string) => `'${command.replaceAll("'", `'"'"'`)}'`;

const darwinLibiconvLibraryPath = process.env.VIZE_DARWIN_LIBICONV_LIB;
const rustTaskEnvironment =
  darwinLibiconvLibraryPath == null
    ? []
    : [
        `export LIBRARY_PATH=${shellQuote(darwinLibiconvLibraryPath)}\${LIBRARY_PATH:+:$LIBRARY_PATH}`,
        `export RUSTFLAGS=${shellQuote(`-L native=${darwinLibiconvLibraryPath}`)}\${RUSTFLAGS:+ $RUSTFLAGS}`,
      ];

/**
 * Applies the optional macOS libiconv environment to Rust-oriented task
 * commands.
 *
 * The environment is injected only when explicitly requested so regular Linux
 * CI and developer machines keep the shortest possible command path. When the
 * variable is present, both Cargo and any nested Rust build script see the same
 * library search path.
 */
export const withRustTaskEnvironment = (command: string) =>
  rustTaskEnvironment.length === 0
    ? command
    : `sh -c ${shellQuote(`${rustTaskEnvironment.join("; ")}; ${command}`)}`;
