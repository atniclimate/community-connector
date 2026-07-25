
## D-078 (2026-07-25) - Implementation adversarial round 3: FAIL, amended; step-9 contract amended

Round 3 (review at _reviews/community-connector/
2026-07-25_intake-pipeline-impl-round3.md, target HEAD ca4e714) closed
F1 (recovery reauthorization + sticky quarantine now match ADR-005 D4 -
the reviewer found no remaining crash window) and confirmed the durable
rename primitive, and returned FAIL on the residue. All verified;
amendments:

1. **Native fail-closed predicates (F3 blocker):** the worktree guard,
   directory enumeration, and quarantine moves now use fallible metadata
   operations - only NotFound means absent; any other IO error refuses
   loudly (I3). Record/sidecar/decision reads distinguish IO failure
   (run refuses - it cannot classify what it cannot read) from parse/
   checksum corruption (the quarantine row), and the corruption reason
   reaches the I12 report. The app's .git probe refuses unverifiable
   roots the same way.
2. **Durable notices (F3/F11):** persistence warnings ride a new
   `notices` state field that scans never clear (only a directory
   change does); a reducer-level FINAL-STATE test proves the warning
   survives the scan that follows it. The dashboard renders notices.
3. **Geo contract end-to-end (F9):** the form now encodes the core's
   canonical raw geo shape ("lat, lon" -> {lat, lon}; anything else ->
   {name}); check_field validates geo objects deeply (exactly {lat,lon}
   or {name}; region names get the full hazard checks; unknown keys
   rejected). NFC Unicode normalization (unicode-normalization crate)
   is applied to stored text/tags/region names per the blueprint rule.
   The app cap is measured in UTF-8 BYTES matching the core
   (TEXT_MAX_BYTES, multi-byte regression test). form_version and
   consent_text_digest get hazard checks; non-sha256 digest shape is a
   WARNING (synthetic fixtures carry marker digests by design).
4. **F12 honesty note:** the self-test cleans up BEFORE claiming
   removal and fails if cleanup leaves fixtures behind.
5. **Step-9 contract AMENDED in the blueprint itself** (the reviewer's
   requirement): the mount is bound to every interactive load path that
   exists; the production/pilot path is owed with the August pilot
   build, where the same mount and the synthetic decide -> apply ->
   reload rehearsal are mandatory before pilot use. Fails closed today.

Still open and honestly recorded: browser-level IndexedDB tests, the
production interactive path itself, .rs/.ts/archive content bypasses in
the tripwire (defense-in-depth disclaimer stands), and the D-068 deploy
gates (fault-injection breadth, quiescent provenance deployment,
provider runbook, ceremony rehearsal). Round 4 verification follows.
