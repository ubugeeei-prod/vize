import * as fs from "fs";
import * as https from "https";

const TRUSTED_RELEASE_DOWNLOAD_HOSTS = new Set([
  "github.com",
  "objects.githubusercontent.com",
  "release-assets.githubusercontent.com",
]);

const RELEASE_DOWNLOAD_TIMEOUT_MS = 30_000;
const RELEASE_DOWNLOAD_MAX_REDIRECTS = 5;

/**
 * Release archives and SHA-256 sidecars must stay on GitHub HTTPS hosts.
 * Following an open redirect off github.com would let a MITM or poisoned
 * Location header supply both the binary and a matching checksum.
 */
export function isTrustedReleaseDownloadUrl(url: string): boolean {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }

  if (parsed.protocol !== "https:") {
    return false;
  }
  if (parsed.username || parsed.password) {
    return false;
  }
  if (parsed.port !== "") {
    return false;
  }

  const host = parsed.hostname.toLowerCase();
  return TRUSTED_RELEASE_DOWNLOAD_HOSTS.has(host) || host.endsWith(".githubusercontent.com");
}

export async function downloadFile(
  url: string,
  destination: string,
  redirectCount = 0,
): Promise<void> {
  if (redirectCount > RELEASE_DOWNLOAD_MAX_REDIRECTS) {
    throw new Error(`too many redirects while downloading ${url}`);
  }
  if (!isTrustedReleaseDownloadUrl(url)) {
    throw new Error(`refusing to download Vize language server from untrusted URL: ${url}`);
  }

  await new Promise<void>((resolve, reject) => {
    const request = https.get(
      url,
      {
        headers: {
          "User-Agent": "vscode-vize",
        },
      },
      (response) => {
        const statusCode = response.statusCode ?? 0;
        if (statusCode >= 300 && statusCode < 400 && response.headers.location) {
          response.resume();
          const redirectUrl = new URL(response.headers.location, url).toString();
          if (!isTrustedReleaseDownloadUrl(redirectUrl)) {
            reject(new Error(`refusing redirect to untrusted URL: ${redirectUrl}`));
            return;
          }
          downloadFile(redirectUrl, destination, redirectCount + 1).then(resolve, reject);
          return;
        }

        if (statusCode !== 200) {
          response.resume();
          reject(new Error(`download failed with HTTP ${statusCode}: ${url}`));
          return;
        }

        const file = fs.createWriteStream(destination);
        file.on("error", reject);
        file.on("finish", () => {
          file.close((error) => {
            if (error) {
              reject(error);
              return;
            }
            resolve();
          });
        });
        response.pipe(file);
      },
    );

    request.setTimeout(RELEASE_DOWNLOAD_TIMEOUT_MS, () => {
      request.destroy(new Error(`download timed out after ${RELEASE_DOWNLOAD_TIMEOUT_MS}ms`));
    });
    request.on("error", reject);
  }).catch(async (error) => {
    await fs.promises.rm(destination, { force: true });
    throw error;
  });
}
