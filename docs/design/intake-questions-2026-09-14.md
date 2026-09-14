# Intake questions for the ATNI convention form (human wording, 2026-09-14)

Source: the human's message on 2026-09-14 ("here are the revised questions, some have
been dropped"). Wording below is verbatim and is community-facing text; the DRAFT tag on
the form stays until D-023 sign-off. Mapping to template attributes is the session's
(D-105); the questions are the human's.

| # | Question | Help text | Attribute (`person` kind) | Type |
|---|---|---|---|---|
| 1 | What is your Name? | How do you prefer to be addressed? | `display_name` | text, required |
| 2 | What do you Do? | Share as many roles, positions, or areas of responsibility as feel right; a single title rarely tells the whole story... | `roles` (new) | tags |
| 3 | Where are you from? | The Tribe(s), Places, Communities, and Organizations that bring you here... | `origins` (new) | tags |
| 4 | What is important to you? | The issues, committees, and areas you devote your time, energy, and thinking to... (e.g. climate, healthcare, human rights, etc.) | `areas_of_interest` | tags |
| 5 | What are you good at? | The things people come to you for, whether or not they show up in a job description... | `specialties` | tags |
| 6 | Who are you connected with? (Optional) | The people, communities, and organizations you carry with you; the ones that stay on your mind when you think about this work... | `connections` (new) | tags |
| 7 | What are you hoping to find? | The connection you came here looking for; a collaborator, a conversation, someone doing similar work, or something you haven't found yet... | `seeking` (new) | tags |
| 8 | What are you here to share? | Your presence matters. The Knowledge, experience, opportunities, or gifts you bring into this room... | `offering` (new) | tags |
| 9 | What committees will you attend? | ATNI committees, working groups, or sessions you plan to participate in... | `committee_memberships` (new on main; same definition as the S-E2 branch) | tags |

Dropped from the QR form (kept in the template so the synthetic fixture and the in-app
facilitator form stay valid): `tribe`, `role`, `events_of_interest`, `contact_email`,
`contact_preference`.

Notes for the human:
- Only question 6 carries "(Optional)" in the wording; every question except the name is
  optional in fact, and the consent text says so. Confirm whether the marker stays on
  question 6 alone.
- Question 6 invites names of third parties who did not consent. Entries are Tier 1,
  facilitator-reviewed, and never rendered on the projector; the D-023 reviewer should
  decide whether the help text should say that third-party names are not shown.
- Questions 7 and 8 are the seeds of need-to-solution routing (R6): what someone hopes
  to find, matched against what others bring.
