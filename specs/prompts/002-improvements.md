# CLAUDE CODE PROMPT — PHASE 2 IMPROVEMENTS

We have already generated the initial architecture and workspace for this project.

This prompt introduces **critical second-phase improvements** that strengthen the system’s contract-driven verification model.

These improvements must integrate cleanly with the existing architecture rather than replacing it.

The improvements fall into three main areas:

1. Public API extraction and contract drift detection
2. Contract coverage analysis
3. Stronger verification and scoring integration

The goal is to make the system significantly more powerful for real production repositories.

---

# IMPROVEMENT 1: PUBLIC API EXTRACTION

The tool must be able to inspect the public API surface of a Rust crate.

This enables detection of drift between:

- declared contracts
- actual public API

This is extremely important because contracts should govern what the API exposes.

## New Commands

Add commands:

charter api scan
charter api diff
charter api report

### charter api scan

This command should:

1. analyze the crate’s Rust code
2. extract public API items including:

- public structs
- public enums
- public traits
- public functions
- public methods
- public modules
- public constants
- public types

3. normalize the extracted API into a structured schema.

The output should be stored under:

.lexicon/api/current.json

or a similar path.

The schema should include fields like:

- item kind
- name
- module path
- signature
- visibility
- trait associations
- stability annotations (if available)
- documentation summary

Prefer a deterministic structure.

Use Rust tooling where appropriate (such as syn-based parsing, rustdoc JSON, or equivalent).

---

### charter api diff

This command should compare:

current public API
vs
previous baseline API

The baseline can be stored under:

.lexicon/api/baseline.json

The diff must detect:

- added public items
- removed public items
- changed signatures
- changed visibility
- changed trait bounds
- changed generics

The output should clearly show:

- breaking changes
- additive changes
- potentially dangerous changes

The output must be both:

- human readable
- machine readable

---

### charter api report

This command should produce a summary report explaining:

- API stability state
- breaking changes detected
- contract mismatch warnings
- suggested contract updates

This report should integrate into the verification pipeline.

---

# IMPROVEMENT 2: CONTRACT VS API VALIDATION

Contracts should reference capabilities and behavior that correspond to real API elements.

The system must validate:

contract expectations vs public API

Examples:

If a contract says:

“the library exposes a key-value store interface”

The system should detect if:

- the trait exists
- the expected methods exist
- the names match expectations

This does not need perfect semantic matching.

Instead:

Provide a “contract coverage hint” system.

For example:

Contract capability:
KeyValueStore operations

Mapped API elements:
crate::store::KeyValueStore trait
crate::store::put
crate::store::get
crate::store::delete

The system should store these relationships.

Add fields to contract schema such as:

expected_api:
- trait: crate::store::KeyValueStore
- methods:
  - put
  - get
  - delete

Then validate them against extracted API.

---

# IMPROVEMENT 3: CONTRACT COVERAGE ANALYSIS

We must measure how much of the declared contract is actually tested.

Add a new concept:

contract coverage

Contract coverage measures:

How many contract clauses are exercised by tests.

Each contract element should optionally reference tests or test tags.

Example:

contracts/my_contract.toml

clauses:
- id: store_retrievable
  description: values stored must be retrievable
  test_tags:
    - conformance.store_retrieval

- id: delete_idempotent
  description: deleting a missing key is safe
  test_tags:
    - conformance.delete_behavior

Tests should declare tags such as:

#[test]
#[lexicon::tags("conformance.store_retrieval")]

or equivalent metadata.

The system should then compute:

contract coverage %

Example output:

Contract Coverage Report

Contract: key_value_store

Clauses: 12
Covered: 9
Missing coverage: 3

Missing clauses:

- concurrent_write_behavior
- serialization_roundtrip
- large_key_handling

This report should be integrated into:

charter verify
charter score explain

---

# IMPROVEMENT 4: SCORING INTEGRATION

Contract coverage must influence scoring.

Add a new scoring dimension:

contract_coverage

Example weight:

contract_coverage = 15

Coverage scoring model:

100% coverage → full points
90–99% → near full
<80% → penalties
<60% → severe penalties

The scoring engine must clearly explain the effect.

Example:

Score breakdown:

correctness: 40/40
contract_coverage: 12/15
gates: 20/20
lint: 5/5
docs: 4/5

Final score: 81/85

---

# IMPROVEMENT 5: VERIFY PIPELINE INTEGRATION

Extend the verify pipeline to include:

1. contract validation
2. conformance tests
3. behavior tests
4. contract coverage analysis
5. API extraction
6. API diff
7. score calculation

The verify command should output:

- gate results
- score breakdown
- contract coverage report
- API drift report

The output should be extremely readable.

---

# IMPROVEMENT 6: DRIFT DETECTION

The `charter doctor` command should detect drift between:

contract vs API
contract vs tests
tests vs API
API vs baseline
score model vs available checks

Example doctor output:

Issues detected:

- contract references missing trait `KeyValueStore`
- contract clause `serialization_roundtrip` has no tests
- new public function `experimental_fast_put` not referenced by any contract

---

# IMPROVEMENT 7: AI WORKFLOW INTEGRATION

The AI improvement loop must use this information.

When running:

charter improve

the system should consider:

- missing contract coverage
- weak scoring areas
- API drift
- missing conformance tests

The AI should propose improvements such as:

- adding missing tests
- strengthening conformance harness
- updating contracts when appropriate
- documenting new API behavior

The AI must **not** silently weaken contracts or remove tests.

---

# IMPLEMENTATION GUIDELINES

These improvements should introduce new modules or crates such as:

crates/api
crates/coverage

Or equivalent modules integrated into the architecture.

Follow the existing design principles:

- explicit schemas
- deterministic output
- testable components
- repo-local state
- no hidden AI behavior

---

# DELIVERABLES FOR THIS PHASE

Implement:

- API extraction
- API baseline storage
- API diff
- contract coverage system
- scoring integration
- verify integration
- doctor integration
- documentation updates
- tests for the new systems

All code must compile and integrate cleanly with the existing architecture.

---

# FINAL GOAL

After this phase, the system should enforce:

contracts
conformance
API stability
test coverage
verification gates
scoring

Together, these form a strong contract-driven engineering system that AI tools can safely operate within.