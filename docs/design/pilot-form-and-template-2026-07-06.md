# Pilot FORM Deliverable: Intake Form, ATNI Template Structure, Consent Email

> The earliest critical-path artifact for the ATNI convention pilot (Execution
> Plan v2, "FORM"; integration plan R1). It is the instrument that shapes the
> entire pilot dataset, so it is drafted before the consent email goes out and
> needs no code. Conventions: hyphens, never em dashes.
>
> **STATUS: DRAFT FOR HUMAN REVIEW.** Per D-023, all community-facing text here
> (the intake form wording and the consent email) must be reviewed by the human
> - and ideally by ATNI Climate - before any use. Nothing here is sent or
> published. No real data is collected by this document.
>
> **Two gates this deliverable deliberately respects:**
> - **G1 (vocabulary authority).** The capability vocabulary - the actual list
>   of offer/need/situation terms - is **PARKED**, not filled in. This document
>   provides the *structure* (the slots and their shape); ATNI Climate authors
>   the *terms* in its own words (see Part F). Every place a term list belongs,
>   this document leaves it empty and marked `[ATNI Climate authors - G1]`.
> - **G2 (licensing).** No Open Eligibility (CC BY-SA) terms are embedded. The
>   *structure* borrows the taxonomy-agnostic HSDS shape (a controlled tag set +
>   a situation facet); the *terms* are ATNI's own.

---

## 0. Design principles (D-023), and how each shows up below

1. **Offers and needs draw from ONE shared capability vocabulary** so
   need-to-solution routing computes directly. In the form, the "what I can
   offer" and "what I am looking for" questions draw from the *same* term list
   (Part A, Q7/Q8; Part B, the `capabilities` tag vocabulary on Person).
2. **Edge-generating questions outrank attribute questions.** The optional depth
   section is mostly relationship questions - collaborators, projects,
   committees, convenings - because an edge is worth more to the graph than an
   attribute (Part A, Section 2).
3. **Response rate is protected:** a required ~5-minute core, an optional depth
   section, and facilitator-assisted completion. Only the core is required.
4. **Capacity and contactability are routing-critical.** Each offer can carry a
   capacity level, and a single contactability-consent question (yes / through
   the facilitator / no) governs whether a person can be a routing endpoint
   (Part A, Q7 and Q11; this feeds the *structural* consent gate in the routing
   engine, integration plan spec 6.2).
5. **Per-field visibility consent is collected at the source** and maps to the
   permission engine's circles (Part A, the "who can see this" column; Part B,
   `default_visibility`).
6. **Never ask for enumeration of traditional or cultural knowledge holdings.**
   The form never asks what sacred, ceremonial, or traditional knowledge a
   person holds - only, at most, whether they are willing to be contacted about
   a topic, at the most protected visibility. (Part A explicitly omits any such
   question.)

All pilot entries enter at **TSDF tier T1** (D-034); ATNI Climate is the tier
authority. The form uses plain-language consent choices for participants; the
mapping to circles/tiers is documented for the builder, and the app's detail
panel shows TSDF codes primary per D-032.

---

## Part A. The intake form (participant-facing)

> Draft wording. Each field is annotated for the builder: **-> maps to** (the
> template kind/attribute/edge it becomes), **visibility** (the default circle),
> and notes. Participants see only the questions and the plain-language consent
> choices, never the annotations.

### Section 1 - Required core (~5 minutes)

**Q1. Your name (or the name you want shown in the network).**
-> Person.`display_name` (text, required). Visibility: group.

**Q2. Your Tribe or Nation.**
-> Person.`tribe` (text). Visibility: group. Notes: free text; do not constrain
to a picklist (sovereignty - people name their own affiliation).

**Q3. Organization(s) you work with.**
-> creates/links Organization node(s) + `affiliated_with` edge(s). Visibility:
group. Notes: edge-generating; one row each, "organization name + your role."

**Q4. Where are you based / what area do you work in?**
-> Person.`based_in` (text/region). Visibility: group. Notes: plain place name
or region for the pilot (geography-as-structure is deferred to v0.2); this is a
label, not a coordinate.

**Q5. Your areas of focus / specialties.**
-> Person.`specialties` (tags). Visibility: group. Notes: this is a *description*
facet (who you are), distinct from Q7's offers (what you can provide).

**Q6. How would you like to be contacted?**
-> Person.`contact_email` (link/email) and/or `contact_other` (text).
Visibility: **trusted** (contact details default to a tighter circle).

**Q7. What can you offer the network?** (Choose from the list; add your own.)
-> Person.`offers` (tags, drawn from the shared `capabilities` vocabulary) +
optional per-offer capacity. Visibility: group. **Vocabulary PARKED (G1).**
Notes: the picklist is the ATNI-authored capability list; "add your own" keeps
it extensible/refusable. Optional capacity per offer: "a little / some / a lot"
-> stored as a capacity qualifier (routing-critical, principle 4).

**Q8. What are you looking for / what would help your work?** (Same list.)
-> Person.`needs` (tags, **same** `capabilities` vocabulary as Q7) + optional
urgency ("someday / this year / now"). Visibility: group. **Vocabulary PARKED
(G1).** Notes: offers and needs sharing one term list is what makes routing
compute (principle 1).

**Q9. Which committee(s) or working group(s) are you part of?**
-> `member_of` edge(s) to Committee/Group node(s). Visibility: group.
Edge-generating.

**Q10. Consent to be included in the network.** "May we include you and the
information you have shared, at the visibility levels you chose, in the ATNI
climate network visualization?" (yes / no)
-> gates the whole record. Notes: **this is the individual consent instrument
(D-030).** No is a complete stop for that record.

**Q11. Contactability.** "If someone in the network could benefit from what you
offer, how may they reach you?" (a) directly / (b) through the facilitator only
/ (c) please do not surface me as a contact.
-> Person.`contactability` (enum: direct / facilitator_only / none).
**Routing-critical (principle 4).** Notes: this drives the **structural routing
consent gate** (integration plan spec 6.2) - a person choosing (b) or (c) is
never rendered as a directly reachable routing endpoint; at most the path stops
at the facilitator.

### Section 2 - Optional depth (edge-generating; skippable)

**Q12. Who do you already collaborate with in this space?**
-> `collaborates_with` edges (Person-Person). Visibility: group. Notes: the
highest-value question in the form for graph richness - name people; if they are
also respondents the edge connects two real nodes, otherwise it seeds a
second-degree node (kept minimal for the pilot).

**Q13. What projects or initiatives are you working on?**
-> Project node(s) + `works_on` edges. Visibility: group.

**Q14. What convenings, gatherings, or past conventions have you taken part in?**
-> Convening node(s) + `attended` edges. Visibility: group. Notes: connects
people through shared events - a strong affinity signal.

**Q15. Social / professional links (optional).**
-> Person.`links` (tags/link). Visibility: **trusted**.

**Q16. A short story: a time a connection helped your work.** (2-4 sentences.)
-> seeds a Story (D-037, integration plan R6). Visibility: group (with explicit
opt-in). Notes: this is the material for the guided tours that make the graph
readable at the assembly - the *measured* remedy for graph-illiteracy. Ask
permission to feature it.

### Section 3 - Per-field visibility (principle 5)

A short "who can see this?" control on the sensitive fields (contact details,
links, story), offering plain-language choices that map to circles:
- "Everyone in the network" -> **group**
- "Only people I have a trusted connection with" -> **trusted**
- "Only me / facilitator" -> **private**
Default per field is as annotated above; participants can tighten.

### What the form deliberately does NOT ask (principle 6)

No question asks a person to enumerate traditional, ceremonial, sacred, or
cultural knowledge they hold. If a future version ever touches this, the only
admissible question is willingness-to-be-contacted about a topic, stored at the
most protected visibility, authored by ATNI Climate.

---

## Part B. The ATNI climate template structure (schema-as-data)

> A SKELETON, not a committed fixture. Vocabulary arrays are intentionally
> **empty** and marked `[ATNI Climate authors - G1]`. It becomes a real
> `fixtures/templates/atni-climate.template.json` only once ATNI's terms exist,
> or with a clearly-labeled illustrative-only vocabulary for dev testing. It
> validates against `schemas/group-template.schema.json`. Per integration plan
> spec 6.2, **offers and needs are tags ON Person** (Person is the routing
> endpoint), not separate resource nodes.

```jsonc
{
  "schema_version": "0.1.0",
  "template_id": "atni-climate",
  "name": "ATNI Climate Resilience Network",
  "description": "Template for the ATNI Climate Resilience Committee pilot: people, organizations, committees, projects, convenings, and the shared capability vocabulary that powers need-to-solution routing. Capability TERMS are authored by ATNI Climate (G1); this skeleton ships the STRUCTURE only.",
  "kinds": [
    {
      "id": "person",
      "label": "Person",
      "attributes": [
        { "id": "display_name",   "type": "text",  "required": true, "default_visibility": "group" },
        { "id": "tribe",          "type": "text",  "default_visibility": "group" },
        { "id": "based_in",       "type": "text",  "default_visibility": "group" },
        { "id": "specialties",    "type": "tags",  "default_visibility": "group" },

        // OFFERS and NEEDS draw from the SAME parked vocabulary (principle 1, spec 6.2).
        { "id": "offers",         "type": "tags",  "values": [], "default_visibility": "group",
          "note": "[ATNI Climate authors the capability term list - G1]" },
        { "id": "needs",          "type": "tags",  "values": [], "default_visibility": "group",
          "note": "[same capability vocabulary as offers - G1]" },

        // Routing-critical (principle 4, spec 6.2 structural consent gate).
        { "id": "contactability", "type": "enum",
          "values": ["direct", "facilitator_only", "none"], "default_visibility": "group" },

        { "id": "contact_email",  "type": "link", "format": "email", "default_visibility": "trusted" },
        { "id": "contact_other",  "type": "text",  "default_visibility": "trusted" },
        { "id": "links",          "type": "tags",  "default_visibility": "trusted" }
      ]
    },
    { "id": "organization", "label": "Organization",
      "attributes": [
        { "id": "display_name", "type": "text", "required": true, "default_visibility": "group" },
        { "id": "org_type",     "type": "enum", "values": [], "default_visibility": "group",
          "note": "[ATNI Climate authors org types if wanted - G1; else omit]" }
      ] },
    { "id": "committee", "label": "Committee / Working Group",
      "attributes": [
        { "id": "display_name", "type": "text", "required": true, "default_visibility": "group" }
      ] },
    { "id": "project", "label": "Project",
      "attributes": [
        { "id": "display_name", "type": "text", "required": true, "default_visibility": "group" },
        { "id": "summary",      "type": "text", "default_visibility": "group" }
      ] },
    { "id": "convening", "label": "Convening",
      "attributes": [
        { "id": "display_name", "type": "text", "required": true, "default_visibility": "group" },
        { "id": "date",         "type": "date", "default_visibility": "group" }
      ] }
  ],
  "edge_kinds": [
    { "id": "affiliated_with",  "label": "works with",      "from": ["person"], "to": ["organization"], "directed": true },
    { "id": "member_of",        "label": "is part of",      "from": ["person"], "to": ["committee"],    "directed": true },
    { "id": "collaborates_with","label": "collaborates with","from": ["person"], "to": ["person"],      "directed": false },
    { "id": "works_on",         "label": "works on",        "from": ["person"], "to": ["project"],      "directed": true },
    { "id": "attended",         "label": "took part in",    "from": ["person"], "to": ["convening"],    "directed": true },

    // Routing edge, derivable from the shared offers/needs tags (spec 6.2).
    { "id": "need_met_by",      "label": "could be met by", "from": ["person"], "to": ["person"],
      "directed": true, "weighted": "optional",
      "note": "Emitted at ingest OR computed live from matching needs/offers tags - see spec 6.2." }
  ],
  "theme": { "mode": "default-dark", "roles": { "note": "[DESIGN sitting D-038 sets the palette; Hearthlight provisional]" } }
}
```

Notes for the builder:
- The `capabilities` vocabulary is a single controlled tag set shared by
  `offers` and `needs`; it is the only sovereignty-sensitive content and stays
  empty until G1.
- Person is the routing endpoint. `skill_resource`-style resource nodes (as in
  the fisheries template) are intentionally omitted for the pilot to keep the
  routing endpoint = the person (spec 6.2). They can return in v0.2 for richer
  asset modeling.
- `theme.roles` is left to the DESIGN sitting (D-038); do not hardcode a palette
  here.

---

## Part C. Consent email (draft)

> DRAFT. To the Climate Resilience Committee and documented past-convention
> attendees, before the convention. Must be reviewed by the human and ideally
> approved by ATNI Climate before sending (D-023). Placeholders in [brackets].

Subject: An invitation - help map the ATNI climate network

Dear [name],

Ahead of the [year] ATNI Annual Convention, the Climate Resilience Committee is
building a picture of the people, organizations, and knowledge that make up our
climate-resilience network - so that a need in one place can more easily find
the person or resource that can help, and so we can all see the connections we
are part of.

We would like to invite you to take part. Taking part means filling out a short
form (about five minutes) about your work: your Tribe and organization, what you
focus on, what you can offer the network, what would help your work, and how -
if at all - you would like others to be able to reach you. You choose what is
shown and who can see each part; you can decline any question; and you can ask
to be removed at any time.

What we build from these responses is a visual map of the network that we will
share at the general assembly, and explore together during the committee
meeting. Your information is kept by the community for this purpose and is not
sold, shared outside the network, or used for anything else. [ATNI Climate
governs how the information is classified and used.]

If you would like to take part, please fill out the form here: [form link].
You will also find a QR code to the same form in your convention packet, so you
can join at the convention itself.

If you have any questions, or would like help filling out the form, please
reach out to [facilitator name, contact].

With respect and thanks,
[Committee / facilitator signature]

> Review notes: (1) confirm the sovereignty/governance sentence with ATNI
> Climate - it should reflect their actual data-governance language if they have
> it (D-032/Q4.3). (2) Confirm the "remove at any time" promise is operationally
> honored by the pipeline before the email says it. (3) The collective FPIC
> checkpoint (a recorded committee approval of the activity) should be in place
> before this goes out (CLAUDE.md real-data gate process).

---

## Part D. Feedback capture (committee meeting)

The pilot arc ends with feedback that improves the system (D-022). Draft plan,
low-tech by design so it does not depend on more software:
- A short paper or single-form prompt at the committee meeting: "What did you
  see about yourself or your connections that you did not expect? What is
  missing? What would make this useful to you?"
- Capture is outreach/product feedback, not graph data - it does not enter the
  network and is not subject to the T1 pipeline.
- The facilitator notes which routing pathways people explored and where the
  graph felt wrong or thin - this shapes plan v3 and the v0.2 priorities.

---

## Part E. How this flows into the pipeline

1. Participants complete the form (platform TBD - gate Q-B); responses export as
   CSV.
2. The facilitator runs `cn ingest` on the CSV. The importer
   (`AtniIntakeBatchV0_1`, integration plan spec 6.1) maps columns to the Part B
   template, resolves each response to a stable identity (deterministic UUIDv5
   from the form-response id), stamps `Origin::Ingested` + tier T1, appends one
   `Imported` custody event, and re-imports idempotently (edits update, they do
   not duplicate).
3. The graph renders; at the assembly it is shown via the flat/list reveal
   projection first, then the 3D constellation (integration plan R4/R6).
4. In the committee meeting, need-to-solution routing surfaces explainable
   pathways, gated by the `contactability` value (Q11) at the query layer so a
   "facilitator-only" or "no" person is never a directly reachable endpoint.
5. Same-day QR joiners are re-ingested and the snapshot rebuilt before the
   pathway exploration.

---

## Part F. Open items for the human

- **G1 - the capability vocabulary (the parked terms).** The single most
  important input this deliverable is waiting on. Recommended method (from the
  research, Net-Map co-construction): a facilitator-led session with the Climate
  Resilience Committee that elicits, in the committee's own words, the list of
  things people offer and need (one shared list), plus any situation/audience
  terms - converted into the Part B `capabilities` vocabulary. This should
  happen before the form goes out.
- **Q-B - the form platform** (Google Forms / Microsoft Forms / other) - fixes
  the exact CSV export the importer maps.
- **Review all community-facing text** in Parts A and C (D-023), ideally with
  ATNI Climate; confirm the governance/sovereignty wording (Q4.3/D-032) and the
  "remove at any time" promise.
- **Confirm the collective FPIC checkpoint** (recorded committee approval) is in
  place before the consent email sends (CLAUDE.md real-data gate process).
