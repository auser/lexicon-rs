# LEXICON FEATURE PROMPT — IMPLEMENTATION PROMPT GENERATOR

I want to extend Lexicon with a new capability:

Lexicon should generate **AI implementation prompts derived from repository artifacts**.

These prompts are designed to instruct AI systems to implement code that satisfies the law defined by the repository.

The generated prompts must be saved to:

specs/prompts/<number>-<title>.md

These prompts can then be copied into AI tools like Claude Code, Codex, or other AI assistants to implement code that conforms to the system law.

---

# FEATURE NAME

Implementation Prompt Generator

This feature translates repository artifacts into structured prompts for AI code generation.

---

# GOAL

Lexicon should be able to convert artifacts such as:

contracts  
conformance suites  
coverage mappings  
architecture rules  
verification gates  

into **a well-structured implementation prompt**.

The prompt should guide AI to implement code that satisfies the repository's defined behavior.

This allows Lexicon to become a bridge between:

system law (contracts)  
and  
code generation (AI).

---

# NEW ARTIFACT TYPE

Introduce a new artifact directory:

specs/prompts/

Prompt files must follow this naming pattern:

specs/prompts/<number>-<title>.md

Examples:

specs/prompts/001-memory-blob-store.md  
specs/prompts/002-file-blob-store.md  
specs/prompts/003-redis-storage-backend.md  

Numbers should increment automatically.

The numbering must preserve chronological ordering.

---

# NEW CLI COMMAND

Add a command:

lexicon prompt generate

Usage examples:

lexicon prompt generate blob_store  
lexicon prompt generate kv_store  
lexicon prompt generate storage_backend --target memory  

This command should:

1. collect repository artifacts
2. synthesize them into an implementation prompt
3. generate a markdown file
4. save it under specs/prompts/

---

# ARTIFACT INPUTS

The prompt generator must inspect the following artifacts:

Contracts:

specs/contracts/

Conformance tests:

tests/conformance/

Coverage mappings (if present)

Verification gates:

specs/gates/

Architecture rules:

.lexicon/architecture/

API scan results:

.lexicon/api/

Scoring rules:

specs/scoring/

These artifacts represent the **law of the system**.

The prompt must instruct AI to satisfy this law.

---

# PROMPT STRUCTURE

Generated prompts must follow a consistent structure.

The sections should include:

Title  
Objective  
Repository Context  
Existing Artifacts  
Behavioral Requirements  
Files To Create Or Modify  
Constraints  
Verification Requirements  
Acceptance Criteria  

---

# PROMPT CONTENT REQUIREMENTS

Generated prompts must:

Explain the implementation objective clearly.

Reference the contract clauses relevant to the implementation.

Reference conformance tests that must pass.

Describe invariants implied by the contract.

Describe error semantics if present.

Describe architectural constraints if defined.

Describe any relevant API structure discovered by Lexicon.

The prompt should never invent behavior that contradicts the contract.

---

# EXAMPLE PROMPT STRUCTURE

The generated markdown should resemble:

# IMPLEMENTATION PROMPT — <TITLE>

## Objective

Explain what must be implemented.

## Repository Context

Describe the repository and relevant modules.

## Existing Artifacts

List contracts  
List conformance tests  
List coverage expectations  

## Behavioral Requirements

Summarize contract clauses in clear language.

## Files To Create Or Modify

Suggest likely implementation files.

## Constraints

Explain restrictions such as:

do not modify contracts  
do not weaken tests  
do not bypass verification gates  

## Verification Requirements

Explain which conformance suites must pass.

## Acceptance Criteria

List concrete success conditions.

---

# PROMPT GENERATION STRATEGY

The generator must synthesize prompts from repository artifacts.

Steps:

1. Identify the relevant contract
2. Extract contract clauses
3. Identify associated conformance tests
4. Extract expected behavior
5. Identify API structures
6. Identify architectural constraints
7. Generate a coherent implementation prompt

The prompt must be deterministic given the same artifacts.

---

# PROMPT NUMBERING

Prompt files must use sequential numbering.

Example:

specs/prompts/001-first-storage-backend.md  
specs/prompts/002-file-storage-backend.md  

If files already exist, Lexicon must determine the next number automatically.

---

# OPTIONAL AI ASSISTANCE

Prompt generation may optionally use AI to improve readability.

However, the generator must be capable of producing prompts deterministically without AI.

AI should only enhance clarity.

---

# TUI INTEGRATION

In the Lexicon TUI, users should be able to:

browse prompt artifacts  
preview generated prompts  
copy prompts to clipboard  
regenerate prompts if artifacts change  

Prompts should appear alongside contracts and conformance artifacts.

---

# PATCH PREVIEW

Before writing a prompt file, Lexicon should show a preview.

Example:

Proposed file:

specs/prompts/001-memory-blob-store.md

User options:

accept  
edit  
cancel

---

# IMPLEMENTATION MODULES

Implement the following components:

prompt artifact schema  
prompt generator engine  
artifact extractor  
prompt template renderer  
prompt numbering system  
CLI integration  
TUI integration  

These modules must integrate with existing Lexicon artifact handling.

---

# SAFETY RULES

Prompt generation must never:

modify contracts  
modify conformance tests  
modify gates  
change architecture rules  

Prompt generation is read-only with respect to existing artifacts.

It only produces prompt artifacts.

---

# FUTURE EXTENSIONS

The prompt generator should be designed to support future commands such as:

lexicon implement <contract>  
lexicon prompt refine  
lexicon prompt regenerate  

These may later integrate directly with AI agents.

---

# FINAL GOAL

Lexicon should enable the following workflow:

Define system law using contracts.

Generate conformance tests.

Generate implementation prompts.

Use those prompts with AI to generate code.

Verify that generated code satisfies the law.

This creates a feedback loop where:

contracts define behavior  
prompts guide implementation  
verification enforces correctness.