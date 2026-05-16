import { ref, shallowRef } from "vue";
import { sendMessage, type MuseaMessage } from "./usePostMessage";

export interface A11yNode {
  html: string;
  target: string[];
  failureSummary?: string;
}

export interface A11yViolation {
  id: string;
  impact: "critical" | "serious" | "moderate" | "minor";
  description: string;
  helpUrl: string;
  nodes: A11yNode[];
}

export interface A11yResult {
  violations: A11yViolation[];
  passes: number;
  incomplete: number;
  error?: string;
}

interface IncomingA11yPayload extends A11yResult {
  requestId?: string;
}

interface PendingRequest {
  key: string;
  requestId: string;
  promise: Promise<A11yResult>;
  reject: (error: Error) => void;
  resolve: (result: A11yResult) => void;
  timeoutId: ReturnType<typeof setTimeout>;
}

const results = shallowRef<Map<string, A11yResult>>(new Map());
const runningKeys = ref<Set<string>>(new Set());
const latestRequestByKey = new Map<string, string>();
const pendingRequests = new Map<string, PendingRequest>();

let requestSequence = 0;
let initialized = false;

function setResult(key: string, result: A11yResult) {
  const newMap = new Map(results.value);
  newMap.set(key, result);
  results.value = newMap;
}

function clearResult(key: string) {
  if (!results.value.has(key)) {
    return;
  }

  const newMap = new Map(results.value);
  newMap.delete(key);
  results.value = newMap;
}

function setKeyRunning(key: string, isRunning: boolean) {
  const newSet = new Set(runningKeys.value);

  if (isRunning) {
    newSet.add(key);
  } else {
    newSet.delete(key);
  }

  runningKeys.value = newSet;
}

function handleRequestTimeout(pending: PendingRequest) {
  pendingRequests.delete(pending.requestId);

  if (latestRequestByKey.get(pending.key) === pending.requestId) {
    latestRequestByKey.delete(pending.key);
    setKeyRunning(pending.key, false);
    setResult(pending.key, {
      violations: [],
      passes: 0,
      incomplete: 0,
      error: "A11y test timed out after 30s",
    });
  }

  pending.reject(new Error("A11y test timed out after 30s"));
}

function getPendingRequestForKey(key: string): PendingRequest | undefined {
  for (const pending of pendingRequests.values()) {
    if (pending.key === key) {
      return pending;
    }
  }

  return undefined;
}

function createPendingRequest(key: string): PendingRequest {
  const requestId = `a11y-${++requestSequence}`;

  let resolveRequest!: (result: A11yResult) => void;
  let rejectRequest!: (error: Error) => void;

  const promise = new Promise<A11yResult>((resolve, reject) => {
    resolveRequest = resolve;
    rejectRequest = reject;
  });

  const pending: PendingRequest = {
    key,
    requestId,
    promise,
    resolve: resolveRequest,
    reject: rejectRequest,
    timeoutId: setTimeout(() => {
      handleRequestTimeout(pending);
    }, 30000),
  };

  pendingRequests.set(requestId, pending);
  latestRequestByKey.set(key, requestId);
  clearResult(key);
  setKeyRunning(key, true);

  return pending;
}

function handleA11yMessage(event: MessageEvent) {
  if (event.origin !== window.location.origin) {
    return;
  }

  const data = event.data as MuseaMessage | undefined;
  if (data?.type !== "musea:a11y-result") {
    return;
  }

  const payload = data.payload as IncomingA11yPayload;
  const requestId = payload.requestId;
  if (!requestId) {
    return;
  }

  const pending = pendingRequests.get(requestId);
  if (!pending) {
    return;
  }

  pendingRequests.delete(requestId);
  clearTimeout(pending.timeoutId);

  const { requestId: _requestId, ...result } = payload;

  if (latestRequestByKey.get(pending.key) === requestId) {
    latestRequestByKey.delete(pending.key);
    setKeyRunning(pending.key, false);
    setResult(pending.key, result);
  }

  pending.resolve(result);
}

function ensureInitialized() {
  if (initialized || typeof window === "undefined") {
    return;
  }

  window.addEventListener("message", handleA11yMessage);
  initialized = true;
}

function startRun(iframe: HTMLIFrameElement, key: string): PendingRequest {
  ensureInitialized();

  const existingPending = getPendingRequestForKey(key);
  if (existingPending) {
    return existingPending;
  }

  const pending = createPendingRequest(key);
  sendMessage(iframe, "musea:run-a11y", { requestId: pending.requestId });

  return pending;
}

export function useA11y() {
  function init() {
    ensureInitialized();
  }

  function runA11y(iframe: HTMLIFrameElement, key: string) {
    startRun(iframe, key);
  }

  function isKeyRunning(key: string): boolean {
    return runningKeys.value.has(key);
  }

  function getResult(key: string): A11yResult | undefined {
    return results.value.get(key);
  }

  function runA11yAsync(iframe: HTMLIFrameElement, key: string): Promise<A11yResult> {
    return startRun(iframe, key).promise;
  }

  function clearResults() {
    results.value = new Map();
  }

  return {
    results,
    runningKeys,
    init,
    runA11y,
    runA11yAsync,
    isKeyRunning,
    getResult,
    clearResults,
  };
}
