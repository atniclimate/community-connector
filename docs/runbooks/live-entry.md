# Live entry runbook (ATNI convention pilot, D-053 in-app path)

In-app facilitator entry is the primary intake path (D-053, D-091c). Mechanical
steps only - this does not authorize real ingestion; that still needs the
D-023 sign-off and the recorded collective checkpoint (HANDOFF.md).

## 1. Create the real queue folder

Propose `I:\cn-data\atni-convention\` (or any folder under a local drive
dedicated to real data). It **must** live:

- outside this repository (never inside `community-connector/`), and
- outside any cloud-synced tree - no OneDrive, Google Drive, or Dropbox path
  component anywhere in its ancestry.

`cn intake apply`'s queue guard canonicalizes the path and refuses it if
either rule is violated, or if it sits inside a git worktree (ADR-005 D4).
Create the empty folder once, by hand, before the first session.

## 2. Point the wizard at it

Open the app as a facilitator or governance viewer of the `atni-convention`
group. In the intake panel's directory step, click "grant" and pick the
folder from step 1 (Chromium's File System Access picker). The wizard writes
staged entries and decisions there; it never mutates the graph itself.

## 3. The entry loop

1. A facilitator enters a new record in the wizard (staged, not published).
2. A facilitator (can be a different person) reviews and approves or rejects
   with a reason - two distinct recorded acts even if one person does both
   (D-030).
3. Run, from a terminal:
   `cn intake apply --queue <folder> --ops <group's .ops.jsonl> --group <group-uuid> --facilitator <facilitator-person-uuid>`
4. Reload the group in the app to see the applied entries.
Repeat step 3 in batches; `cn intake apply` is the only thing that writes to
the authoritative op log.

**Real data is never copied into the repo tree.**
