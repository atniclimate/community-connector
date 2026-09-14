# Intake consent statement v3 (human-authored, 2026-09-14)

Source: the human's message on 2026-09-14, verbatim. Supersedes
`intake-consent-text-draft-2026-09-14.md` (draft v2, session-drafted) and the
2026-07-24 draft. This is the wording the QR form and the in-app facilitator form carry
(D-106). D-023 sign-off status is recorded in DECISIONS.md, not here.

---

This is a demonstration of the connections between all of us here; where effort
overlaps, and brings to light what is already here between us.

A name badge tells you who is here. This asks what holds us together.

Everything you share here stays here. This runs entirely on ATNI software; no outside
platforms, no third-party services. The only data collected is what you enter into this
form. Nothing more.

How your information is held: Everything entered here is designated Tier 1 of the Tiered
Sovereign Data Framework; shared within the network and governed by the community it
belongs to.

Sealed on your phone. Your answers are encrypted on your own device before they are sent.
The internet services that carry the message cannot read them; the key that opens them is
used only on the facilitator's computer.

Nothing appears in the network without care and human review. The content and narrative
remains yours, and what is presented are the common connection points.

After the conference ends on Wednesday, all information entered here is deleted from the
system entirely.

The connections you make, the conversations that follow, the shared experiences; those
are yours to keep.

[ ] I understand and consent to my information being used for this demonstration.

---

## Session notes for the author (truthfulness under ADR-005)

1. "This runs entirely on ATNI software; no outside platforms, no third-party services."
   The software is ATNI's, but if the QR path goes live the form is served by GitHub
   Pages and the sealed messages travel through a Cloudflare Workers relay (D-059.8 rows).
   Those carriers hold only ciphertext, which the next paragraph already says. A wording
   that stays true either way: "This runs on ATNI software; no outside platform holds
   your answers, and no third-party service can read them."
2. "After the conference ends on Wednesday, all information entered here is deleted from
   the system entirely." This is a commitment that needs an operational step: a purge of
   the relay queue, the facilitator's staging folder, the applied group ops, and any
   exports, executed and recorded (the D-059.11 purge sweep is the existing hook). Until
   that step is scheduled and owned, the sentence promises more than the system
   guarantees on its own.
3. In-app facilitator entry cannot say "Sealed on your phone"; that path carries the
   in-app variant recorded in `app/src/ui/forms/consent.ts`.
