# CLAUDE CODE PROMPT — PHASE 3 ECOSYSTEM / MULTI-REPO ARCHITECTURE LAYER

We already have an initial implementation of this project and a second phase that added:

- public API extraction
- contract coverage
- stronger verify / drift / scoring integration

This prompt introduces a third major architectural layer:

# Ecosystem / Multi-Repository Governance

The goal is to evolve this tool from a single-repo contract system into a broader architecture and governance system that can support many repositories within a larger platform.

This is especially important for organizations or projects with many moving parts, shared standards, layered architecture, and cross-repository contracts.

This phase should make the tool capable of acting as a repository constitution and architecture enforcement system across a fleet of repositories.

---

# CORE IDEA

A single repo can have:

- contracts
- conformance suites
- gates
- score
- API drift checks

But a multi-repo ecosystem also needs:

- repository roles
- architectural boundaries
- allowed dependency directions
- shared platform rules
- cross-repo interface contracts
- upstream/downstream compatibility checks
- ecosystem-level drift detection
- architecture governance over time

This phase must add that.

---

# PRODUCT EVOLUTION GOAL

Evolve the tool so it can operate at three levels:

## 1. Repo level
Defines local contract, conformance, score, gates, drift, and AI context.

## 2. Workspace / monorepo level
Defines architecture rules, package relationships, layering, and shared standards inside a single larger codebase.

## 3. Ecosystem / multi-repo level
Defines cross-repo roles, shared contracts, dependency law, interface compatibility, and platform-level governance.

The same tool should be able to support all three.

---

# NEW CONCEPTS TO ADD

Add first-class support for the following concepts:

## 1. Repository role
A repository may have a declared role such as:

- core library
- runtime
- interface definition repo
- adapter / backend repo
- application
- SDK
- plugin / extension
- tooling / CLI
- experimental repo
- architecture / governance repo
- documentation repo
- test harness repo

These roles matter because they should influence:

- what contracts are expected
- what gates are mandatory
- what dependencies are allowed
- what score dimensions are emphasized
- how AI is allowed to modify the repo

Add this to the repo manifest model.

---

## 2. Layer / architecture zone
A repo or crate may belong to an architectural layer such as:

- foundation
- core
- interface
- adapter
- integration
- application
- tooling
- experimental

The system should understand:

- allowed dependency directions
- forbidden imports
- required interface boundaries
- layering rules

This is a crucial addition.

---

## 3. Shared contract / federated contract
Some contracts should be shareable across multiple repositories.

Examples:

- API compatibility rules
- data interchange invariants
- trait semantics expected across implementations
- serialization / wire format guarantees
- plugin interface contracts
- backend capability contracts

Add a concept of shared or federated contracts that can be referenced by many repos.

These should be stored in a structured way.

Possible locations:

- `ecosystem/contracts/`
- `specs/shared_contracts/`
- `.lexicon/ecosystem/contracts/`

You may improve the naming.

---

## 4. Cross-repo interface mapping
The tool should be able to describe relationships like:

- repo A publishes trait/interface X
- repo B implements or depends on trait/interface X
- repo C consumes API surface Y
- repo D must remain compatible with contract Z

This should enable validation such as:

- “consumer repo depends on interface that no longer exists”
- “backend repo no longer conforms to shared contract”
- “adapter repo is importing forbidden internal types”
- “plugin API drift breaks extension ecosystem”

---

## 5. Architecture rule sets
Add architecture rule definitions such as:

- allowed dependency edges
- forbidden dependency edges
- required isolation boundaries
- public API visibility constraints
- layering constraints
- package ownership boundaries
- repo role constraints
- stability expectations by layer

These rules should be versioned, inspectable, and enforceable.

---

## 6. Ecosystem doctor / ecosystem verify
The system should gain higher-level verification modes such as:

- `charter ecosystem verify`
- `charter ecosystem doctor`
- `charter workspace verify`
- `charter workspace doctor`

These should detect:

- forbidden dependency edges
- role violations
- layering violations
- cross-repo contract drift
- interface compatibility breaks
- architecture drift over time
- orphan shared contracts
- stale ecosystem AI context
- inconsistent scoring/gate policies across related repos

---

# COMMAND EXPANSION

Add command families such as:

- `charter workspace init`
- `charter workspace verify`
- `charter workspace doctor`
- `charter ecosystem init`
- `charter ecosystem verify`
- `charter ecosystem doctor`
- `charter role set`
- `charter architecture init`
- `charter architecture graph`
- `charter architecture lint`
- `charter contract share`
- `charter contract import`
- `charter dependency scan`
- `charter dependency diff`

You may refine names, but this is the capability set I want.

---

# ARCHITECTURE GRAPH MODEL

Add a first-class architecture graph model.

This graph should represent things like:

- repositories
- crates/packages/modules
- public interfaces
- contracts
- shared contracts
- dependency edges
- implementation edges
- conformance edges
- ownership zones
- architecture layers

This graph should be inspectable and serializable.

Potential uses:

- TUI architecture visualization
- dependency law enforcement
- drift detection
- cross-repo impact analysis
- AI context generation
- conformance discovery
- shared contract propagation

The architecture graph should be deterministic and testable.

---

# WORKSPACE AND ECOSYSTEM MANIFESTS

Introduce new schema types for:

## workspace manifest
Describes:
- workspace identity
- packages/crates
- local layers
- dependency rules
- shared contracts used within the workspace
- package roles

## ecosystem manifest
Describes:
- participating repositories
- repo roles
- shared contracts
- architecture rules
- allowed dependency directions
- global governance defaults
- ecosystem-level AI policy defaults

These manifests should be:

- explicit
- versioned
- migration-friendly
- inspectable
- suitable for both humans and AI tools

---

# CROSS-REPO CONTRACT MODEL

A shared contract should support:

- contract id
- contract type
- owning repo or authority
- participating repos
- expected providers
- expected consumers
- expected conformance targets
- stability level
- compatibility strategy
- deprecation strategy
- test/conformance references
- API/interface references

Examples:

- a trait contract implemented by many backend repos
- a wire-format contract shared by a runtime and SDK
- a plugin host contract used by extension repos
- a shared scoring/gate standard for all foundation repos

---

# DEPENDENCY LAW

This is important.

The system must support explicit dependency law such as:

- foundation repos may not depend on application repos
- interface crates may not import adapter internals
- runtime crates may depend on interfaces but not applications
- experimental repos cannot publish stable shared contracts
- plugins may only use approved extension interfaces
- adapters may implement shared traits but not define core policy

Add a formal rule model and verification engine for this.

The output should make violations very obvious.

---

# IMPACT ANALYSIS

Add a concept of impact analysis.

When a contract or public API changes, the system should estimate impact such as:

- local repo impact
- workspace package impact
- downstream repo impact
- shared contract consumers affected
- likely breaking change severity

Commands like:

- `charter api diff`
- `charter contract diff`
- `charter ecosystem doctor`

should be able to incorporate impact analysis.

This does not need to be perfect, but it should be useful and structured.

---

# ECOSYSTEM SCORE / GOVERNANCE SCORE

Add optional higher-level scoring for workspaces or ecosystems.

Examples:

- architecture conformance
- dependency law compliance
- shared contract coverage
- downstream compatibility health
- repo governance completeness
- drift hygiene
- stability discipline

This should not replace repo-level score.
It should complement it.

Add explainable score outputs for workspace/ecosystem level analysis.

---

# TUI EXPANSION

The TUI should gain views for:

- workspace graph
- ecosystem graph
- repo roles
- shared contracts
- dependency law violations
- architecture drift reports
- impact analysis
- cross-repo compatibility status
- architecture health dashboard

I want this to feel like a real architecture governance console, not just a CLI report.

Even if the visualization starts simple, structure it for future richness.

---

# AI CONTEXT EXPANSION

The AI context generation system should now also produce context about:

- repo role
- architecture layer
- allowed dependencies
- forbidden dependencies
- shared contracts referenced
- cross-repo responsibilities
- compatibility constraints
- architecture rules relevant to this repo
- impact sensitivity of certain files or APIs

This is extremely valuable for safe AI-assisted work.

Claude should be able to understand not only local repo law, but also ecosystem law.

---

# AI SAFETY EXPANSION

The AI safety model must become architecture-aware.

Examples:

- AI should not introduce forbidden dependency edges
- AI should not move a repo across architectural layers silently
- AI should not redefine shared contracts without history/policy updates
- AI should not modify interfaces with many downstream consumers without surfacing impact
- AI should not weaken architecture rules to “fix” violations
- AI should not bypass ecosystem governance for local convenience

This should be built into doctor/verify/improve logic.

---

# CONVERSATIONAL WORKFLOWS FOR ARCHITECTURE

Extend the conversation loop system so users can create and refine:

- repo roles
- architecture layers
- dependency policies
- shared contracts
- workspace manifests
- ecosystem manifests

These should follow the same pattern as earlier artifact creation:

1. inspect local facts
2. propose a structured draft
3. refine through a guided conversation
4. separate stable vs draft vs advisory content
5. validate
6. write
7. summarize decisions for future reuse

Do not reduce this to flat config editing.

---

# SUGGESTED NEW INTERNAL MODULES / CRATES

Consider adding crates or equivalent modules for:

- `crates/architecture`
- `crates/dependency_law`
- `crates/impact`
- `crates/ecosystem`

Or integrate these responsibilities carefully into existing architecture if that is cleaner.

Keep boundaries explicit.

---

# SAMPLE FILES / SCHEMA TO ADD

Consider file structures such as:

- `.lexicon/workspace.toml`
- `.lexicon/ecosystem.toml`
- `.lexicon/architecture/rules.toml`
- `.lexicon/architecture/graph.json`
- `.lexicon/ecosystem/repos.toml`
- `.lexicon/ecosystem/contracts/*.toml`
- `.lexicon/impact/*.json`
- `specs/shared_contracts/*.toml`

You may refine naming if you have a stronger design.

---

# VERIFY / DOCTOR INTEGRATION

Integrate this new layer into verification and doctor flows.

Examples:

## Repo verify
Should still focus on local repo health.

## Workspace verify
Should validate:
- package layering
- crate dependency law
- local shared contract usage
- workspace architecture rules

## Ecosystem verify
Should validate:
- repo role rules
- cross-repo shared contract conformance
- interface compatibility assumptions
- dependency law across repos
- ecosystem-level drift

## Repo doctor / workspace doctor / ecosystem doctor
Should produce:
- issue lists
- severity levels
- suggested fixes
- impact hints

---

# MIGRATION AND COMPATIBILITY

Design this phase so the system can evolve gradually.

A user should be able to:

- use only repo-level features
- opt into workspace-level features later
- opt into ecosystem-level governance later

Do not require all repos to immediately adopt the full model.

This must be layered and progressive.

---

# DELIVERABLES FOR THIS PHASE

Implement:

1. new schema/model support for workspace/ecosystem/architecture
2. repo role support
3. architecture layer support
4. shared/federated contract support
5. dependency law model
6. architecture graph generation
7. workspace/ecosystem verify foundations
8. workspace/ecosystem doctor foundations
9. impact analysis foundations
10. TUI views or models for architecture inspection
11. AI context expansion
12. documentation and examples
13. tests

All code should compile and fit coherently into the current architecture.

---

# FINAL GOAL

After this phase, the tool should be capable of acting as:

- a repo contract system
- a workspace architecture system
- an ecosystem governance system

Together, these should make it possible to manage not just one repo’s correctness, but the law and structure of an entire software ecosystem.