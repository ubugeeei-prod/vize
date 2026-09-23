const defaultSleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));

function retryDelayMs(response, attempt) {
  const retryAfterHeader = response.headers.get("retry-after");
  const retryAfter = Number(retryAfterHeader);
  if (retryAfterHeader != null && Number.isFinite(retryAfter) && retryAfter >= 0) {
    return retryAfter * 1000;
  }
  return 2 ** (attempt - 1) * 1000;
}

function isRetryableResponse(response) {
  return (
    response.status === 429 ||
    response.status >= 500 ||
    (response.status === 403 && response.headers.has("retry-after"))
  );
}

export async function githubApiRequest({
  apiUrl,
  repository,
  token,
  resource,
  method = "GET",
  body,
  fetchImpl = globalThis.fetch,
  requestTimeoutMs = 30_000,
  sleep = defaultSleep,
}) {
  if (!Number.isFinite(requestTimeoutMs) || requestTimeoutMs <= 0) {
    throw new Error(`GitHub API request timeout must be positive, got ${requestTimeoutMs}`);
  }
  const url = new URL(`${apiUrl}/repos/${repository}/${resource}`);
  const maxAttempts = method === "GET" ? 3 : 1;
  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), requestTimeoutMs);
    let response;
    let detail;
    try {
      response = await fetchImpl(url, {
        method,
        headers: {
          accept: "application/vnd.github+json",
          authorization: `Bearer ${token}`,
          "content-type": "application/json",
          "x-github-api-version": "2022-11-28",
        },
        body: body == null ? undefined : JSON.stringify(body),
        signal: controller.signal,
      });
      detail = (await response.text()).trim();
    } catch (error) {
      if (attempt < maxAttempts) {
        await sleep(2 ** (attempt - 1) * 1000);
        continue;
      }
      if (controller.signal.aborted) {
        throw new Error(
          `GitHub API ${method} ${url.pathname} timed out after ${requestTimeoutMs}ms`,
        );
      }
      throw error;
    } finally {
      clearTimeout(timer);
    }

    if (response.ok) return { body: detail, headers: response.headers, status: response.status };
    if (attempt < maxAttempts && isRetryableResponse(response)) {
      await sleep(retryDelayMs(response, attempt));
      continue;
    }
    throw new Error(
      `GitHub API ${method} ${url.pathname} failed: ${response.status} ${response.statusText}${detail === "" ? "" : `\n${detail}`}`,
    );
  }
  throw new Error(`GitHub API ${method} ${url.pathname} exhausted its retry budget`);
}

export async function githubApiPages({
  apiUrl,
  repository,
  token,
  resource,
  collection,
  query = {},
  ...requestOptions
}) {
  const values = [];
  for (let page = 1; ; page += 1) {
    const params = new URLSearchParams({ per_page: "100", page: String(page) });
    for (const [name, value] of Object.entries(query)) params.set(name, value);
    const response = await githubApiRequest({
      apiUrl,
      repository,
      token,
      resource: `${resource}?${params}`,
      ...requestOptions,
    });
    let payload;
    try {
      payload = JSON.parse(response.body);
    } catch {
      throw new Error(`GitHub API ${resource} returned invalid JSON on page ${page}`);
    }
    const pageValues = collection == null ? payload : payload[collection];
    if (!Array.isArray(pageValues)) {
      throw new Error(`GitHub API ${resource} did not return ${collection ?? "an array"}`);
    }
    values.push(...pageValues);
    if (pageValues.length < 100) return values;
  }
}
