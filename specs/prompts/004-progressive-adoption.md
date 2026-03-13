# CLAUDE CODE PROMPT — PHASE 4 PROGRESSIVE ADOPTION / SMALL-TO-LARGE REPO SCALING

We already have prior phases that introduced:

- repo-level contracts, conformance, scoring, gates, AI context, and conversation loops
- API extraction, API diffing, contract coverage, stronger verify/doctor/score integration
- workspace/ecosystem governance, repo roles, architecture graphs, dependency law, shared contracts, and impact analysis

This phase is about something equally important:

# Progressive Adoption and Scale-Appropriate UX

The goal is to ensure this tool works extremely well for:

1. a small single-library repo
2. a medium multi-crate workspace
3. a large multi-repo ecosystem

without forcing the complexity of the largest model onto the smallest use cases.

This is a critical product and architecture requirement.

The tool must feel lightweight and delightful for small repos, while still being capable of scaling up into a serious architecture governance system.

---

# CORE PRODUCT REQUIREMENT

The system must support progressive complexity.

A user with a single crate should be able to get strong value quickly without needing:

- workspace manifests
- ecosystem manifests
- architecture graphs
- repo role taxonomies
- shared contract registries
- cross-repo governance rules

A user with a large workspace or ecosystem should be able to opt into those features in a structured and coherent way.

The architecture, commands, schemas, and UX must reflect this.

---

# THE THREE OPERATING MODES

Design Lexicon as having three explicit operational scopes:

## 1. Repo Mode
For a single repo or single crate, focused on local contract-driven verification.

Capabilities should include:
- local manifest
- contracts
- conformance generation
- behavior scenarios
- scoring
- gates
- verify
- doctor
- API scan/diff
- contract coverage
- Claude context sync
- local conversation history/context

This mode should feel complete and first-class.

It must not feel like a crippled subset.

---

## 2. Workspace Mode
For a multi-crate Rust workspace in one repo.

Additional capabilities may include:
- workspace manifest
- crate roles
- local dependency law
- local shared contracts
- architecture graph
- workspace verify
- workspace doctor
- local impact analysis
- workspace-aware AI context

This mode should build on Repo Mode rather than replace it.

---

## 3. Ecosystem Mode
For multiple repos or a larger software platform.

Additional capabilities may include:
- ecosystem manifest
- repo roles
- federated/shared contracts across repos
- cross-repo compatibility checks
- ecosystem-level architecture policies
- ecosystem verify
- ecosystem doctor
- downstream impact analysis
- ecosystem-aware AI context

This mode should be fully opt-in.

---

# REQUIRED PRODUCT BEHAVIOR

Lexicon must automatically or interactively choose the right operating model based on repository shape and explicit user choice.

Examples:

## Small repo detection
If the repo appears to have:
- one crate
- no workspace structure
- no prior architecture files

then `lexicon init` should default to Repo Mode.

It should scaffold only what is necessary for repo-local success.

## Workspace detection
If the repo appears to be a Cargo workspace with multiple crates, `lexicon init` should:
- detect that structure
- explain the options
- offer:
  - simple repo-level bootstrap only
  - workspace-aware bootstrap
- avoid forcing workspace governance if the user does not want it

## Ecosystem mode activation
Ecosystem governance should not be implicitly forced.
It should require explicit opt-in or explicit configuration.

Examples:
- `lexicon ecosystem init`
- `lexicon init --mode ecosystem`

---

# DESIGN PRINCIPLE: SMALL REPOS MUST FEEL GREAT

This is non-negotiable.

A single-repo user should feel like Lexicon is:

- fast
- intuitive
- low-ceremony
- helpful immediately
- not “enterprise software”
- not overburdened with architecture concepts they do not need yet

When run in a small repo, the generated files should be minimal and elegant.

Do not scaffold unnecessary complexity.

---

# DESIGN PRINCIPLE: BIG REPOS MUST NOT HIT A CEILING

At the same time, larger workspaces and ecosystems should not outgrow the tool.

That means the architecture must be intentionally extensible and layered.

The system should be designed so that a repo can evolve from:

Repo Mode
→ Workspace Mode
→ Ecosystem Mode

without needing a rewrite.

This implies:
- migration-friendly manifests
- stable schema layering
- additive configuration
- reusable internal domain models
- mode-aware command behavior

---

# MODE-AWARE COMMAND BEHAVIOR

Commands should adapt based on mode and context.

Examples:

## Repo-focused commands
These should always work in Repo Mode:
- `lexicon init`
- `lexicon contract new`
- `lexicon conformance add`
- `lexicon behavior add`
- `lexicon score init`
- `lexicon gate init`
- `lexicon verify`
- `lexicon doctor`
- `lexicon sync claude`

## Workspace-aware extensions
These should activate only when relevant:
- `lexicon workspace init`
- `lexicon workspace verify`
- `lexicon workspace doctor`
- `lexicon architecture graph`

## Ecosystem-aware extensions
These should be clearly advanced:
- `lexicon ecosystem init`
- `lexicon ecosystem verify`
- `lexicon ecosystem doctor`

The general commands like `lexicon verify` and `lexicon doctor` should produce sensible scope-aware behavior.

For example:
- in Repo Mode → local verify only
- in Workspace Mode → repo-local verify plus optional workspace summary
- in Ecosystem Mode → local verify by default, ecosystem verify through explicit command or flag

Do not overload the default flow so much that simple repos become noisy.

---

# SCHEMA LAYERING REQUIREMENTS

The schema system must support progressive layering.

I want explicit schema types such as:

## Repo-local schema
Examples:
- `.lexicon/manifest.toml`
- `specs/contracts/*.toml`
- `specs/scoring/*.toml`
- `.lexicon/context/*.json`
- `.lexicon/api/*.json`

## Workspace schema
Examples:
- `.lexicon/workspace.toml`
- `.lexicon/architecture/rules.toml`
- `.lexicon/architecture/graph.json`

## Ecosystem schema
Examples:
- `.lexicon/ecosystem.toml`
- `.lexicon/ecosystem/repos.toml`
- `.lexicon/ecosystem/contracts/*.toml`

These schemas must compose cleanly.

A small repo should not need workspace or ecosystem schema files unless explicitly opting in.

---

# MIGRATION / ESCALATION MODEL

Add a first-class migration path.

A repo should be able to grow from one mode to another.

Examples:

## Repo → Workspace
Command:
- `lexicon workspace init`

This should:
- inspect current repo-local state
- preserve existing contracts/scoring/gates/context
- add workspace manifests and architecture rules
- propose crate roles
- propose dependency law
- let the user refine those through conversation loops

## Workspace → Ecosystem
Command:
- `lexicon ecosystem init`

This should:
- preserve existing workspace state
- add ecosystem manifests and cross-repo governance structures
- propose repo roles and shared contracts
- support guided refinement

This migration must be additive and safe.

Do not force users to restart from scratch.

---

# UX REQUIREMENTS FOR PROGRESSIVE ADOPTION

The UX must make scope visible and understandable.

I want the tool to clearly communicate:
- current mode
- current scope
- features enabled
- advanced features available but not required

Examples:
- dashboard badges like “Repo Mode”, “Workspace Mode”, “Ecosystem Mode”
- explanations during init flows
- concise upgrade prompts when appropriate

Example UX:
“This repository appears to be a single-crate library. I recommend Repo Mode. You can add Workspace Mode later if this grows into a multi-crate workspace.”

This should reduce overwhelm and improve trust.

---

# TUI REQUIREMENTS FOR MODE PROGRESSION

The TUI should reflect the current scope.

## In Repo Mode
Focus on:
- contracts
- conformance
- score
- gates
- API coverage/drift
- local verify/doctor
- conversation artifacts

## In Workspace Mode
Add:
- crate role browser
- local architecture graph
- dependency law explorer
- shared contract browser
- workspace verify/doctor views

## In Ecosystem Mode
Add:
- repo role browser
- ecosystem graph
- federated contract browser
- downstream impact explorer
- ecosystem governance dashboard

Do not clutter the Repo Mode UI with advanced architecture panels by default.

The TUI should feel proportional to the current mode.

---

# CONVERSATION LOOP REQUIREMENTS

The conversation loop system must become mode-aware.

Examples:

## Repo Mode artifact conversations
Should focus on:
- local public API
- contract semantics
- local test expectations
- local scoring/gates
- local behavior scenarios

## Workspace Mode artifact conversations
Should additionally focus on:
- crate roles
- crate boundaries
- dependency direction
- shared internal contracts
- architecture layering

## Ecosystem Mode artifact conversations
Should additionally focus on:
- repo roles
- cross-repo responsibilities
- interface ownership
- federated/shared contracts
- compatibility constraints

The conversation layer should only ask questions relevant to the current scope.

Do not burden a single-library user with ecosystem questions.

---

# INTERNAL ARCHITECTURE REQUIREMENTS

The internal design should support layered capabilities rather than branching into separate products.

Design this using explicit capability boundaries.

For example, think in terms of:
- core repo-domain capabilities
- optional workspace capability layer
- optional ecosystem capability layer

Possible approaches:
- trait-based capability interfaces
- feature-specific service modules
- mode-aware orchestration layer
- additive manifest/schema loaders

The implementation should avoid:
- giant mode-switch conditionals everywhere
- duplicating logic across modes
- coupling simple repo flows to ecosystem internals

---

# REQUIRED FEATURE FLAGS / CAPABILITY MODEL

Introduce an explicit capability model.

Examples:
- repo_contracts
- repo_conformance
- repo_scoring
- repo_gates
- repo_api
- workspace_architecture
- workspace_dependency_law
- workspace_shared_contracts
- ecosystem_governance
- ecosystem_shared_contracts
- ecosystem_impact

The active capability set should be discoverable in code and UI.

This should help keep the architecture clean and avoid accidental cross-mode coupling.

---

# VERIFY / DOCTOR BEHAVIOR REFINEMENT

Refine verification and doctor behavior for progressive scale.

## Repo Mode
`lexicon verify` should be concise and local.
It should focus on:
- contract validation
- tests
- coverage
- API drift
- scoring/gates

## Workspace Mode
`lexicon verify` may summarize:
- local repo verification
- workspace architecture status
without overwhelming the user

`lexicon workspace verify` should provide the deeper workspace analysis.

## Ecosystem Mode
`lexicon verify` should still remain primarily local by default unless explicitly configured otherwise.

`lexicon ecosystem verify` should handle the broader governance checks.

This separation is important for both UX and performance.

---

# PERFORMANCE / SCALE REQUIREMENTS

The tool must remain responsive.

A small repo should not pay the cost of ecosystem scanning.

That means:
- lazy loading of advanced state
- scope-limited scans
- mode-aware verification pipelines
- optional graph/impact analysis
- caching where appropriate

Do not make the default small-repo path expensive.

---

# DEFAULT GENERATED FILES BY MODE

Define clear defaults.

## Repo Mode default generated files
Examples:
- `.lexicon/manifest.toml`
- `specs/contracts/*.toml`
- `specs/scoring/default.toml`
- `tests/conformance/*`
- `tests/behavior/*`
- `.lexicon/context/*`
- `.lexicon/api/*`
- `CLAUDE.md`

## Workspace Mode additional files
Examples:
- `.lexicon/workspace.toml`
- `.lexicon/architecture/rules.toml`
- `.lexicon/architecture/graph.json`

## Ecosystem Mode additional files
Examples:
- `.lexicon/ecosystem.toml`
- `.lexicon/ecosystem/repos.toml`
- `.lexicon/ecosystem/contracts/*`

This file layout should make the progression visible and understandable.

---

# DOCUMENTATION REQUIREMENTS

Update the docs to clearly explain:

- what Repo Mode is
- what Workspace Mode is
- what Ecosystem Mode is
- when to use each
- how to upgrade between them
- what files are added at each stage
- what commands are available at each stage

I want a strong section in the README and architecture docs called something like:

- “Progressive Adoption”
- “Scaling from One Repo to an Ecosystem”
- “Choosing the Right Scope”

---

# TESTING REQUIREMENTS

Add tests for:
- repo shape detection
- mode selection
- init behavior by mode
- migration from repo → workspace
- migration from workspace → ecosystem
- mode-aware verify behavior
- mode-aware doctor behavior
- schema layering
- TUI state/model differences by mode

This phase should not just change docs; it must produce real architecture and behavior.

---

# DELIVERABLES FOR THIS PHASE

Implement:

1. explicit mode/scope model
2. progressive init flow
3. mode-aware command behavior
4. schema layering for repo/workspace/ecosystem
5. migration commands
6. mode-aware TUI structure
7. mode-aware conversation loops
8. performance-aware scope handling
9. documentation updates
10. tests

All code must compile and integrate cleanly with previous phases.

---

# FINAL GOAL

After this phase, Lexicon should feel:

- simple and elegant in a small repo
- powerful and structured in a workspace
- governance-capable in an ecosystem

without forcing enterprise-scale complexity onto small projects.

This phase is about ensuring Lexicon is not only powerful, but adoptable, scalable, and enjoyable across the full range of repository sizes.