# Intake Form Wording - Draft v2 for D-023 Human Review (2026-09-14)

> **DRAFT - COMMUNITY-FACING TEXT - NOT FOR USE until D-023 human review is
> recorded.**
>
> This draft SUPERSEDES the consent statement and remote-form wording in
> `docs/design/intake-consent-text-draft-2026-07-24.md` (sections 1 and 2 of
> that file). That file's consent email (section 3), sealed-envelope explainer
> (section 4), reviewer checklist (section 5), open items (section 6), and the
> ADR-005 round-1 truthfulness findings (section 7) still apply and are not
> restated here; every section-7 correction is carried into this wording.
>
> Status: PENDING D-023. The form shows the DRAFT status tag until the sign-off
> is recorded in DECISIONS.md. Nothing here may be wired into an email, a
> printed card, or a deployed page before that entry exists.
>
> Conventions (ATNI house style and the public-facing voice): institutional
> third person ("we", "our", and "I" never speak for ATNI; the participant is
> "you"); hyphens and semicolons, never em dashes; Oxford comma; Title Case
> headings and buttons, sentence case everywhere else; numbers spelled out in
> prose; Tribe, Tribal, and Tribal Nation always capitalized; the Tiered
> Sovereign Data Framework expanded on first use, then "the framework". No real
> names, no real contact information anywhere in this file; placeholders stay
> in [BRACKETS].

---

## 0. What changed since the 2026-07-24 draft

- The wording moved to the ATNI house voice: institutional third person
  (every "we"/"our" removed), bold lead-in + colon pattern, no em dashes.
- The consent statement names the Tiered Sovereign Data Framework in full on
  first use (the 07-24 draft said "the community's data framework"; that file's
  vocabulary flag on this point is resolved in favor of the full name, pending
  the reviewer's confirmation) and keeps the T1 code (D-032, codes primary).
- The remote form is now built against the ATNI convention template
  (`fixtures/templates/atni-convention.template.json`) with the `person` kind
  only, so the field set below replaces the 07-24 draft's four-field set.
- Page title, heading, intro sentence, field labels, help text, button labels,
  and every status message are drafted here for the first time.
- The consent text digest (`consent_text_digest`, ADR-005 D5) changes with the
  wording; the `form/src/consent.test.ts` golden and the D8 manifest carry the
  new value. Any submission sealed under the 07-24 wording carries the old
  digest, as designed.

Source of truth for the strings the page renders: `form/src/consent.ts`
(consent statement, heading, tag, affirmation), `form/src/config.ts` (labels
and help text, keyed to the `atni-convention` template id), `form/src/render.ts`
and `form/src/main.ts` (page copy, buttons, status messages), and
`form/index.html` (title tag). This file records the same words verbatim for
the reviewer.

---

## 1. Page chrome

> Browser tab title: **Community Connector: Join the Network Map (DRAFT)**
>
> Wordmark, top-left, text only: **ATNI** over **CLIMATE**
>
> Heading: **Add Yourself to the Network Map**
>
> Intro sentence: Community Connector maps the people, committees, and
> organizations at work across the Affiliated Tribes of Northwest Indians
> (ATNI) so that relatives pursuing the same priorities can find one another.
>
> Status tag: **DRAFT wording, pending human review (D-023). Not for
> community use.**

Vocabulary flags: "Community Connector" (the product is "Community Navigator"
in the repository contract; the human's 2026-09-14 direction names the
community-facing product "Community Connector", and the terminology guide lists
"ATNI Community Connector"; confirm which name the public sees); "network map";
"relatives" (the voice's own term; confirm it reads as intended to a first-time
participant).

## 2. The form fields (ATNI convention template, person kind)

Labels appear exactly as written; "(required)" follows the one required label.
Optional fields carry no marker. The "What are you adding?" kind selector is
not shown: the ATNI build offers the person kind only.

| Template attribute | Label shown | Help or placeholder |
|---|---|---|
| `display_name` | **Name (required)** | |
| `tribe` | **Tribal Nation or organization** | |
| `role` | **Role** | |
| `areas_of_interest` | **Priority areas** | Help text under the label: "One or two areas, one per line." Placeholder: "One per line" |
| `specialties` | **Specialties** | Placeholder: "One per line" |
| `events_of_interest` | **Events you plan to attend** | Placeholder: "One per line" |
| `contact_email` | **Email** | |
| `contact_preference` | **How you prefer to be reached** | Choices: email, phone, no-contact; first option reads "Choose one" |

Per-field advisory messages (unchanged from the built form, shown under the
field): "This field is required." / "Enter a number." / "Keep this under 2000
bytes." / "List at most 20 items." / "Keep each item under 2000 bytes." / "Keep
the place name under 2000 bytes." / "Pick one of the listed choices."

Vocabulary flags: "Tribal Nation or organization" (one field carries both; the
template has a single `tribe` attribute); "Priority areas" versus the
template's `areas_of_interest`; the enum values "email", "phone", and
"no-contact" are template data and render as typed.

## 3. Consent statement (shown above the checkbox, before the send button)

> **Before You Send This**
>
> - **What the form keeps:** only what is typed here. Nothing else is gathered
>   from you or your device.
> - **A person reviews it first:** every entry goes to the network facilitator,
>   a person working for the ATNI Climate program. Nothing appears in the
>   network until the facilitator has read and approved it; anything that
>   looks off is set aside rather than published.
> - **How it is classified:** everything entered here is held at Tier 1 (T1)
>   of the Tiered Sovereign Data Framework, shared within the network and
>   governed by the community. Under the framework, ATNI Climate decides how
>   it is classified and used; that decision never belongs to a company or to
>   a server.
> - **Taking part is your choice:** every question except your name is
>   optional. You can stop at any time before sending, and nothing is kept.
>   After sending, you can ask to be removed at any time, and your information
>   will no longer be shown in the network.
> - **To be removed or to ask a question:** contact [REMOVAL CONTACT, set at
>   deployment; never a real name or address in this repository].
> - **Your answers are sealed on your phone:** your answers are locked
>   (encrypted) on your own phone before they are sent. The internet services
>   that carry the message cannot read them; the key that opens them is used
>   only on the facilitator's computer.
>
> [ ] **I understand, and I agree to be included in the network at the level
> described above.**
>
> Button label: **Send to the Facilitator**

Builder notes:
- The checkbox is the individual consent instrument (D-030). Unchecked means
  nothing sends; there is no partial submission. The affirmation is the
  participant's own statement, so its "I" is theirs, not the institution's.
- The six items are the digest-covered consent text, joined in display order
  with the affirmation; the heading and the DRAFT tag are UI chrome and are
  excluded from the digest.
- Truthfulness corrections from the 07-24 draft's section 7 are all kept: (1)
  the collection claim is scoped to what is typed; (2) the key is "used only
  on the facilitator's computer", never "the only copy"; (3) removal is worded
  as "no longer be shown"; (4) the confirmation screen promises acceptance for
  delivery, not receipt (section 4 below).
- "ATNI Climate program": the 07-24 draft said "ATNI Climate community". The
  reviewer picks the noun.

Vocabulary flags: "facilitator" (still the most load-bearing term);
"network"; "the ATNI Climate program"; whether the T1 gloss "shared within the
network and governed by the community" is the wording ATNI Climate wants;
"a company or a server" (plain-language stand-in for "no vendor and no hosting
provider decides").

## 4. Status messages and the confirmation screen

While sending (live region):

> Sealing your answers on this device.
>
> Sending your sealed answers.

Confirmation screen (after the relay accepts the sealed envelope):

> **Thank You**
>
> Your sealed answers were accepted for delivery to the facilitator. Nothing
> appears in the network until the facilitator has read and approved them.
>
> [DRAFT tag]
>
> Button: **Add Another Response**

Errors (each with a **Try Again** button unless noted):

> This form is out of date and cannot send. Reload the page, or scan the code
> again, and then try once more. (no button; the send button stays disabled)
>
> Too many submissions are arriving right now. Wait [N] seconds and try again.
> (or: Wait a moment and try again.)
>
> The service is temporarily unavailable. Try again shortly.
>
> The service could not be reached, so your answers were not sent. Check your
> connection and try again.
>
> Something went wrong (code [N]). Your answers were not sent.
>
> Your answers could not be prepared on this device. Reload the page and try
> again. (no button)

Page cannot start (replaces the whole form):

> **This Form Is Not Available**
>
> The intake key failed its own integrity check. Tell the facilitator, and do
> not enter anything.
>
> (or) Something went wrong while preparing this form. Reload the page and try
> again.

## 5. Reviewer checklist additions for this draft

In addition to the 07-24 draft's section 5 checklist:

- [ ] Product name the public sees: "Community Connector" as directed, or
      "ATNI Community Connector" per the terminology guide.
- [ ] The intro sentence's "relatives" and "priorities" read correctly to a
      first-time participant at the convention.
- [ ] "Tribal Nation or organization" as a single field label is acceptable
      for the pilot (the template carries one `tribe` attribute).
- [ ] The full framework name on first use, then "the framework", is the
      wording ATNI Climate wants in consent text.
- [ ] Every status message promises only what ADR-005 guarantees.
