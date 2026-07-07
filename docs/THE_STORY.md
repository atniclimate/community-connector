# The Story So Far

> A short narrative overview, for a human audience, of where this work comes
> from and what it is part of. Community Navigator (this repository) is one
> piece of a larger effort; this document exists so that anyone returning to it
> later - including future collaborators - understands the purpose behind the
> code. Written 2026-07-06.

---

This work began not as a software idea but as an answer to a call. When the
Affiliated Tribes of Northwest Indians (ATNI) set out to build an InterTribal
Climate Resilience Strategy, one of the very first needs its members named was a
way to share information with one another - to know who was working on what,
where knowledge and resources already existed, and how Tribal Nations facing the
same accelerating climate threats could find each other quickly. That call sits
inside a larger, standing charge from the National Congress of American Indians:
to document Tribal climate action, to keep Tribal Nations informed about
impacts, policies, and funding, to support Tribal climate initiatives, to
identify and advocate for empowering policies and funding, to build
consensus-based policy priorities, and to ensure Tribal Nations hold an equal
seat at the national and international tables where climate decisions are made.
Every tool described here is, in the end, an attempt to make those commitments
practical.

The insight underneath all of it is that InterTribal connection - especially for
rapid-response coordination, sharing scarce resources, and mounting a strategic
defense of lands and communities - is itself a form of resilience, and that it
must be built in a way that honors sovereignty rather than eroding it. The hard
reality is a severe and chronic lack of capacity and resources. So the strategy
was never to buy a platform or hand data to a commercial service, but to draw on
the best of the technologies available and turn them toward Tribal purposes,
with Indigenous Data Sovereignty built in from the ground up rather than bolted
on afterward. The early visioning of what an Information Sharing Network (ISN)
could be - who it would serve, and how it would strengthen rather than
compromise sovereignty - was led by a small team within the ATNI Climate program
alongside the Climate Resilience Committee.

The first proof that the idea could live in the real world came through a shared
pilot with the Cascadia Partners Forum. With a small sample of just twenty
participants, the CPF-RCN demonstration showed, for the first time, something
that had only ever been described in words: the interconnectedness of people
across the network, the areas of focus they shared, the affinities between them,
and how close - or how far - any two people were from one another. Running
alongside that work was a parallel and equally important question of how to
protect the data itself, which grew into the Tiered Sovereign Data Framework
(TSDF) - a way of classifying information by who holds authority over it, from
openly shared to sovereign and restricted. TSDF became the foundation on which
the rest of the ecosystem is built.

From that foundation, a constellation of complementary tools has been taking
shape, most of them still in active development. GeoBase offers a common
geospatial baseline - a kind of digital twin of the Cascadia bioregion that can
be draped with decision-making layers and federated InterTribal data. The Tribal
Climate Resilience (TCR) Policy Scanner works to translate climate risk into
real projections and dollar figures, and to trace the funding that could meet
them. The Climate Action Plan (CAP) Assessor reads across the planning documents
of federally recognized Tribes to find gaps, understand the science beneath each
plan, and estimate the capacity needed to address documented risks, impacts, and
vulnerabilities - a task so large it required building a new OCR engine simply to
read the corpus. Others extend the reach further: a land-use analyzer, because
land-based adaptation depends on the people who live on the land; a family of
hazard and weather tools focused on atmospheric rivers, drought, extreme heat,
and wildfire, delivered to community members on the mobile devices they actually
use, through active alerts, tailored forecasts, ready resources, emergency
contacts, and safety information at indigenousaccess.org - and, along the way,
the discovery that federal calculations are not always useful at local scale, so
that even the underlying engines sometimes have to be rebuilt; and an engagement
database that keeps decades of notes, relationships, and history from being lost.
Each of these is related to the others, and each carries a utility that serves a
purpose larger than itself.

What ties them together is the thing this project is really about. A sovereign
network - one that does not depend on commercial services, and that makes the
connections between people, places, knowledge, and need visible in digital space
- is what the InterTribal Climate Resilience Strategy envisioned from the start.
That network has come to be called ISDGraph, the InterTribal Strategic Defense
Graph, and Community Navigator is the piece exploring what the Information
Sharing Network needs simply to make those connections known. The direction of
travel is toward stable 1.0 releases of the full set of tools, their
interoperability made plain, and - for ISDGraph - most likely a network protocol
that links federated nodes, where an address like `ISDGraph:\ATNI-Climate`
reaches one space that is itself only a single node among many. It has been
carried this far by a team of two, neither of whom came from a coding
background, working with the help of large language models to turn decades of
thought - unfinished papers, notes, discussions, the long accumulated memory of
a community - into working software that is open-source and meant to stay that
way. Its promise is simple and deliberate: built by Indigenous People, for
Indigenous People and everyone else. There is far more to build, and to relearn,
and much of the purpose is still only barely possible to put into words - but the
shape of it is becoming clear, and the work is real.

---

### Reference documents and repositories

Internal strategy documents (ATNI): the InterTribal Climate Resilience Strategy;
the NCAI standing charge; the Cascadia Partners Forum pilot; the data-sovereignty
exploration; the sovereign-network and isdGRAPH / ISDGraph concept notes.

Public repositories and services (atniclimate on GitHub; indigenousaccess.org):
- Tiered Sovereign Data Framework - github.com/atniclimate/TieredSovereignDataFramework
- GeoBase - github.com/atniclimate/GeoBase
- Land Use Analyzer - github.com/atniclimate/land-use-analyzer
- PNW Tribal / hazard dashboards - github.com/atniclimate/pnw-tribal-dashboard and related repos
- isdGRAPH / ISDGraph - github.com/atniclimate/isdGRAPH
- Alerts, forecasts, resources, contacts, safety - indigenousaccess.org

The Tribal Climate Resilience Policy Scanner, the CAP Assessor, the OCR engine,
and the engagement database are in active development on local machines alongside
this repository.
