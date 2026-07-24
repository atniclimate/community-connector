# Intake Form and Consent Text - Draft Package for D-023 Human Review

> **DRAFT - COMMUNITY-FACING TEXT - NOT FOR USE until D-023 human review is
> recorded.**
>
> This file must NOT be referenced from any live UI, deployed form, email, or
> printed material until the D-023 review sign-off is recorded in DECISIONS.md.
> It exists only so the human reviewer (and ideally ATNI Climate) can read,
> correct, and approve the exact words a community member will see.
>
> Scope: the D-053 remote intake form (the QR / GitHub Pages path), the consent
> statement shown before submit, the pre-convention consent email (D-050 window),
> and the plain-language sealed-envelope explainer. The fuller in-app form
> question set remains in `docs/design/pilot-form-and-template-2026-07-06.md`;
> where the two disagree on wording, this file is newer.
>
> Conventions: hyphens, never em dashes. No real names, no real contact
> information anywhere in this file - placeholders only, in [BRACKETS].

---

## 0. How to read this document

- **Quoted blocks** are the exact words a community member would see, drafted
  in plain standard language per D-051. ATNI Climate authors the final
  vocabulary after the system is stable; until then this is developer-standard
  wording offered for review, not final community language.
- **[PLACEHOLDER]** marks text that must be filled in by the human before any
  use. No placeholder may be filled with a real person's contact information
  inside this repository (prime directive; I1).
- **Vocabulary flags** after each block list every term ATNI Climate may want
  to reword (D-051). The flag list is part of the review package: the reviewer
  either accepts the standard term for the pilot or supplies ATNI's word.
- **Builder notes** explain where each field goes technically. Community
  members never see them.

---

## 1. The remote intake form (D-053 field set)

The static form collects exactly four content fields - name, Tribe or Nation,
organizations, and roles - plus the consent confirmation. Nothing else. The
form runs in the phone's browser; nothing is installed; answers are encrypted
on the phone before they are sent (section 4).

### Field 1 - Name (required)

> **Your name**
>
> The name you want shown in the network map. It can be your full name, or
> just what people know you by.

Builder note: maps to Person.`display_name`, required text. The only required
content field.

Vocabulary flags: "network map" - ATNI may prefer another name for the whole
artifact (for example "our climate network" or a name of its own choosing).

### Field 2 - Tribe or Nation (optional)

> **Your Tribe or Nation**
>
> Name your affiliation in your own words. This is optional, and there is no
> list to pick from - you write it the way you say it.

Builder note: maps to Person.`tribe`, free text, never a picklist
(sovereignty; people name their own affiliation).

Vocabulary flags: "Tribe or Nation" - the label itself is ATNI's to word;
"affiliation" may want a warmer term.

### Field 3 - Organizations (optional)

> **Organizations you work with**
>
> List the organizations, programs, or departments you work with - one per
> line. You can list one, several, or none.

Builder note: each line creates or links an Organization node plus an
`affiliated_with` edge. Edge-generating.

Vocabulary flags: "organizations, programs, or departments" - confirm this
covers the bodies ATNI communities actually name (consortia, councils,
inter-tribal bodies); "work with" versus "belong to".

### Field 4 - Roles (optional)

> **Your role(s)**
>
> What is your role in that work? For example: staff, director, committee
> member, elder advisor, volunteer, student. Write it your own way - this is
> not a job application.

Builder note: role text attaches to the matching organization edge where the
respondent pairs them, otherwise to the Person record; committee or working
group names given here create `member_of` edges. Free text for the remote
form; no picklist in v0.1.0.

Vocabulary flags: every example role term ("staff", "director", "committee
member", "elder advisor", "volunteer", "student") is a placeholder example set
ATNI may replace entirely; "role" itself may want rewording.

### The submit control

> Button label: **Review my answers**
>
> (A confirmation screen shows the four answers back, then the consent
> statement in section 2, then the final **Send to the facilitator** button.)

Vocabulary flags: "facilitator" - the single most load-bearing term in the
whole package; ATNI may have a preferred title for this role.

---

## 2. Consent statement (shown before submit, both in-app and remote)

The person must see this statement and actively confirm it before anything is
sent. It covers, in order: what is collected, facilitator review, tiering and
authority, voluntariness and withdrawal, removal contact, and encryption.

> **Before you send this**
>
> **What we collect.** Only what you typed on this form: your name, your Tribe
> or Nation if you gave it, the organizations you listed, and your roles.
> Nothing else is collected - no location, no tracking, nothing from your
> phone.
>
> **A person reviews it first.** Your submission goes to the network
> facilitator, a person working for the ATNI Climate community. Nothing you
> send appears anywhere until the facilitator has read and approved it. If
> something looks off, the facilitator will set it aside rather than publish
> it.
>
> **How it is classified.** Everything entered through this form is held at
> Tier 1 (T1) under the community's data framework: shared within the network,
> governed by the community. ATNI Climate decides how information is
> classified and used - not a company, and not a server.
>
> **Taking part is your choice.** Every question except your name is optional.
> You can stop at any time before sending, and nothing is kept. After sending,
> you can ask to be removed at any time, and your information will be taken
> out of the network.
>
> **To be removed or to ask a question**, contact [REMOVAL CONTACT - to be
> filled by the human at deployment; do not put a real name or address in this
> repository].
>
> **Your answers are sealed on your phone.** If you are using this form from a
> QR code, your answers are locked (encrypted) on your own phone before they
> are sent. The internet services that carry the message cannot read it - only
> the facilitator's computer holds the key that opens it.
>
> [ ] **I understand, and I agree to be included in the network at the level
> described above.**
>
> Button label: **Send to the facilitator**

Builder notes:
- The checkbox is the individual consent instrument (D-030). Unchecked means
  nothing sends; there is no partial submission.
- "Tier 1 (T1)": TSDF code included per D-032 (codes primary in the UI), with
  the plain-language meaning alongside for the consent context.
- The encryption paragraph must stay literally true to the ADR-005
  implementation (libsodium sealed box, private key only on the pilot PC). If
  the implementation changes, this text changes first.
- The in-app facilitator-entry path shows the same statement; the facilitator
  reads it aloud or hands the device over, and the last paragraph (phone
  encryption) is hidden when the entry is made directly on the pilot PC.

Vocabulary flags: "facilitator"; "the community's data framework" (whether to
name the Tiered Sovereign Data Framework in full); "Tier 1 (T1)" plain-gloss
wording ("shared within the network, governed by the community"); "network";
"included in the network"; whether "ATNI Climate" should read "the ATNI
Climate Resilience Committee" in community-facing text.

---

## 3. Consent email draft (pre-convention window)

To committee members and documented past participants, sent in the D-050
window: after the recorded collective checkpoint exists, aiming for
completions by early September (soft deadline) ahead of the convention.
Subject to the same D-023 sign-off as everything else here.

> Subject: An invitation - help map our climate network
>
> Dear [NAME],
>
> Ahead of this year's ATNI Annual Convention, the Climate Resilience
> Committee is building a living picture of the people and organizations in
> our climate work - so that a need in one place can find the person who can
> help, and so we can see the connections we are already part of.
>
> We would like to include you. Taking part means answering four short
> questions - your name, your Tribe or Nation, the organizations you work
> with, and your role - about two minutes. Every question except your name is
> optional.
>
> A few things we want you to know up front:
>
> - A facilitator from our committee personally reviews every submission
>   before it appears anywhere.
> - Your answers are encrypted on your own phone before they are sent; no
>   internet service in between can read them.
> - The information stays under the community's governance. It is not sold,
>   not shared outside the network, and not used for anything else.
> - Taking part is voluntary, and you can ask to be removed at any time by
>   contacting [REMOVAL CONTACT - placeholder].
>
> To take part, use this link: [FORM LINK - placeholder]. If you would rather
> wait, there will be QR codes at the convention, and facilitators there to
> help.
>
> If you have questions, or would like help with the form, contact
> [FACILITATOR CONTACT - placeholder].
>
> With respect and thanks,
> [COMMITTEE / FACILITATOR SIGNATURE - placeholder]

Builder notes:
- Do not send before (1) D-023 sign-off is recorded and (2) the recorded ATNI
  Climate collective checkpoint exists (D-030/D-050 - it must precede even the
  first August internal-pilot ingestion).
- The "removed at any time" promise must be operationally true (a working
  removal path in the pipeline) before this email says it.
- "About two minutes" should be re-timed against the real deployed form.

Vocabulary flags: "living picture"; "map"; "our climate network"; the
governance sentence ("stays under the community's governance ... not used for
anything else") should be replaced with ATNI's own data-governance language if
ATNI has it; "facilitator" throughout.

---

## 4. QR / sealed-envelope explainer (attendee-facing, one paragraph)

For convention packets, table cards, or the form's "how does this work?" link.

> **How this works.** Point your phone's camera at the QR code and a short
> form opens in your browser - nothing to install, four quick questions. When
> you press send, your phone seals your answers inside a locked digital
> envelope before they leave your hand. The envelope travels over the internet
> to a mailbox that only holds sealed envelopes - it has no key and cannot
> open them. The one key that opens the envelope lives on a single computer
> kept by the committee's facilitator. The facilitator opens your envelope,
> reads your answers, and only then - after a person has reviewed them - are
> they added to the network map.

Vocabulary flags: "locked digital envelope" and "sealed envelope" (the
central metaphor - ATNI may prefer different imagery); "mailbox";
"facilitator"; "network map".

Builder note: every claim here must match ADR-005 as built: client-side
sealed-box encryption before transmit, ciphertext-only relay, single private
key on the pilot PC, facilitator pending-review queue ahead of any graph
entry.

---

## 5. Reviewer checklist (the D-023 sign-off)

The human reviewer works through this list and records the outcome. Sign-off
is recorded as a DECISIONS.md entry stating: the file and version reviewed
(this filename plus commit hash), the date, changes required or none, and
which vocabulary flags were resolved now versus deferred to ATNI's
post-stability language pass (D-051).

**Truthfulness - every promise must be operationally real before use:**
- [ ] The encryption claim matches the implementation exactly (sealed on the
      phone; relay stores ciphertext only; sole key on the pilot PC). If
      ADR-005 is not yet built as described, this text does not ship.
- [ ] The "a person reviews it first" claim matches the pending-review queue
      as built: no path lets a submission enter the graph without facilitator
      approval.
- [ ] The "removed at any time" promise has a working removal path, and the
      removal contact route actually reaches someone who can act on it.
- [ ] "What we collect" is complete and exact: the deployed form collects the
      four fields and nothing else (no analytics, no hidden metadata beyond
      the technical submission envelope; confirm what the envelope carries and
      whether the text needs to mention it).

**Consent completeness:**
- [ ] The statement covers all six required elements: what is collected,
      facilitator review, T1 tiering with ATNI Climate as tier authority,
      voluntary and withdrawable participation, removal contact, and on-phone
      encryption.
- [ ] The consent checkbox wording is acceptable as the individual consent
      instrument (D-030).
- [ ] The recorded ATNI Climate collective checkpoint is in place, or
      scheduled to be in place, before the first real ingestion (D-030/D-050).

**Language and accessibility:**
- [ ] Reading level: understandable without technical background; read one
      block aloud to someone outside the project as a check.
- [ ] Every vocabulary flag in sections 1-4 has been considered: keep the
      standard term for the pilot, or supply ATNI's word now. Whatever is
      deferred goes to the D-051 post-stability language pass.
- [ ] The sovereignty and governance sentences reflect language ATNI Climate
      is comfortable with, ideally reviewed by the committee itself.
- [ ] Nothing asks about, or implies collection of, traditional, ceremonial,
      sacred, or cultural knowledge (design principle 6 in the pilot FORM
      doc).

**Placeholders and privacy:**
- [ ] All [PLACEHOLDERS] identified, and a plan exists for filling them at
      deployment WITHOUT committing real personal contact information to this
      repository (see open question on the removal contact, below).
- [ ] No real person's name, email, or contact detail appears in the repo copy
      of any of this text (I1).

**Process:**
- [ ] Sign-off recorded in DECISIONS.md as described above; only after that
      entry exists may this text be wired into the in-app form, the Pages
      form, the email, or print.

---

## 6. Open items carried with this draft

- **Removal-contact mechanics.** The deployed Pages form needs a real contact
  route, but the form's source lives in this public repository, and the prime
  directive bars committing personal contact info. Options for the reviewer:
  an organizational role address (not a personal one), a deploy-time
  substitution kept outside the repo, or an in-form "ask the facilitator at
  the event" route. Human decision.
- **Whether this file itself is world-readable.** The D-055 pre-publish sweep
  flags community-facing text pending D-023 review as a candidate for the
  gitignored `_private/` staging. Reviewer decides whether this draft rides in
  the public repo or moves until sign-off.
- **In-app long-form wording.** The fuller in-app question set (offers, needs,
  contactability, stories) still carries the older draft wording in
  `docs/design/pilot-form-and-template-2026-07-06.md`; that text needs its own
  D-023 pass before the P3.6 entry forms render it. This package covers the
  D-053 remote field set and the shared consent statement only.

## 7. ADR-005 round-1 findings affecting this draft (added 2026-07-24, for the D-023 pass)

The ADR-005 adversarial round (round 1, gpt-5.6-sol) found four places where
this draft's community-facing claims conflict with the accepted architecture.
The reviewer should reconcile each during the D-023 pass; the ADR's
"Consent-text implications" section carries the same list:

1. **"Nothing else is collected" (section 2, What we collect).** The relay
   operator (Cloudflare) necessarily observes traffic metadata: submission
   counts, sizes, timing, and source addresses (ADR-005 D6, accepted
   residual). The sentence is true of form CONTENT but overclaims as
   written. Suggested direction: scope the claim to "we collect only what
   you typed"; the metadata reality can live in the explainer if the
   reviewer wants it surfaced at all.
2. **"Only the facilitator's computer holds the key" (section 2, sealed
   paragraph).** The keygen ceremony creates two offline recovery copies
   (printed sheet in a sealed envelope; encrypted USB). Accurate wording:
   the key is USED only on the facilitator's computer; locked-away recovery
   copies exist. The reviewer decides how much of that belongs in
   community-facing text, but the current sentence is not literally true.
3. **Removal promise ("your information will be taken out of the
   network").** The underlying change log is append-only (ADR-002): today
   "removal" means the information is no longer shown to anyone, not that
   every stored trace is erased. HUMAN DECISION REQUIRED: either word the
   promise as "no longer appears in the network," or direct a true-erasure
   design (which is new architecture work). This is the largest consent
   question the round surfaced.
4. **Confirmation screen wording (remote path).** A successful send means
   "the relay accepted your sealed envelope" - not that the facilitator has
   it yet; unpulled envelopes expire after a bounded window (ADR-005 D6).
   The confirmation text should promise only what is guaranteed, e.g. "Your
   sealed answers were accepted for delivery to the facilitator," avoiding
   "the facilitator has received it."
