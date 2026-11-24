---------------------------- MODULE Publication ----------------------------
EXTENDS Naturals

CONSTANT Fault
VARIABLES world, ticket, prepared, acknowledged, phase, reused, answer
vars == <<world, ticket, prepared, acknowledged, phase, reused, answer>>

\* A previously published response binds the native snapshot, logical file,
\* complete source set, generated projection and authored editor revision.
Saved == [native |-> 0, doc |-> 0, sources |-> 0, host |-> 0, revision |-> 0]
Editor(w) == <<w.doc, w.revision>>
Packet(w, editor) == <<w.native, w.doc, w.sources, w.host, editor>>
Empty == <<>>
Init == /\ world = Saved /\ ticket = Editor(Saved) /\ prepared = Saved
        /\ acknowledged = FALSE /\ phase = "idle" /\ reused = FALSE /\ answer = Empty

\* Changes to physical project inputs can occur without an editor notification.
\* A document change during the request is caught by the final editor check.
Change == /\ phase # "done"
          /\ \E field \in {"native", "doc", "sources", "host", "revision"}:
                /\ world[field] = 0 /\ world' = [world EXCEPT ![field] = 1]
          /\ UNCHANGED <<ticket, prepared, acknowledged, phase, reused, answer>>

Begin == /\ phase = "idle" /\ ticket' = Editor(world) /\ phase' \in {"probe", "prepareFirst"}
         /\ UNCHANGED <<world, prepared, acknowledged, reused, answer>>
Probe == /\ phase = "probe" /\ acknowledged' \in BOOLEAN /\ phase' = "sync"
         /\ UNCHANGED <<world, ticket, prepared, reused, answer>>
Sync == /\ phase = "sync" /\ prepared' = world /\ phase' = "read"
        /\ UNCHANGED <<world, ticket, acknowledged, reused, answer>>

\* Empty native batches can synchronize before the owner is queried. Both
\* schedules check the acknowledged native world before returning a cached row.
PrepareFirst == /\ phase = "prepareFirst" /\ prepared' = world /\ phase' = "lateProbe"
                /\ UNCHANGED <<world, ticket, acknowledged, reused, answer>>
LateProbe == /\ phase = "lateProbe" /\ acknowledged' \in BOOLEAN /\ phase' = "read"
             /\ UNCHANGED <<world, ticket, prepared, reused, answer>>

CanReuse == /\ (acknowledged \/ Fault = "ack")
            /\ (Saved.native = prepared.native \/ Fault = "native")
            /\ (Saved.doc = prepared.doc \/ Fault = "doc")
            /\ (Saved.sources = prepared.sources \/ Fault = "sources")
            /\ (Saved.host = prepared.host \/ Fault = "host")
            /\ (Saved.revision = ticket[2] \/ Fault = "revision")
Read == /\ phase = "read" /\ reused' = CanReuse /\ phase' = "publish"
        /\ answer' = IF CanReuse THEN Packet(Saved, Editor(Saved))
                      ELSE Packet(prepared, ticket)
        /\ UNCHANGED <<world, ticket, prepared, acknowledged>>
Publish == /\ phase = "publish" /\ phase' = "done"
           /\ IF ticket = Editor(world) \/ Fault = "publish"
              THEN UNCHANGED answer ELSE answer' = Empty
           /\ UNCHANGED <<world, ticket, prepared, acknowledged, reused>>
Done == /\ phase = "done" /\ UNCHANGED vars

Next == Change \/ Begin \/ Probe \/ Sync \/ PrepareFirst \/ LateProbe \/ Read \/ Publish \/ Done
Spec == Init /\ [][Next]_vars
ExactPublication == phase # "done" \/ answer = Empty \/
                    /\ ticket = Editor(world)
                    /\ answer = Packet(prepared, ticket)
                    /\ (~reused \/ acknowledged)
=============================================================================
