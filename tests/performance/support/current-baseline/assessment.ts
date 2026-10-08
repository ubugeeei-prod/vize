import type { auditSession } from "./audit.ts";
import { statistics } from "./wire.ts";

export type SessionAssessment = {
  index: number;
  status: string;
  failure: string | null;
  report: ReturnType<typeof auditSession> | null;
};

// A failed/missing session cannot contribute a favorable accepted subset.
export function assessCampaign(sessions: SessionAssessment[]) {
  const success =
    sessions.length === 3 &&
    sessions.every(
      (session, index) =>
        session.index === index + 1 &&
        session.status === "passed" &&
        session.report &&
        session.failure === null,
    );
  const reports = success ? sessions.map((session) => session.report!) : [];
  return {
    status: success ? "passed" : "failed",
    failures: sessions.filter((session) => session.status !== "passed").length,
    missingSessions: Math.max(0, 3 - sessions.length),
    timeoutCount: success ? 0 : null,
    timeoutReason: success
      ? null
      : "unqualified/partial sessions retain all test/RPC errors; timeout total is not inferred",
    sessions,
    warm: success
      ? Object.fromEntries(
          Object.keys(reports[0].lanes).map((lane) => [
            lane,
            statistics(reports.flatMap((report) => report.lanes[lane])),
          ]),
        )
      : null,
    startup: success
      ? {
          raw: reports.map((report) => report.startup),
          spawnToReady: statistics(
            reports.map((report) => report.startup.spawnToReadyMs),
            0,
            true,
          ),
          coldOpen: statistics(
            reports.map((report) => report.startup.coldOpenMs),
            0,
            true,
          ),
        }
      : null,
  };
}
