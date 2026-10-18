import fs from "node:fs";
import path from "node:path";

// VS Code's existing ordinary OutputChannel writes these native files. Copy
// Vize's complete bytes after the host exits, before removing its profile.
// The original fake-host diagnostic additionally keeps the existing logs subtree.
export function retainHostOutput(profilePath, destination, retainAllLogs = false) {
  const logsRoot = path.join(profilePath, "user-data", "logs");
  for (const directory of [profilePath, path.join(profilePath, "user-data"), logsRoot]) {
    if (fs.lstatSync(directory).isSymbolicLink()) {
      throw new Error(`Refusing symbolic link in host output logs: ${directory}`);
    }
  }
  let copied = 0;
  function visit(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const source = path.join(directory, entry.name);
      if (entry.isSymbolicLink()) {
        throw new Error(`Refusing symbolic link in host output logs: ${source}`);
      }
      if (entry.isDirectory()) {
        visit(source);
      } else if (entry.isFile()) {
        const isVizeOutput =
          /^\d+-Vize\.log$/.test(entry.name) &&
          path.basename(directory).startsWith("output_logging_");
        if (!retainAllLogs && !isVizeOutput) continue;
        const target = path.join(destination, path.relative(logsRoot, source));
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.copyFileSync(source, target);
        if (isVizeOutput) copied += 1;
      }
    }
  }
  visit(logsRoot);
  if (copied === 0) throw new Error("No native Vize OutputChannel log was retained");
}

export function retainHostOutputBeforeCleanup(profilePath, destination, retainAllLogs = false) {
  try {
    retainHostOutput(profilePath, destination, retainAllLogs);
  } catch (error) {
    // A diagnostic retention refusal must not replace the original host error.
    console.error(`Vize OutputChannel retention refused: ${String(error)}`);
    try {
      fs.mkdirSync(destination, { recursive: true });
      fs.writeFileSync(path.join(destination, "retention-error.txt"), String(error) + "\n");
    } catch (writeError) {
      console.error(`Vize OutputChannel refusal could not be saved: ${String(writeError)}`);
    }
  } finally {
    fs.rmSync(profilePath, { force: true, recursive: true });
  }
}
