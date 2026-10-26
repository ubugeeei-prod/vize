/** Next 16.4.3 start-server handles SIGTERM by exiting with 128 + 15. */
export function normalHostShutdown(result, shutdown) {
  return (
    result.status === 0 ||
    result.signal === "SIGTERM" ||
    (result.status === 143 &&
      result.signal === null &&
      shutdown.statusBefore === null &&
      shutdown.signalBefore === null &&
      shutdown.requested === true)
  );
}
