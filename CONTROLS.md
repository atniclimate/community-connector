# Controls

Launch with `pwsh scripts/reveal.ps1` (or `-Built` for a built preview).

## Presenter mode

Click the **Present** toolbar button once beats have loaded (the app does not
read a URL query parameter for this - see `scripts/reveal.ps1`'s header note).

| Key | Action |
| --- | --- |
| `Space` / `ArrowRight` | Advance to the next beat |
| `ArrowLeft` | Back to the previous beat |
| `Home` | Jump to the first beat |
| `F` | Zoom-to-fit the whole network (camera only, no beat change) |
| `Escape` | Exit presenter mode |

Every key above only does something while presenter mode is active.

## Editing beats

Beats live in `app/public/beats.atni.json` - a plain array, no schema
validation. Each entry:

```json
{ "id": "members", "label": "Members", "filter": { "kinds": ["person"] } }
```

- `id`, `label`: required.
- `filter.kinds`: optional list of entity kinds to fit the camera to; omit to
  fit the whole projection.
- `measure`: optional, `"betweenness_top_n"` or `"single_tie"` (UNIT 2);
  `topN` applies only to `betweenness_top_n`.

Edit the file, save, and reload the app - it's fetched once at boot.

## Live entry loop (facilitator, real data)

Full detail: `docs/runbooks/live-entry.md`. The loop itself:

1. Open the real queue folder (granted once via the intake panel's directory
   step - outside this repo, outside any cloud-synced tree).
2. A facilitator enters and a facilitator approves in the wizard (two
   distinct acts).
3. Run `cn intake apply --queue <folder> --ops <group's .ops.jsonl> --group <group-uuid> --facilitator <facilitator-person-uuid>`.
4. Reload the group in the app to see the applied entries.

Real data is never copied into this repository.
