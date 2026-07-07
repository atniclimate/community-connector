# Graphs Beyond the Social Network: What a Graph of Many Peoples Could Illuminate

> A research report for Community Navigator. Compiled 2026-07-06 from a
> multi-agent deep-research run (5 search angles, 25 fetched sources, ~120
> extracted claims, adversarial verification) plus two supplemental
> research passes. Every load-bearing claim carries a source URL. Where the
> underlying source is a vendor page or a self-report, this report says so
> and does not launder it into fact. Conventions: hyphens, never em dashes.

## How to read this document

This is not a literature dump. It is organized around the five questions
the director posed, and it ends with the part that matters for us:
Section 6, what this means for Community Navigator and the sibling tools.
If you read one thing, read Section 6, then come back for evidence.

A running theme, stated up front because it recurs in every section: **the
graph is rarely the product; the graph is the reasoning substrate.** In
almost every successful real-world deployment below, users never see a
node-link "hairball." They see a search box, a map, a briefing, a guided
tour, or an answer. The 3D constellation is our reveal moment and our
exploration surface, but the durable value is the queries it makes
possible, not the picture. Hold that tension through everything that
follows.

A second theme, equally load-bearing: **the honest evidence on 3D is
mixed, and the honest evidence on lay-audience graph literacy is
sobering.** This report deliberately includes the findings that argue
against our current design so that plan v3 is built on the real terrain,
not the flattering version of it.

---

## 1. Real-world graph databases beyond social media (the past decade)

The question was whether graph databases are used for real innovation
outside social-network analysis. They are, across at least seven verticals,
and the pattern is consistent: graphs win when the *relationships* between
records - not the records themselves - are the thing you need to compute
over, and when those relationships are many-hop, variable-length, or
densely interconnected.

### Investigative journalism - the flagship public example
The International Consortium of Investigative Journalists (ICIJ) used Neo4j
to process the offshore-leaks investigations. Their CTO's framing is the
cleanest one-sentence case for graph technology in the whole corpus: "With
Neo4j graph technology, we made connections between activities and entities
that otherwise would have been missed," and "relational databases cannot
efficiently analyze relationships within the large, densely interconnected
datasets journalists encounter"
(https://neo4j.com/customer-stories/icij/, fetched 2025-09-10).

Two things matter for us. First, scale: the leaks were in the terabytes and
tens of millions of records. **Caveat, verified during this research:** the
Neo4j page attributes "2.9 terabytes / 11.9 million records" to the Panama
Papers, but ICIJ's own reporting attributes that figure to the *Pandora*
Papers and gives 2.6 TB / 11.5M documents for Panama. The vendor page
likely conflates them. Cite the concept, not the number. Second, and more
important for Community Navigator: **more than 2.6 million people have used
ICIJ's public Offshore Leaks Database to explore graph connections**
(same source). That is hard evidence that non-specialist audiences *will*
engage with a public-facing graph exploration tool when the framing is
"follow the money / follow the connection." That is precisely our reveal.

### Fraud, AML, and financial-crime investigation
This is the highest-volume commercial vertical. Reported deployments
(vendor-sourced, treat magnitudes as marketing but the pattern as real):
Zurich Insurance using Neo4j for fraud detection, "queries that used to
take minutes now come back in milliseconds ... 50,000 hours saved each
year"; a GraphRAG platform ("reView") that "cut analyst workload by 50%
while maintaining decision accuracy"
(https://neo4j.com/blog/graph-database/graph-database-use-cases/ and a
2025-03-20 Neo4j blog). The mechanism that recurs: money-laundering and
collusion are *ring* and *shared-attribute* patterns - circular payment
chains, multiple accounts sharing a device or address - which are
variable-length path queries, exactly what relational joins do badly. A
Neo4j KYC demo detects laundering rings with a single variable-length
Cypher traversal of up to 6 hops over 8,000 synthetic customers
(https://neo4j.com/blog/developer/graphrag-in-action-know-your-customer/,
2025-08-26).

### Biomedical discovery and drug repurposing
The most rigorous non-vendor evidence in the corpus. A 2024 peer-reviewed
review (Briefings in Bioinformatics,
https://pmc.ncbi.nlm.nih.gov/articles/PMC11426166/) documents biomedical
knowledge graphs at serious scale: the Clinical Knowledge Graph has "16
million nodes and over 220 million relationships"; PharMeBINet has ~2.87M
nodes across 66 labels and ~15.88M relationships across 208 edge types;
Hetionet integrates 29 sources into 47,031 nodes and 2.25M relationships.
Drug repurposing is framed as a **link-prediction problem over a graph** -
"does this existing drug treat this other disease?" is literally predicting
a missing edge.

Two findings from this source are directly transferable to us even though
we are not doing biomedicine:
- **The method families for link prediction are named and mature:** graph
  neural networks (GCN, GraphSAGE, GAT), random-walk embeddings (Node2Vec),
  translational embeddings (TransE, RotatE), and metapath/rule-mining
  (AnyBURL). "Who here could help with X, whom you have not met" is the same
  mathematical shape as edge prediction.
- **The honest limitations are stated:** knowledge graphs are "frequently
  incomplete and not kept up to date," which "gives rise to predictions
  that do not offer new insights," and healthcare demands *explainability* -
  methods like PaGE-Link "generate explanations as paths through the graph."
  For a community tool, the lesson is decisive: **a routing suggestion must
  show its path** ("A knows B who runs project C"), never an opaque score.

### Supply chains, infrastructure, and institutional memory
- A global automaker maps "millions of connections between parts, suppliers,
  and assembly steps" to assess disruption impact.
- BT Group's graph network-inventory system manages 20,000 cell sites,
  1,900 ethernet exchanges, 150,000 circuits, "over 50,000 product
  availability checks daily," and cut capacity-planning time 50% and human
  decision points 60%.
- NASA applied graph master-data management to mission data since the 1950s,
  letting engineers "find links between past missions, procedures, and
  incident reports in seconds instead of weeks"
  (all: Neo4j use-cases blog, 2025-03-20).

The NASA case is the quiet one worth dwelling on: the value was not
prediction or fraud, it was **institutional memory made queryable** -
surfacing a connection between a current situation and a decades-old
incident report. A Tribal-network graph that outlives the staff turnover of
any single committee is doing the same job.

### GraphRAG - graphs as the retrieval substrate for LLMs
The newest vertical, and the most relevant to our sibling tools. Microsoft
Research's GraphRAG builds an LLM-generated entity knowledge graph, clusters
it hierarchically into "semantic communities," and uses both at query time
to answer *global* sensemaking questions a vector search cannot
(https://www.microsoft.com/en-us/research/blog/graphrag-unlocking-llm-discovery-on-narrative-private-data/,
2024-02-13). On a real conflict-news dataset (VIINA), baseline vector RAG
answered "What has Novorossiya done?" with "the text does not provide
specific information," while GraphRAG connected the dots across documents.

**Verification note - read this carefully.** The workflow's adversarial
verifiers *refuted* the strong claim that "GraphRAG substantially improves
QA performance over vector RAG" as stated (0-3 against). The refutation is
about *overgeneralization*, not fraud: Microsoft's own results are on
specific metrics (comprehensiveness, diversity, "human enfranchisement") and
specific global-question workloads, and do not license a blanket "GraphRAG
is better than RAG" claim. The nuance is the finding: **graph retrieval wins
specifically for multi-hop, connect-the-dots, whole-corpus questions**, and
that is exactly the question shape a resource-navigation tool asks.

The Neo4j KYC agent adds a design caveat we must inherit: free-form
text-to-Cypher "is not bulletproof ... relies heavily on the model to
produce valid and correct Cypher," so they wrap **pre-defined parameterized
queries as tools** rather than letting the LLM freestyle. And notably, they
run natural-language-to-query translation on a **local fine-tuned Gemma3-4B
via Ollama** - graph-query generation does not require a cloud model, which
matters for our local-first, sovereignty-constrained posture.

### Epidemiology and physical infrastructure (supplemental)
Two more verticals, added because they are named *operational* deployments
and because both are close to ATNI's climate-resilience domain.

**Contact tracing as a graph, in production.** Hainan Province, China ran an
operational COVID-19 contact-tracing deployment on Neo4j (Jan-Feb 2020),
fusing health, police, hospital, and telecom data: **persons, vehicles, and
public places as nodes; co-presence at the same place/time as edges.** It
traced 10,871 contacts from 61,439 analysis objects, surfaced 378
highest-risk individuals and 6 high-risk communities, and caught a case
traditional methods missed (https://pmc.ncbi.nlm.nih.gov/articles/PMC7837510/,
JMIR 2021). Geneva's health office similarly modeled transmission *chains* in
Neo4j to determine infection direction from exposure dates and rank chains for
resource allocation
(https://neo4j.com/blog/extensive-representation-of-contact-tracing-with-neo4j-in-the-canton-of-geneva/).
The transferable idea is the co-presence edge and the temporal direction of
relationships - both relevant if a community graph ever models shared events
or convenings over time.

**Energy grids as graphs, at TSO scale.** State Grid Corporation of China
(the world's largest utility) deployed TigerGraph as a "faster-than-real
-time" Energy Management System, running State Estimation, Power Flow, and
Contingency Analysis on a real provincial bus system in "a little above 1
second" combined - under the 5-second SCADA cycle - with the graph model
eliminating 25-35% of relational-join overhead
(https://www.tigergraph.com/stategrid/). A grid digital-twin study used
breadth-first graph traversal to simulate cascading outage propagation and
found three lines whose disconnection cut a modeled outage to 14% of its
scope (Capgemini, 2019). The transferable idea is **cascading/impact analysis
as graph traversal** - "if this node fails, what downstream is affected" - the
same primitive as "if this coordinator leaves, which collaborations break."

### Care and social-services referral networks - our closest analog
Unite Us runs closed-loop social-care referrals: a provider "gains digital
consent and electronically refers [a client] to multiple community
partners" simultaneously, with multiple intake channels (in person, phone,
online self-service form), and positions network analytics to "identify
service gaps and at-risk populations"
(https://uniteus.com/how-it-works/). This is the deployed, at-scale version
of Community Navigator's need-to-solution routing.

**Two honest caveats, both verified.** First, the "how it works" page
publishes *zero* quantitative statistics - no partner counts, no completion
rates - so any scale claim must be sourced elsewhere (Section 3 supplement
handles referral-completion evidence). Second, the verifiers *refuted* the
tidy claim that the platform "closes the loop so the originating provider
learns whether the need was actually met" (0-3): the page says outcomes are
"tracked," which is not the same as the loop reliably closing. This is the
single most important cautionary flag in the report for our routing feature,
and Section 3's supplement returns to why closed-loop referral is harder
than it sounds.

---

## 2. Graphs in three dimensions and anchored to geography

This section answers the "does 3D actually help, and how do you tie a graph
to real places" questions. The evidence is genuinely mixed on 3D and
genuinely encouraging on geo-anchoring.

### The geo-anchored layout: the closest thing to our design problem
**GeoGraphViz** (Transactions in GIS, 2023,
https://onlinelibrary.wiley.com/doi/10.1111/tgis.13053; preprint
arxiv.org/pdf/2304.09864) is the single most relevant academic artifact
found. It is a browser-based ThreeJS/WebGL 3D force-directed graph that adds
a **"geolocational force"** with a tunable parameter **K**: at K=0 the layout
is pure semantic clustering (force-directed as usual); as K grows, each node
is pulled toward its actual map location. They quantified the tradeoff (mean
locational offset 0.554 at K=0, 0.119 at K=5, ~0.001 at K=10,000, while
edge-length variation rises) and **recommend K between 3 and 20** as the
usable middle - close enough to geography to read as a map, loose enough
that the graph structure stays legible.

This is, more or less, a published version of the exact knob Community
Navigator will want between "constellation" (affinity-clustered) and "map"
(geography-anchored) views. It was demonstrated on a real humanitarian case
- 41 infectious-disease experts curated by Direct Relief for COVID-19 aid
distribution - and is deployed against KnowWhereGraph. Its stated
performance ceiling is a caution for us: rendering "within about 7 seconds"
under ~800 nodes, "lag growing beyond ~1,600 nodes." Their ThreeJS approach
was not instanced; ours is (ADR-004), which is why our own spike already
beats this. But it confirms the design and the failure mode.

### KnowWhereGraph: the reference GIS+graph hybrid
KnowWhereGraph (https://arxiv.org/html/2502.13874v1, 2025-02-19) is one of
the largest public geospatial knowledge graphs: **29 billion+ RDF triples**
(~20B asserted, ~8B inferred), integrating 30+ datasets - natural hazards,
climate, soil, demographics, public health, humanitarian relief. Two
architectural choices are directly instructive:

- **Everything is anchored to geography through Google's S2 hierarchical
  spherical grid**, cell levels 8 (~1300 km²) to 13 (~1.3 km²), each cell a
  64-bit S2CellID, linked by topological relations (within, contains,
  overlaps). This is how you make "resources near this place" a graph
  traversal instead of a geometry computation. (deck.gl offers the same
  pattern with H3 hexagons via H3HexagonLayer - instanced GPU rendering,
  multi-resolution aggregation - which is a **privacy lever**: you can show
  resource density per hex cell without exposing a household's point
  location, a direct fit for TSDF-tiered data.)
- **It is built entirely on open W3C standards** - RDF, GeoSPARQL, SOSA/SSN,
  OWL-Time, PROV-O, SKOS - and queried with (Geo)SPARQL. A standards-based
  hybrid, not a proprietary stack. PROV-O in particular is the W3C
  provenance ontology; our IEEE-2890 custody envelope is conceptually
  adjacent and could be expressed in it if we ever needed interop.

Crucially, **KnowWhereGraph does not make users write SPARQL.** It ships a
faceted "Knowledge Explorer" with a map interface, ArcGIS/QGIS plugins, and
the GeoGraphVis tool - and can deliver geo-enriched "area briefings ...
within seconds." This is the progressive-disclosure lesson again: the graph
is the engine; the interface is a map and a briefing.

### Does 3D actually help? The honest, mixed answer
This is where the director's skepticism is vindicated by the literature.
Collected evidence, strongest first:

- **The classic pro-3D result** (Ware and Franck, cited in a 2025 immersive
  survey, https://arxiv.org/pdf/2501.08500): head-coupled stereoscopic 3D
  lets users "perceive three times more complex graphs than 2D." But the
  same survey immediately bounds it: "S3D is only more efficient for data of
  higher complexity ... 2D is faster in simpler cases," and "VR enhances
  structural interpretation ... yet spatial memory tasks favour 2D."
- **A VR multilayer-network study** (IEEE VIS 2023,
  https://arxiv.org/pdf/2307.10674, 22 participants): "no clear overall
  winner"; higher dimensions won 3 of 6 tasks; for *large* 7-layer networks,
  flat 2D beat 3D on the two highest-cognitive-load tasks because "adding a
  third dimension ... increases the cognitive load [and] does not help."
  User preference inverted with scale: 2.5D preferred for small networks,
  full 3D preferred (45%) for large ones.
- **A mixed-reality cyber-defense study** (Frontiers in Big Data, 2023,
  https://www.frontiersin.org/journals/big-data/articles/10.3389/fdata.2023.1042783/full):
  the 3D group had better situational awareness and identified more correct
  adversary hosts, *and* lower communication demand in teams - but performed
  **worse** on a forced-choice topology-recognition test (possibly biased by
  the test being a 2D image). Note this was HoloLens head-mounted MR, not
  screen 3D.
- **The reproducibility gap:** across 138 immersive-viz publications, "only
  one investigation focuses on scalability ... a large majority of
  applications are not available or not maintained anymore." Much of the
  field's tooling is abandonware.

**Synthesis for us:** 3D earns its place for *engagement, spatial/structural
awareness, and separating clusters that collide in 2D* - which is exactly
the general-assembly reveal ("see the whole community, see the clusters,
see yourself in it"). 3D *costs* on precise structural reading and spatial
memory - which is exactly the committee-meeting analytical work ("trace this
specific pathway"). The design implication is not "2D or 3D"; it is **offer
both, and default to the right one per task**: 3D constellation for the
reveal and free exploration, a 2D/2.5D or geo-anchored projection (and a
parallel list/table, which we already owe for accessibility) for precise
tracing and routing readouts. The GeoGraphViz K-knob is one clean mechanism
for moving along that axis.

### The GPU rendering benchmark
cosmos.gl (OpenJS Foundation, formerly Cosmograph,
https://github.com/cosmosgl/cosmos) runs the *entire* force simulation and
drawing in WebGL shaders - "all computations and drawing occur on the GPU"
- and simulates "hundreds of thousands of points and links" in real time,
far past the ~500-node ceiling of CPU force layouts. **It is 2D only** (no
documented 3D mode), and as of v3 its rendering moved from regl to luma.gl
on WebGL2. It is the performance bar for browser graphs and a candidate for
our 2D projection view, but it does not replace our Three.js instanced 3D
layer. The verifiers could not fully confirm the scale numbers (source
errors, not refutation), so treat "hundreds of thousands" as the project's
own claim.

---

## 3. What a graph of many Peoples could illuminate

This is the heart of the director's question, and it splits into three:
what the graph reveals, how you route needs to solutions inside it, and how
you do all of that under Indigenous data sovereignty.

### What becomes visible
Community asset mapping is a mature practitioner tradition, and the network
turn on it is explicit: strength-based asset mapping combined with social
network analysis surfaces "who collaborates, who is isolated, where trust
lies" (Visible Network Labs, 2025). Translated to a graph of many Peoples,
the illuminations are:

- **Bridges and brokers:** people or organizations whose removal would
  disconnect sub-communities (betweenness centrality). In a Tribal climate
  network these are the coordination linchpins - and the succession risks.
- **Affinity pools:** community detection (the same hierarchical clustering
  GraphRAG uses for "semantic communities") reveals clusters that share a
  species, a watershed, a hazard, or a specialty - latent working groups
  nobody named.
- **Structural holes and gaps:** the *absence* of an edge is a finding. Two
  Tribes working the same drought problem with no connecting path between
  them is a routing opportunity; a need with no reachable resource is a gap
  the network should escalate. Unite Us frames this exactly as "identify
  service gaps."
- **Isolates:** members connected to no one are the equity signal - the
  people a network is failing to include. A strength-based tool should
  surface them for outreach, gently.
- **Reach and paths:** "how many hops from this need to a plausible
  resource, and through whom" - the routing readout itself.

### Need-to-solution routing: the algorithms
The mathematics is well established and modest in scale for us (low
thousands of nodes). Named approaches from the corpus and the supplement:

- **Constrained/variable-length pathfinding.** The literal "who can help
  with X" query is a shortest or constrained path from a need node to
  capability-bearing actor nodes. Cypher/GQL variable-length patterns
  (`-[:...*1..6]->`) express it directly; the KYC ring-detection demo is the
  same primitive turned to a different purpose.
- **Bipartite matching.** Needs on one side, offers on the other, edges are
  compatibility; classic assignment/matching algorithms optimize who gets
  matched to what. This is the shape of volunteer-, mentor-, and mutual-aid
  matching (supplement, Section 3-bis below).
- **Link prediction** for "you have not met them, but you probably should"
  (the biomedical method families: embeddings, metapaths, GNNs). Powerful,
  but remember the biomedical caution: incomplete graphs yield uninsightful
  predictions, and **every suggestion must show its path** for trust.
- **Community detection** for affinity pools (Louvain/Leiden-style
  modularity, or the hierarchical clustering GraphRAG uses).

The design rule that falls out of the biomedical explainability finding and
the local-first LLM finding together: **routing is a graph query first and
an LLM convenience second.** Compute paths and matches in the core
(deterministic, explainable, offline); optionally use a *local* small model
only to translate a plain-language question into a parameterized query, and
never to invent the answer.

### 3-bis. Resource matching in practice (supplemental research)
This is the most operationally sobering - and useful - part of the report,
because it is the field that has actually tried to do what our hero workflow
promises, at scale, for a decade. Three findings reshape how we should scope
routing.

**Finding 1: the field runs on shared *taxonomies* and shared *data
structure*, not on matching optimization.** North American human services
are indexed by the AIRS/211 taxonomy - "more than 9,200 terms" in 10
categories, nesting up to 6 levels, mutually exclusive, but proprietary and
license-gated
(https://public.dhe.ibm.com/software/solutions/curam/6.0.5.0/en/html/BusinessAnalysts/CuramProviderManagementGuide/c_CPM_TaxonomyAIRSAbout.html).
The open alternative is **Open Eligibility** (Aunt Bertha/findhelp, 2012, CC
BY-SA), deliberately small and **two-faceted: Human Services (what is
offered) x Human Situations (who you are - "veterans," "seniors")** - and it
has been adopted into HL7 FHIR as a code system, i.e., it is becoming an
interoperability lingua franca
(https://github.com/openreferral/openeligibility ;
https://terminology.hl7.org/CodeSystem-OpenEligibilityTaxonomy.html).
findhelp's own data model is **provider -> program -> service -> service tag**,
with the taxonomy classifying tags and Situations driving eligibility
filtering (https://company.findhelp.com/the-open-eligibility-project/). And
critically, **Open Referral's HSDS standard is taxonomy-agnostic by design** -
it standardizes the *directory structure* and lets you "overlay a taxonomy of
[your] choosing" (https://docs.openreferral.org/en/latest/hsds/overview.html).

The direct lesson for D-023 (our intake form's shared offer/need taxonomy):
we are reinventing a solved problem, and we should not invent our vocabulary
from scratch. **The offer/need taxonomy should start from Open Eligibility's
two-facet model (service x situation), extended with community-defined terms
via our group templates** - which is exactly the "community vocabularies are
first-class" commitment from the sovereignty section. The provider ->
program -> service -> tag shape is also a ready-made node model.

**Finding 2: closed-loop referral works only when funding and CBO capacity
exist - the platform is not the bottleneck.** This is the empirical
puncturing of the Unite Us marketing gloss, and it is well-evidenced. The
strongest independent evaluation, of NCCARE360 (the Unite Us-powered NC
statewide network, N=4,080 Durham cases), found resolution rates swung with
*money*, not software: **88% resolution when COVID support funds were
available versus 30% when they were not**; successful-connection 65% vs 38%
(https://ncmedicaljournal.com/article/94877, 2024-03-18). And the study had
to *invent a separate "successful connection" metric* because cases are
marked "closed" regardless of whether the service was actually received -
which is the precise mechanism behind the verifier's refutation of Unite
Us's "closes the loop" claim in Section 1. A findhelp-based study found
**56% of patients versus 93% of staff** agreed the tool reduced effort, with
optimistic bias because dissatisfied users declined interviews; a related NC
intervention saw only **32.7% of patients report service initiation within 4
weeks** (https://pmc.ncbi.nlm.nih.gov/articles/PMC12052525/, 2025). The
foundational SIREN review of 9 platforms plus 39 adopter interviews found the
platforms functionally near-identical and the real barriers **organizational**
- engaging partners, change management, compliance - not technical
(https://www.healthaffairs.org/doi/10.1377/hlthaff.2019.01588, 2020). A 2025
AMA council report concludes closed-loop effectiveness evidence "remains
limited." No independent peer-reviewed Unite Us completion study surfaced;
its headline "93% acceptance" figures are vendor-reported.

The lesson for our routing promise is now sharp and must be carried into
plan v3: **surfacing a plausible path is a v0.1 deliverable; confirming a
need was met is a research-grade problem gated by resources outside the
software.** Promise the former honestly; do not imply the latter. And design
the "closed" state carefully - the human-services field's own mistake was
conflating "referral sent" with "need met."

**Finding 3: real algorithmic matching succeeds only where constraints are
hard and formalizable, or where a light re-ranking beats popularity.** The
canonical success is **kidney exchange** - integer programming over a
directed compatibility graph, packing disjoint cycles plus altruist chains,
NP-hard beyond cycle-length 2, fielded nationally by UNOS
(https://www.pnas.org/doi/10.1073/pnas.1421853112, 2015). At the community
end, **VolunteerMatch** found that popularity-ranked recommendation starved
small orgs; a capacity-aware re-ranking (down-weighting opportunities that
already have signups) **raised the share of orgs getting >=1 volunteer by
8-9% with negligible total loss** - "VolunteerMatch is not a search engine;
it is a matching platform"
(https://insights.som.yale.edu/insights/better-algorithm-can-bring-volunteers-to-more-organizations,
2023). Mentor matching in production is weighted-criteria scoring or
reciprocal recommenders **with human approval** (Mentorly; a Codementor
learning-to-rank study). And time banks (hOurworld, TimeRepublik) **do not
algorithmically match at all** - they run an offer/request marketplace on a
1-hour = 1-credit ledger with reputation for discovery
(https://hourworld.org/_TimeAndTalents.htm).

The synthesis: **for a low-thousands community graph, "matching" should mean
taxonomy-indexed search plus explainable pathfinding plus a light
diversity/capacity-aware re-rank - not an optimization engine.** Reserve
formal optimization for a future case with hard constraints. Keep a human in
the loop on every suggested connection (the mentor-matching and the
sovereignty "refusal" lessons converge here). This *lowers* our engineering
risk while keeping the distinctive claim intact.

### Indigenous data sovereignty in graph form - the hardest and most important part
This is not an add-on to the graph; for a graph of many Peoples it is the
architecture. The corpus is unusually strong here.

**The sovereignty principle.** Local Contexts defines Indigenous Data
Sovereignty as "a legitimate right of Indigenous Peoples to control the
access, the collection, ownership, application and governance of their own
data or knowledge" (https://localcontexts.org/indigenous-data-sovereignty/).
This is the CARE-principles / OCAP posture our TSDF framework already
encodes.

**The label mechanism - and its crucial limit.** Traditional Knowledge (TK)
and Biocultural (BC) Labels are the concrete metadata mechanism. Communities
author and *retain governance over* their customized labels via the Local
Contexts Hub; Mukurtu CMS attaches "up to four TK Labels" per Digital
Heritage Item in the metadata sidebar
(https://mukurtu.org/support/traditional-knowledge-labels-faq/). But the
limit is decisive for our design: **TK Labels are explicitly not legally
binding and Local Contexts "is not an authorizing or policing entity."**
The labels are *advisory provenance and protocol metadata*; enforcement is
architectural and rests with the implementing system. In other words: the
label tells you the protocol; **your permission engine has to be the thing
that enforces it.** That is precisely the division of labor between a
TK-Label-style annotation and our cn-perm/TSDF tier enforcement. We are, in
effect, building the enforcement layer the labels assume exists.

**A live integration path exists.** The Local Contexts Hub API (v2, default
since 2025-02-10, https://localcontexts.org/support/api-guide/v2/) serves
machine-readable Labels and Notices by Project ID over authenticated GET.
Two properties map straight onto TSDF thinking: **"Private projects are
inaccessible via API - only Public and Discoverable projects are
available"** (a sovereignty-aligned visibility boundary that mirrors our
tiers), and label/notice edits "propagate to API consumers in real time" (so
the Hub is the source of truth, not a cache). Display-integrity is a
contract: Notice text, icons, and titles "cannot be changed." If Community
Navigator ever attaches TK/BC Labels to nodes, this is the how, and the
display constraints are non-negotiable requirements on the 3D layer and
detail panel.

**The deepest caution - the ontology itself can colonize.** Two 2025-2026
scholarly sources argue that linked-data interoperability is not neutral. A
2026 study traced the term "Two-Spirit" across LCSH, Wikidata, and the
community-built Homosaurus and found LCSH "isolates and removes context,"
Wikidata links "favour Western categorical frameworks," while the
community-built vocabulary "maintains richer, experience-based relational
structures" (https://link.springer.com/article/10.1007/s10502-025-09526-5
and the 2026-01-14 companion). The recommendation is not to reject graphs
but to treat interoperability as **"predictable, protocol-aware access
governed by Indigenous jurisdiction"** where "semantic mappings are
negotiated, scoped, or refused." For us this lands as three concrete design
commitments:
1. **Community-defined vocabularies are first-class** - our group templates
   (schema-as-data) are exactly the mechanism to let each community define
   its own kinds, attributes, and relationship types rather than inheriting
   a settler ontology. This is a design strength we already have; name it.
2. **"Refusal" is a supported operation, not an error.** A community can
   decline a mapping, a link, or a tier promotion. The permission model and
   the export model must treat refusal as legitimate and expressible.
3. **Place and kinship are relationship types, not just attributes.**
   Land-based ontologies want geography and relation to be structural. Our
   graph is well suited to this if we resist flattening place into a text
   field - which is the same point GeoGraphViz makes technically.

---

## 4. Code and technical structures: local-first graph engines

The director named Kuzu, CozoDB, Oxigraph, and sqlite-based options. Here is
the state of each as of this research, with the adoption risks stated
plainly, because one of them just changed materially.

### The Kuzu situation - a real risk to note
Kuzu was the strongest architectural match: an embedded, in-process,
serverless property-graph engine, "the DuckDB of graphs" - columnar storage,
vectorized/factorized execution, worst-case-optimal joins, Cypher, single
-file storage (`data.kz`), official Rust bindings, native full-text and
vector indices, and **kuzu-wasm to run the whole DB in the browser**
(https://github.com/kuzudb/kuzu). Benchmarks were striking: ~18x faster
ingest than Neo4j and 180x+ on multi-hop path queries
(https://thedataquarry.com/blog/embedded-db-2/); a production user (Bauplan)
reported "20x faster" DAG planning, 500+ Cypher statements in ~1.5s.

**But: the Kuzu GitHub repo was archived on 2025-10-10 and is now read-only;
a European Commission filing ~4 months later confirmed Apple acqui-hired the
team** (https://gdotv.com/blog/kuzu-legacy-embedded-graph-database-landscape/).
Frozen releases (latest v0.11.3, MIT-licensed) "will continue to be usable
without modifications," and a community fork, **LadybugDB**, is actively
developing (it has added multi-label nodes, no-copy Arrow/DuckDB/Parquet
integration, and `CREATE GRAPH`/`USE` subgraphs). So Kuzu is not dead, but
adopting the original is adopting an archived project, and the fork is young.
This is a genuine "measure twice" input for any engine decision.

### The others
- **CozoDB** (https://github.com/cozodb/cozo): Rust, embedded (SQLite-style),
  **Datalog** query language, graph-algorithms built in (community
  detection, pathfinding **in-engine**), five swappable backends (in-memory,
  SQLite, RocksDB, Sled, TiKV), **compiles to WASM to run a full instance in
  the browser at near-native speed**, and since v0.6 supports HNSW vector
  indices *inside recursive Datalog* plus (v0.7) MinHash-LSH near-duplicate
  detection and full-text search. That vector+graph+dedup-in-one-engine combo
  is remarkably aligned with our needs (routing + GraphRAG-style + ingest
  dedup). **Risk: it is pre-1.0, "no syntax/API/storage stability," and the
  latest release is v0.7.6 dated 2023-12-11 - roughly 2.5 years stale as of
  this report.** Powerful, but release activity looks stalled.
- **Oxigraph** (https://github.com/oxigraph/oxigraph): Rust, **RDF/SPARQL**
  (not Cypher), shipped as a crate, a WASM npm module, a Python package, and
  a server. The right choice *if and only if* we want semantic-web interop -
  e.g., consuming Local Contexts Labels as linked data, or aligning with
  KnowWhereGraph-style GeoSPARQL. **Self-described as immature: "SPARQL query
  evaluation has not been optimized yet," pre-1.0 (v0.5.9, 2026-06-18),
  RocksDB-backed.** Standards-conformant but slow today.

### The relevant read for Community Navigator
We are not, in fact, shopping for one of these to be our primary store - we
already made the architecturally load-bearing decision: **the Rust/WASM core
(cn-model/cn-store/cn-graph) is our engine, event-sourced, permission-first,
already measured (255ms fold / 133ms projection at 5k+10k).** The value of
this section is threefold: (1) it validates the local-first, embedded,
compiles-to-WASM, graph-in-the-browser architecture as a real and populated
category, not an exotic bet; (2) it says which *concepts* to borrow -
columnar/CSR adjacency, factorized multi-hop joins, in-engine graph
algorithms, and vector-index-alongside-graph for hybrid queries; and (3) it
flags that if we ever want a drop-in Cypher/Datalog/SPARQL surface (for
power users, or for interop), CozoDB's Datalog-with-vectors or Oxigraph's
SPARQL are the local-first options, each with a maturity caveat. **Do not
adopt Kuzu-original now; if a Cypher engine is ever wanted, evaluate
LadybugDB's trajectory first.**

### The query-language standard: GQL
Confirmed by supplemental research. **ISO/IEC 39075:2024 (GQL) was published
April 2024** (sources split on the 11th vs 12th; treat as "April 2024"), the
**first new ISO database-query standard since SQL in 1987**. It "fuses ideas
from openCypher, GSQL, and PGQL with SQL" and keeps openCypher's visual
"ASCII-art" pattern syntax; openCypher's stated purpose is now to "pave the
road to GQL"
(https://en.wikipedia.org/wiki/Graph_Query_Language ;
https://neo4j.com/blog/cypher-and-gql/gql-database-language-standard/).
Adoption as of 2025-2026 is **partial and incremental**: Neo4j's Cypher "now
accommodates most mandatory GQL features and a substantial portion of its
optional ones" per an official conformance appendix
(https://neo4j.com/docs/cypher-manual/current/appendix/gql-conformance/), but
**no engine is yet fully GQL-conformant.**

The strategic point: **GQL is the standards-backed, Cypher-lineage target,
and engines are converging on it.** If cn-graph ever exposes a textual query
language (for power users or interop), an openCypher-compatible subset
tracking GQL is the future-proof choice, not a proprietary dialect. Our
internal API need not commit now, but our naming and traversal semantics
should not fight GQL/Cypher conventions - variable-length path patterns,
labeled property graph, MATCH-style pattern semantics.

---

## 5. Real-world usability: graphs for non-technical audiences

The director asked what works when non-technical community audiences use
graph tools. The evidence here is the most important corrective in the whole
report, because it is largely a warning, and it comes from studies of
exactly our audience.

### The uncomfortable baseline: people cannot read network diagrams
This is not hedging; it is measured. Börner et al. (PNAS,
https://www.pnas.org/doi/10.1073/pnas.1807180116) assessed 273 science-museum
visitors and found "significant limitations in naming and interpreting
visualizations and particular difficulties when reading network layouts";
the broader literature is summarized bluntly: "people have difficulties
reading most visualization types but especially, networks." Node-link
diagrams degrade sharply for lay readers as size and edge density grow (the
"hairball problem"; Ren et al., NetSci), and matrix or filtered
representations often beat them on lay-reader tasks.

And from the most on-the-nose source: **Net-Map**, a participatory
network-mapping method used in a Ghana fisheries community, found that "most
interview partners were unable to read the networks because of illiteracy
but also a lack of understanding [of] somewhat abstract network concepts"
(Field Methods 2010,
https://www.researchgate.net/publication/249629762). The organizational
-learning benefits that appeared with institutional stakeholders "did not
replicate at the local level." A tool for Tribal and community audiences
cannot assume the constellation is self-evidently readable. It is not.

### What actually works: co-construction, guided tours, progressive disclosure
The same literature is generous about *what to do instead*, and every remedy
is something Community Navigator can build.

- **The learning happens during construction, not from the finished
  artifact.** Net-Map's core finding: "a lot of the learning process happens
  during data collection and not through documents produced after analysis."
  The map is low-tech on purpose (paper actor-cards, felt pens, stackable
  "influence towers"), usable "from rural community members with little
  formal education to policy makers." **Implication for us:** the intake form
  and the facilitated assembly *are* the literacy on-ramp. People understand
  the graph because they helped build it and they find themselves in it -
  which is precisely the win condition the director already articulated. And
  a wry, load-bearing detail: when Net-Map scaled to groups, facilitators
  **flattened the 3D influence-towers into a 2D wall layout** so everyone
  could see - an early, concrete instance of 3D trading legibility for
  expressiveness in a group setting. Our assembly reveal is a group setting.
- **Guided data tours dramatically help novices.** NetworkNarratives
  (ACM CHI 2023, https://arxiv.org/pdf/2303.06456) is a semi-automatic
  slideshow of network "facts" with annotations, where a user can break out
  and explore at any slide. In a novice study (14 data-science students with
  no network experience), tours beat free-form exploration for *learning*
  (4.86 vs 3.86) and for *teaching network analysis* (4.71 vs 2.86), and
  novices voluntarily spent **longer** with tours (8:48 vs 5:00 minutes),
  browsing ~31 fact-slides at ~17s each. This is direct, quantitative
  evidence for our Stories feature - and it reframes Stories from "nice
  narrative extra" to **the primary comprehension scaffold for a graph-novice
  audience.** The director's decision to build in-app story authoring in v0.1
  (D-037) is, on this evidence, one of the highest-leverage calls in the
  plan.
- **Progressive disclosure is the interface pattern, not a preference.**
  Every successful deployment in this report hides the graph behind a
  simpler surface: KnowWhereGraph's faceted map explorer, ICIJ's search box
  (2.6M users), Unite Us's intake form. The graph is the last thing you show,
  not the first. Start with a person's own record or a plain-language
  question; expand on demand.
- **Data visualization literacy is teachable and worth teaching.** Börner's
  framework (DVL-FW) exists precisely to define, teach, and assess it, and
  the payoff is framed as "better communication, collaboration, self-efficacy
  and decision making." A short in-product "how to read this" and consistent
  visual encoding is not hand-holding; it is the documented remedy.

### The facilitation pattern that fits ATNI directly
Participatory network mapping has been used for climate/just-transition
coordination with real communities (Dingle Peninsula low-carbon transition,
Tandfonline 2021) - a near-identical use case to a Tribal climate-resilience
network. The consistent finding across Net-Map and its successors: the value
is collective sensemaking mediated by a facilitator, not a tool handed to
individuals. **This validates the pilot's facilitator-run design end to
end** (D-022/D-025): the facilitator is not a scope-limiting compromise, it
is the evidence-based delivery model.

---

## 6. What this means for Community Navigator (and the sibling tools)

Pulling the evidence into decisions and candidate plan-v3 inputs. These are
recommendations for the human sitting, not autonomous commitments.

### Six design commitments the evidence supports
1. **The graph is the engine; the interface is a map, a question, a
   briefing, and a tour.** Lead every surface with a person's own record or a
   plain-language question. The 3D constellation is the reveal and the
   free-exploration space, never the first or only thing shown. (Backed by
   every deployment in Sections 1-2 and all of Section 5.)
2. **Stories are the comprehension layer, not a garnish.** Guided tours are
   the measured remedy for the fact that our audience largely cannot read
   network diagrams cold. Treat S3-C authoring as core, and seed the first
   tours from intake-form story material and from the assembly script
   itself. (NetworkNarratives; Net-Map; D-037.)
3. **Every routing result must show its path.** No opaque match scores.
   "This need connects to that resource through B and C" - the explainability
   lesson from biomedical KGs, and the trust foundation for the hero
   workflow. (PaGE-Link; the whole of Section 3.)
4. **Offer 3D and a flat/geo/list projection, and default per task.** 3D for
   the assembly reveal and cluster-spotting; 2D/2.5D/geo-anchored and the
   parallel list for tracing and routing readouts. The GeoGraphViz K-knob is
   a clean mechanism for the constellation-to-map slider. (Sections 2's mixed
   3D evidence; our own accessibility parallel-DOM debt becomes a feature
   here.)
5. **Sovereignty is enforced by the permission engine; labels are advisory
   metadata on top.** TK/BC Labels tell the protocol; cn-perm/TSDF enforces
   it. Support *refusal* (of a mapping, link, or tier change) as a
   first-class operation. Keep community-defined vocabularies first-class via
   group templates - that is our answer to the "ontology can colonize"
   critique. (Local Contexts; Mukurtu; the Archival Science and Two-Spirit
   studies; our existing TSDF work.)
6. **Geography is structural, not a text field.** Anchor places as nodes with
   topological relations; consider H3/S2 cells for privacy-preserving spatial
   aggregation (resource density per hex, not per household) - a natural fit
   for tiered sovereign data and a direct bridge to GeoBase. (KnowWhereGraph;
   deck.gl H3; land-based-ontology critique.)

### Where the sibling tools plug in (integration hypotheses for plan v3)
The graph is the connective tissue these tools have been missing. Each maps
to a node/edge contribution:

- **cap-assessor** extracts *needs and capacity* from Tribal climate
  adaptation plans. That is a node-and-edge feed directly into the
  need-to-solution graph: a Tribe's plan yields need-nodes and
  capability-nodes with provenance. This is arguably the richest structured
  source of the "offers and needs" taxonomy the intake form is trying to
  collect by hand - the two should share one capability vocabulary (D-023).
- **TCR-policy-scanner** produces per-Tribe *policy/program intelligence*.
  In graph terms: funder/program nodes and eligibility edges. This makes a
  new routing target reachable - "who here is eligible for / working on
  program X" - and connects community needs to federal resources, not just
  to each other.
- **GeoBase** is the *sovereign geospatial spine* with TSDF tiering. It is
  the natural home for the geography-as-structure commitment (#6): Community
  Navigator's place-nodes could resolve against GeoBase layers, and the two
  share the TSDF enforcement model. The H3/S2 privacy-aggregation pattern is
  a shared technique.
- **engagement-database** already holds contacts/orgs/engagements in SQLite.
  It is the most immediate real-data feeder (subject to the pilot's consent
  gate and PII rules) - the existing engagement graph is a pre-built edge
  set. The migration-recipe discipline we are writing for CPF-RCN (Session E)
  is the same discipline that would govern an engagement-database feed.
- **TSDF / Local Contexts** is the governance layer across all of them. The
  Hub API is a live integration path if any tool wants community-authored
  labels as linked data; the enforcement stays ours.

The through-line: **one capability taxonomy, one provenance/tier envelope,
one graph.** If cap-assessor, the intake form, TCR-policy-scanner, and
engagement-database all emit into a shared node/edge model with shared
capability terms and shared TSDF tiers, Community Navigator becomes the
surface that reasons across all of them - and the "graph of many Peoples"
stops being a metaphor. And that shared capability taxonomy should not be
invented from scratch: **start from Open Eligibility's two-facet model
(service x situation), adopted into HL7 FHIR, and extend it per community via
group templates** (Section 3-bis). Using a taxonomy that already speaks FHIR
is also the cleanest future bridge to health-and-social-care data systems the
Tribes may already use.

### The three cautions to carry into plan v3
1. **Lay-audience graph literacy is a real barrier, measured, in exactly our
   population.** Budget for tours, "how to read this," progressive
   disclosure, and facilitator scaffolding as core scope, not polish.
2. **Closed-loop routing is harder than the demo, and the bottleneck is not
   software.** The NCCARE360 evidence is unambiguous: resolution rates swung
   88% -> 30% with *funding*, and "closed" often meant "referral sent," not
   "need met" (Section 3-bis). Surfacing an explainable path is the honest
   v0.1 deliverable; confirming a need was met is resource-gated and belongs
   to a later, humbler promise. Design the "closed" state so it never
   conflates the two.
3. **The engine landscape shifts under you.** Kuzu archived mid-2025. Our own
   core insulates us, but any decision to lean on an external graph engine
   inherits real project-viability risk - measure the fork trajectories.

---

## Sources (verified during this run)

Primary and load-bearing, grouped by section. "V" = passed adversarial
verification (2-3 of 3 verifiers confirmed); "R" = a specific overclaim was
refuted (noted inline); "S" = self-report / vendor page, used for pattern
not magnitude; "U" = extracted but verifier votes errored (treat as
single-source).

- ICIJ / Neo4j offshore leaks - https://neo4j.com/customer-stories/icij/ (V; one figure conflated, noted)
- Neo4j use-cases (Zurich, BT, NASA, automaker) - https://neo4j.com/blog/graph-database/graph-database-use-cases/ (S)
- Drug-repurposing KG review, Briefings in Bioinformatics 2024 - https://pmc.ncbi.nlm.nih.gov/articles/PMC11426166/ (V)
- Microsoft GraphRAG - https://www.microsoft.com/en-us/research/blog/graphrag-unlocking-llm-discovery-on-narrative-private-data/ (V pipeline; R on the "beats RAG" overclaim)
- Neo4j GraphRAG KYC agent - https://neo4j.com/blog/developer/graphrag-in-action-know-your-customer/ (S)
- Unite Us - https://uniteus.com/how-it-works/ (V workflow; R on "closes the loop")
- GeoGraphViz, Transactions in GIS 2023 - https://onlinelibrary.wiley.com/doi/10.1111/tgis.13053 (V)
- KnowWhereGraph - https://arxiv.org/html/2502.13874v1 (V)
- deck.gl H3HexagonLayer - https://deck.gl/docs/api-reference/geo-layers/h3-hexagon-layer (U)
- VR multilayer 2D/2.5D/3D study, IEEE VIS 2023 - https://arxiv.org/pdf/2307.10674 (U; single-source)
- MR cyber SA study, Frontiers in Big Data 2023 - https://www.frontiersin.org/journals/big-data/articles/10.3389/fdata.2023.1042783/full (U; single-source)
- Immersive network viz survey 2025 - https://arxiv.org/pdf/2501.08500 (S/primary)
- cosmos.gl - https://github.com/cosmosgl/cosmos (U on scale numbers)
- Kuzu repo - https://github.com/kuzudb/kuzu ; landscape/archival - https://gdotv.com/blog/kuzu-legacy-embedded-graph-database-landscape/ ; benchmark - https://thedataquarry.com/blog/embedded-db-2/ (V archival fact)
- CozoDB - https://github.com/cozodb/cozo (V; staleness noted)
- Oxigraph - https://github.com/oxigraph/oxigraph (V; immaturity noted)
- Local Contexts IDSov - https://localcontexts.org/indigenous-data-sovereignty/ ; Hub API v2 - https://localcontexts.org/support/api-guide/v2/ (V)
- Mukurtu TK Labels FAQ - https://mukurtu.org/support/traditional-knowledge-labels-faq/ (V)
- Grounding the Semantic Web (Archival Science 2025) - https://link.springer.com/article/10.1007/s10502-025-09526-5 ; Two-Spirit KG study 2026 (V)
- Börner et al. DVL, PNAS - https://www.pnas.org/doi/10.1073/pnas.1807180116 (V)
- NetworkNarratives, CHI 2023 - https://arxiv.org/pdf/2303.06456 (V)
- Net-Map, Field Methods 2010 - https://www.researchgate.net/publication/249629762 (V)
- Dingle participatory mapping 2021 - https://www.tandfonline.com/doi/full/10.1080/13549839.2021.1936472 (S)
- Community asset mapping + SNA - Visible Network Labs 2025 (S)

Supplemental pass 1 (resource matching), single-source primary/peer-reviewed:
- AIRS/211 taxonomy - IBM Curam docs mirror (S/primary); 211taxonomy.org
- Open Eligibility - https://github.com/openreferral/openeligibility ; HL7 FHIR code system (V-structure)
- findhelp Open Eligibility Project - https://company.findhelp.com/the-open-eligibility-project/ (S)
- Open Referral HSDS - https://docs.openreferral.org/en/latest/hsds/overview.html (primary)
- NCCARE360 evaluation - https://ncmedicaljournal.com/article/94877 (peer-reviewed, 2024)
- findhelp/Resourceful CBPR study - https://pmc.ncbi.nlm.nih.gov/articles/PMC12052525/ (peer-reviewed, 2025)
- SIREN platform review - https://www.healthaffairs.org/doi/10.1377/hlthaff.2019.01588 (peer-reviewed, 2020)
- Kidney exchange IP - https://www.pnas.org/doi/10.1073/pnas.1421853112 (PNAS 2015)
- VolunteerMatch field experiment - https://insights.som.yale.edu/insights/better-algorithm-can-bring-volunteers-to-more-organizations (2023) + Management Science
- Time banks - https://hourworld.org/_TimeAndTalents.htm (primary)

Supplemental pass 2 (epidemiology / energy / GQL), primary + case-study:
- Hainan COVID contact tracing on Neo4j - https://pmc.ncbi.nlm.nih.gov/articles/PMC7837510/ (JMIR 2021, primary)
- Geneva transmission chains - https://neo4j.com/blog/extensive-representation-of-contact-tracing-with-neo4j-in-the-canton-of-geneva/ (case study)
- State Grid / TigerGraph EMS - https://www.tigergraph.com/stategrid/ (S/case study)
- Grid digital-twin cascading analysis - Capgemini/EnergyCentral 2019 (case study)
- GQL ISO/IEC 39075:2024 - https://en.wikipedia.org/wiki/Graph_Query_Language ; https://neo4j.com/blog/cypher-and-gql/gql-database-language-standard/ ; conformance: https://neo4j.com/docs/cypher-manual/current/appendix/gql-conformance/

---

## Appendix: research provenance

This report was produced by a deep-research workflow (run wf_311bfd8b-dae):
5 parallel search angles, 25 sources fetched and claim-extracted (~120
falsifiable claims), and 3-vote adversarial verification on the top claims.
The workflow's automatic synthesis step failed on a session usage limit;
this report was synthesized by the director from the full agent journal
(all 25 source extractions preserved), which is richer than the 14 claims
the failed step returned. Two director-commissioned supplemental agents
extended coverage on resource-matching and on epidemiology/energy/GQL. No
private or PII data was used in any query; the predecessor PII exclusion
list was in force throughout.
