# Controls

Launch with `pwsh scripts/reveal.ps1`. It opens `?group=atni-convention` (the
synthetic convention fixture); `-Group research-network` opens the research
demo. Don't use `-Built` for the reveal: the production build loads no data.

## Presenter mode

Click the **Present** toolbar button once beats have loaded. Click a node to
focus it. No person's name is ever rendered on stage in present mode (D-099):
the hover tooltip is suppressed, and each beat's `labelKinds` limits node
labels to committees and organizations. Hover names return on exit.

| Key | Action |
| --- | --- |
| `Space` / `ArrowRight` | Advance to the next beat |
| `ArrowLeft` | Back to the previous beat |
| `Home` | Jump to the first beat |
| `F` | Zoom-to-fit the whole network (camera only, no beat change) |
| `r` | Toggle the operator's button rail (hidden by default in present mode) |
| `c` | Jump to the `committees` beat |
| `o` | Jump to the `organizations` beat |
| `m` | Jump to the `members` beat |
| `p` | Jump to the `shared-priorities` beat |
| `1` | Jump to the `one-node` beat |
| `End` | Jump to the `constellation` beat |
| `Escape` | Exit presenter mode |

Every key above only does something while presenter mode is active. The
letter and `1`/`End` keys jump straight to a beat by id (a small ARIA-labeled
button rail mirrors them on screen once `r` shows it);
`Space`/`ArrowRight`/`ArrowLeft`/`Home` still step through the sheet in order.

## A9 cue sheet (2026-09-15)

The eight-beat sheet in `app/public/beats.atni.json` ends on the constellation
finale for act A9 of the General Assembly session (D-103). Before the reports,
the browser is already open via `pwsh scripts/reveal.ps1`, **Present** has been
clicked, and the display is pre-loaded on beat 5 (`connectors`) - or held on
the venue's thanks slide per the operator's call - so nothing needs to be
touched during the floor conversation.

At the spine's single documented cue, press `Space` exactly once per line and
nothing else:

| Spine line | Press | Beat reached |
| --- | --- | --- |
| "communities connected in new ways" | `Space` | `one-node` |
| "But the ways that matter most are as relatives" | `Space` | `shared-priorities` |
| "Relationships. Being a good relative. Being a good ancestor." | `Space` | `constellation` |

Nothing is pressed after the third cue - the constellation beat holds through
applause. `Escape` is never pressed during A9 (it exits presenter mode). The
button rail stays hidden on stage: `r` is not pressed during A9, and the
display shows only the graph and the caption - never a person's name.

**Recovery if the wrong beat shows:** press `Home`, then `Space` seven times -
that walks `network-overview -> members -> committees -> organizations ->
connectors -> one-node -> shared-priorities -> constellation`, landing back on
the finale regardless of where the display was.

**Alternative (needs a spine edit, human's call):** a pre-switch at "These
tools can be more" instead of the single documented cue, giving four Space
presses spread earlier in the close rather than three bunched at the end.
Not wired up unless the human asks for the spine change (D-103.7).

**Operator:** ________ (OQ-11)

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
- `labelKinds`: optional list of entity kinds whose labels may render on
  this beat; every other label (the focused node and its neighbors included)
  is hidden. Every beat in the ATNI sheet carries it without `person`, and
  `npm run test` enforces that (D-099).

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
