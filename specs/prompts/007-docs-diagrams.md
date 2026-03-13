# LEXICON DOCS DIAGRAMS PROMPT

I want you to design and implement **beautiful documentation diagrams** for the Lexicon docs website.

The docs site uses **Astro Starlight**.

Your goal is to create a cohesive set of diagrams that visually explain the Lexicon system in a way that feels:

- premium
- modern
- minimal
- architectural
- polished
- easy to understand
- suitable for an open-source systems tool

These diagrams should feel like they belong in a serious developer product.

They must not feel generic, childish, or cluttered.

---

# DESIGN GOALS

The diagrams should communicate Lexicon as a system for defining and enforcing the **behavioral law of software systems**.

The visuals should help users quickly understand:

- what Lexicon is
- how the core concepts relate to each other
- how verification flows through the system
- how Lexicon scales from repo to workspace to ecosystem
- how AI agents interact safely with Lexicon
- how contracts, conformance, coverage, gates, and scoring work together

The style should feel intentional and consistent across all diagrams.

---

# OUTPUT FORMAT

I want production-ready assets that work well in a Starlight docs site.

Please generate the diagrams as either:

1. **inline SVG**
2. **standalone SVG files**
3. **small Astro/HTML components using inline SVG**

Preferred direction:
- use inline SVG or Astro components with inline SVG
- avoid heavy JavaScript
- avoid external libraries unless absolutely necessary
- keep them easy to embed in Markdown or MDX pages

Where appropriate, create companion wrappers or component files for docs usage.

---

# VISUAL STYLE

Use a refined visual system.

The diagrams should use:

- soft gradients or subtle strokes where appropriate
- rounded containers
- clean typography
- semantic grouping
- strong spacing
- minimal visual noise
- tasteful hierarchy
- elegant arrows or directional connectors
- modern systems-diagram aesthetics

The diagrams should work in both light and dark themes if possible.

Use theme-friendly styling and avoid hard-coding overly bright or ugly colors.

The diagrams should look good in a Starlight docs layout.

---

# DIAGRAM SET TO CREATE

I want you to create a cohesive diagram set for the following concepts.

## 1. The Lexicon Model Diagram

Create a primary “Lexicon model” diagram that shows the relationship between:

- Contracts
- Conformance
- Behavior
- API Surface
- Coverage
- Gates
- Scoring
- Architecture
- Ecosystem Governance
- AI Context

This should be the main conceptual diagram for the docs.

The design should clearly show that these are layered or interconnected components of a governed software system.

This diagram should feel like the canonical visual for Lexicon.

---

## 2. Verification Pipeline Diagram

Create a diagram that shows the verification flow:

Contracts
→ Conformance
→ Tests
→ Coverage
→ API Validation
→ Gates
→ Scoring
→ Verification Result

This should clearly communicate the pipeline and its purpose.

It should feel crisp and easy to read at a glance.

---

## 3. Progressive Scope Diagram

Create a diagram that explains the three operating scopes:

- Repo Mode
- Workspace Mode
- Ecosystem Mode

This should visually show:

- Repo Mode as the core foundation
- Workspace Mode as an expansion
- Ecosystem Mode as the broadest layer

This diagram should make it obvious that Lexicon scales progressively rather than forcing enterprise complexity on every repo.

---

## 4. AI Safety Boundary Diagram

Create a diagram that explains how AI interacts with Lexicon.

Show concepts such as:

- AI agent
- AI context
- contracts
- gates
- scoring
- architecture rules
- safe patch loop
- audit trail

The core idea should be:

AI can help evolve the system, but it must operate within the law defined by Lexicon.

This diagram should make the safety model feel clear and trustworthy.

---

## 5. Architecture Governance Diagram

Create a diagram that shows how architecture rules work across a workspace or ecosystem.

Show concepts such as:

- foundation/core crates
- interface crates
- adapter crates
- application crates
- dependency law
- shared contracts

This should help explain how Lexicon prevents architecture drift in larger systems.

---

## 6. Contract Coverage Diagram

Create a diagram that explains:

- contract clauses
- test tags
- tests
- coverage mapping
- missing clauses
- coverage report

This should visually explain what “contract coverage” means.

---

# IMPLEMENTATION REQUIREMENTS

I want the diagrams implemented in a way that is practical for the docs site.

Please do the following:

1. Propose a diagram asset structure, such as:
   - `src/components/diagrams/`
   - `src/assets/diagrams/`
   - or similar

2. Implement reusable diagram components where appropriate.

3. Keep diagram code clean and readable.

4. Use theme-aware classes or variables where possible.

5. Add short usage examples showing how each diagram should be embedded in Starlight pages.

6. If helpful, provide small wrapper components for captions and layout.

---

# FILES TO CREATE

I want you to create a production-ready set of files such as:

- `src/components/diagrams/LexiconModelDiagram.astro`
- `src/components/diagrams/VerificationPipelineDiagram.astro`
- `src/components/diagrams/ProgressiveScopeDiagram.astro`
- `src/components/diagrams/AISafetyDiagram.astro`
- `src/components/diagrams/ArchitectureGovernanceDiagram.astro`
- `src/components/diagrams/ContractCoverageDiagram.astro`

You may use a different structure if you can justify something better.

Also update the relevant docs pages to include these diagrams in tasteful places.

---

# DOCS INTEGRATION

Place diagrams thoughtfully into the docs.

For example:

- the Lexicon model diagram on the homepage or core concepts page
- the verification pipeline in the verification docs
- the progressive scope diagram in the getting started or scope docs
- the AI safety diagram in the AI agents docs
- the architecture governance diagram in the architecture/ecosystem docs
- the contract coverage diagram in the coverage docs

Do not just dump diagrams into pages randomly.
Make the placement intentional and high quality.

---

# UX REQUIREMENTS

The diagrams should:

- look great on desktop
- remain legible on mobile
- scale cleanly
- avoid tiny unreadable labels
- preserve clarity at docs-site widths

Please make them responsive.

---

# QUALITY BAR

I want these diagrams to feel like the docs of a serious, premium developer tool.

They should feel closer to:

- a modern infrastructure product
- a serious systems platform
- a polished open-source docs site

They should not feel like quick placeholder boxes.

Focus on beauty, clarity, and coherence.

---

# DELIVERABLES

Please produce:

1. the diagram component files
2. any supporting styles
3. thoughtful docs integration updates
4. usage examples
5. a brief explanation of the diagram system and design choices

Implement real assets, not just design notes.