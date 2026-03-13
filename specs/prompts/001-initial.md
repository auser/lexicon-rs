# CLAUDE CODE BUILD PROMPT

I want you to design and implement a new standalone Rust tool inspired by my `archon` project, but this must be a distinct product with its own identity, architecture, and implementation.

The new tool should be a production-ready Rust CLI/TUI that helps developers create, evolve, and maintain a contract-driven verification system for Rust libraries and workspaces using templates, structured specs, verification gates, and AI-assisted conversational workflows.

Working name: `lexicon`

The core idea is:

- make stable contracts explicit
- make conformance suites reusable
- make scoring functions concrete
- make no-regression gates enforceable
- make AI useful, but bounded and auditable
- make the whole thing colorful, interactive, and enjoyable to use
- make the process repeatable across many repositories

This tool should not be a thin wrapper around templates.
It should be an opinionated engineering system.

---

# PRODUCT VISION

The user should be able to point `lexicon` at a repo and have it help them:

1. define what the library is supposed to do
2. define how to prove it
3. generate reusable testing structure
4. create scoring and verification rules
5. evolve those artifacts over time
6. use AI conversation loops to improve the generated artifacts
7. safely guide Claude Code to make changes without violating contract boundaries

The tool should feel like a mix of:

- architecture scaffolder
- contract authoring assistant
- conformance suite generator
- verification orchestrator
- AI context manager
- repo doctor
- colorful terminal workbench

---

# KEY PRODUCT DIFFERENTIATOR

For every scaffold or authoring workflow, I want an AI conversation loop.

This is not optional.

Commands like:

- `lexicon init`
- `lexicon contract new`
- `lexicon conformance add`
- `lexicon behavior add`
- `lexicon score init`
- `lexicon gate init`

must not just dump static files.

Instead, they should open an interactive, colorful, guided workflow that:

1. asks structured questions
2. proposes an initial artifact
3. shows the draft to the user
4. opens an AI-assisted refinement conversation
5. lets the user iterate on the artifact conversationally
6. updates the generated artifact based on the conversation
7. records enough metadata to improve future generations
8. preserves explicit boundaries between stable contract and speculative ideas

I want the tool to treat artifact creation as an evolving conversation, not a one-shot form.

This conversational workflow should get better as more artifacts are created over time.

That means the system should preserve and reuse context like:

- repo type
- library domain
- past contract patterns
- scoring preferences
- gate policies
- naming conventions
- prior user decisions
- style preferences
- prior AI-generated artifacts
- prior conversation summaries

Do not build a vague “chat mode.”
Build explicit conversational artifact refinement loops.

---

# DESIGN PRINCIPLES

This tool must be:

- opinionated
- contract-first
- spec-driven
- colorful
- interactive
- deterministic where enforcement matters
- flexible where ideation matters
- AI-assisted but not AI-dependent for core verification
- safe against AI gaming behavior
- suitable for real production repos
- pleasant enough that I want to use it every day

The UX should feel premium.

---

# INSPIRATION FROM ARCHON

This new tool is inspired by `archon`, but it is not a clone.

Borrow and reinterpret ideas such as:

- manifest-driven metadata
- structured rules
- contract-like declarations
- generated AI context
- managed `CLAUDE.md` insertion/update
- repository-aware tooling
- architecture-level reasoning

But do not turn this into “archon but renamed.”

This new tool should specialize in:

- stable behavior contracts
- reusable conformance suites
- scoring functions
- no-regression gates
- conversational artifact refinement
- bounded AI-guided improvement loops

Design it as a separate repo first.
It can become compatible with `archon` concepts later.

---

# PRIMARY USER EXPERIENCE

I want a beautiful Rust CLI/TUI.

It should have:

- colorful terminal output
- panel-based TUI views where useful
- interactive wizards
- keyboard-driven flows
- previews before file writes
- diff views
- scorecards
- tree views
- contract browsers
- gate browsers
- validation summaries
- strong help text
- delightful status messaging
- machine-readable output modes for CI

The TUI should feel modern and polished, not like a basic prompt loop.

Use strong terminal UI libraries and build this like a real product.

---

# CORE COMMAND MODEL

I want a command system roughly like this:

- `lexicon init`
- `lexicon contract new`
- `lexicon contract edit`
- `lexicon contract lint`
- `lexicon contract list`
- `lexicon conformance add`
- `lexicon conformance sync`
- `lexicon behavior add`
- `lexicon behavior sync`
- `lexicon score init`
- `lexicon score explain`
- `lexicon gate init`
- `lexicon verify`
- `lexicon improve`
- `lexicon improve --goal correctness`
- `lexicon improve --goal performance`
- `lexicon doctor`
- `lexicon sync claude`
- `lexicon tui`

You may refine the command names if you have a better structure, but keep the spirit.

---

# CONVERSATIONAL ARTIFACT CREATION REQUIREMENTS

This is critical.

For every major authoring or generation command, the flow should support a conversation loop.

For example, `lexicon contract new` should:

1. inspect the repo
2. infer likely public API/domain shape
3. ask the user guided questions
4. propose a first contract draft
5. show the draft in a readable view
6. let the user refine it conversationally
7. optionally call AI to improve clarity, coverage, and consistency
8. separate:
   - stable requirements
   - edge cases
   - forbidden behavior
   - non-goals
   - examples
   - implementation notes
9. validate the result
10. write the artifact

Likewise, `lexicon conformance add` should:

1. inspect contract artifacts
2. infer trait/factory/test harness structure
3. propose reusable conformance scaffolding
4. let the user refine it conversationally
5. optionally use AI to improve harness completeness
6. preserve the distinction between:
   - stable conformance obligations
   - generated starter tests
   - optional/advisory tests

Likewise, `lexicon score init` and `lexicon gate init` should use conversational refinement loops rather than static generation.

These conversation loops should produce structured summaries that can later be reused.

---

# REPO-LOCAL MEMORY / CONTEXT MODEL

I want the tool to build repo-local context over time.

It should maintain internal files such as:

- repo profile
- artifact summaries
- prior conversational decisions
- naming/style preferences
- verification policy defaults
- scoring defaults
- contract evolution history
- prompt context for future AI calls
- drift reports
- audit logs for AI-guided changes

This context should be inspectable and local to the repo.
Do not depend on a hosted service.

I want a system that becomes more context-aware the more I use it in the same repo.

---

# WHAT THE TOOL MUST GENERATE

The tool should scaffold and maintain structures like:

- `specs/contracts/`
- `specs/behavior/`
- `specs/scoring/`
- `specs/non_goals/`
- `tests/conformance/`
- `tests/behavior/`
- `tests/integration/`
- `fuzz/`
- `benches/`
- `.lexicon/`
- `CLAUDE.md`

And likely internal files like:

- `.lexicon/manifest.toml`
- `.lexicon/context/*.json`
- `.lexicon/conversations/*.json`
- `.lexicon/audit/*.json`
- `.lexicon/templates/`
- `.lexicon/cache/`
- `.lexicon/state/`

Feel free to improve the layout, but keep it explicit and repo-local.

---

# SCHEMA REQUIREMENTS

Design explicit schemas for:

- repo manifest
- contract definitions
- invariants
- required semantics
- forbidden behavior
- examples
- non-goals
- conformance suite definitions
- behavior scenarios
- scoring model
- gates model
- AI policy model
- conversation transcript summaries
- artifact generation sessions
- audit records
- doctor/drift reports

All schemas must be:

- versioned
- migration-friendly
- validated
- understandable by both humans and AI tools

Prefer TOML or YAML for user-authored files and JSON/TOML for internal machine state where appropriate.

---

# CONTRACT MODEL

Contracts should support concepts like:

- contract id
- title
- status
- stability level
- scope
- capabilities
- invariants
- required semantics
- forbidden semantics
- edge cases
- examples
- non-goals
- implementation notes
- test expectations
- version history

The contract system must clearly distinguish between:

- stable contract
- draft contract
- advisory notes
- implementation hints

Do not blur these together.

---

# CONFORMANCE MODEL

I want reusable conformance testing.

The tool must help generate trait-based and factory-based conformance suites so multiple implementations can be tested against the same contract.

The generated approach should support:

- shared harness logic
- implementation adapters
- fixture reuse
- table-driven cases where appropriate
- extension points for implementation-specific behavior
- separation between required contract tests and optional implementation tests

The tool should guide the user toward reusable structure, not scattered ad hoc tests.

---

# BDD / BEHAVIOR MODEL

I want a behavior layer for readable intent, but it must not be the only source of truth.

Use it for:

- acceptance scenarios
- user-visible behavior
- regressions described narratively
- examples that explain intent

The system should generate and manage behavior artifacts without letting them replace the actual contract.

---

# SCORING MODEL

I want explicit scoring functions.

The tool must support weighted scoring across dimensions like:

- correctness
- contract pass rate
- conformance coverage
- behavior pass rate
- property test health
- fuzz smoke health
- benchmark regressions
- lint quality
- docs/spec completeness
- panic safety
- diagnostics quality
- forbidden change detection

There must be:

- required gates
- scored checks
- advisory checks

The score system must be deterministic and explainable.

I want commands like `lexicon score explain` to tell me exactly why the score is what it is.

---

# NO-REGRESSION GATES

This is critical.

The tool must create and enforce hard no-regression gates such as:

- fmt
- clippy
- unit tests
- conformance tests
- behavior tests
- property tests
- fuzz smoke
- benchmark smoke
- snapshot validation
- contract drift validation
- public API drift checks
- required file integrity checks

The tool must explicitly protect against AI or human attempts to game the system.

Examples of forbidden behavior:

- deleting tests to make CI pass
- weakening assertions without acknowledgment
- silently loosening score thresholds
- rewriting contract semantics without updating contract status/history
- weakening required gates without policy approval
- gaming performance baselines
- hiding failures behind skipped tests

Build architectural protections for this.

---

# AI-GUIDED IMPROVEMENT LOOP

I want a safe AI-guided improvement loop.

`lexicon improve` should:

1. read contracts, scoring, gates, and repo context
2. identify failures, weaknesses, or improvement opportunities
3. propose a narrow patch
4. explain why the patch should help
5. apply the patch in a controlled way
6. run verification
7. compare score delta
8. reject patches that violate required gates
9. generate an audit record
10. produce a human-readable summary

This is not an uncontrolled auto-rewrite feature.
It must be bounded and auditable.

---

# CLAUDE CODE INTEGRATION

This tool is intended to work well with Claude Code.

Design the system so Claude can read generated files and immediately understand:

- what the repo does
- what behavior is stable
- what can and cannot change
- how to add tests
- how to interpret conformance
- how score is computed
- what gates are mandatory
- what changes require spec updates
- how to work safely in the repo

Generate and maintain `CLAUDE.md` using explicit managed blocks.

I want `lexicon sync claude` to manage repo-specific AI context safely and repeatably.

---

# SAFETY AND POLICY MODEL

The tool must embed a strong safety model.

I want explicit policy around:

- what files AI may edit
- what files require manual review
- what files are protected
- what changes require contract history updates
- what scoring/gate changes are restricted
- how test deletions are handled
- how skipped tests are handled
- how contract downgrades are handled
- how benchmark baselines are updated

The architecture must assume AI can make “locally clever but globally bad” choices and defend against that.

---

# TECHNICAL ARCHITECTURE

Implement this as a real Rust workspace with strong crate boundaries.

A likely layout is:

- `crates/cli` — command entrypoint and CLI UX
- `crates/tui` — rich interactive TUI
- `crates/core` — orchestration and domain services
- `crates/spec` — schemas, parsing, validation, migrations
- `crates/scaffold` — template generation and file emission
- `crates/conversation` — conversational refinement loops and session state
- `crates/conformance` — conformance domain model/generation
- `crates/gates` — verification runners/result normalization
- `crates/scoring` — scoring engine
- `crates/ai` — AI prompt/context generation and integration boundaries
- `crates/fs` — file ops, diffs, patching, safe writes
- `crates/repo` — repository inspection and codebase analysis
- `crates/audit` — audit logs and history
- `xtask/` — dev automation

You may adjust crate boundaries if you have a better design, but keep the architecture modular and production-grade.

---

# TUI REQUIREMENTS

The TUI matters a lot.

I want:

- a dashboard
- contracts browser
- gate browser
- scoring summary
- repo health view
- conversation review view
- artifact diff preview
- doctor/drift report explorer

The TUI should support flows like:

- initialize repo
- author new contract
- refine artifact through conversation
- preview generated conformance harness
- inspect score breakdown
- review verify results
- inspect AI improvement audit trail

This should not be an afterthought.

---

# IMPLEMENTATION REQUIREMENTS

I want you to actually implement this, not just describe it.

Please do the following:

1. choose the final product name
2. write an architecture document
3. define schema files and examples
4. create the Rust workspace
5. implement the CLI foundation
6. implement the TUI foundation
7. implement the conversation-loop foundation
8. implement scaffolding for repo init
9. implement contract creation flow
10. implement conformance generation flow
11. implement scoring/gates initialization
12. implement verify foundation
13. implement Claude context sync
14. include tests
15. include docs
16. include sample generated repo output

I want at least one end-to-end happy path working:

- `lexicon init`
- conversationally define a contract
- generate conformance scaffolding
- initialize scoring and gates
- run verify
- sync `CLAUDE.md`

---

# DEVELOPMENT CONSTRAINTS

Follow these rules:

- do not create a toy
- do not fake major features with placeholders
- do not hide core flows behind TODOs
- do not centralize everything into one crate
- do not make AI required for basic local verification
- do not require a hosted service
- do not make target repos depend on lexicon at runtime
- do not confuse speculative ideas with stable contract
- do not let “conversation memory” become hidden magic; keep it inspectable
- do not weaken safety boundaries for convenience

---

# DELIVERABLES

I want:

- full Rust workspace
- compileable code
- implemented starter commands
- implemented schema models
- implemented templates
- sample generated repo structure
- README
- architecture doc
- AGENTS.md
- sample `CLAUDE.md` managed block format
- sample contract file
- sample conformance harness
- sample scoring config
- sample gate config
- sample doctor report
- sample conversation session record
- sample audit record

---

# BUILD PHASES

Work in phases and keep the code compiling:

1. product name and architecture
2. schema design
3. workspace scaffolding
4. CLI foundation
5. TUI foundation
6. conversation system foundation
7. repo init scaffolding
8. contract generation flow
9. conformance generation flow
10. scoring/gates generation
11. verify foundation
12. Claude context sync
13. tests/docs/polish

At each phase:
- explain decisions briefly
- implement real code
- keep the project coherent

---

# WHAT I WANT FIRST

Start by doing the following:

1. choose the final name
2. propose the crate layout
3. propose the schema layout
4. propose the generated repo layout
5. propose the command structure
6. explain the conversation-loop architecture
7. then begin implementation

Be highly opinionated, production-minded, and concrete.
Optimize for a tool I can actually build and iterate on with Claude Code.