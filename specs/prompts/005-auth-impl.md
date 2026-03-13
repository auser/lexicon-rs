I want to port the authentication system from my existing project `holoarch` (directory: `/Users/auser/work/uor/hologram/hologram-architecture-v1/holoarch`) into this repository (`lexicon`).

This is NOT a request to design a new authentication system.

Instead, reuse the existing implementation from `holoarch` with minimal redesign and integrate it properly into the Lexicon architecture.

The `holoarch` project already implements a working browser-based OAuth login flow with a local callback server.

Files to port:

src/auth.rs  
src/commands/auth.rs

These implement:

- OAuth browser login
- provider selection
- callback listener
- token exchange
- token storage
- refresh token flow
- login status
- logout
- CLI UX elements

The behavior must remain the same.

Do not redesign the auth model unless necessary to integrate with the Lexicon architecture.

---

# AUTHENTICATION MODEL

The authentication system works like this:

1. The user runs:

lexicon auth login

2. The CLI:

- opens a browser
- directs the user to the provider OAuth page
- starts a temporary local HTTP server
- waits for the OAuth redirect
- exchanges the code for tokens
- stores tokens locally

3. The CLI then confirms login success.

This flow is already implemented in `holoarch`.

You must preserve this behavior.

---

# PROVIDERS

The current implementation supports:

Claude (Anthropic)
OpenAI

Each provider has:

- client id
- authorization URL
- token exchange endpoint
- redirect port

Preserve this structure.

The provider configuration should remain explicit constants unless there is a strong reason to refactor.

---

# STORAGE

The existing system stores credentials locally.

Port the same behavior, but update paths to use Lexicon directories.

For example:

.lexicon/auth/

Suggested structure:

.lexicon/
  auth/
    claude.json
    openai.json

Tokens must never be printed in logs.

---

# CLI COMMANDS

The following commands must exist in Lexicon:

lexicon auth login
lexicon auth refresh
lexicon auth status
lexicon auth logout

They should map directly to the same behaviors from `holoarch`.

---

# USER EXPERIENCE

Preserve the CLI UX style:

- colored terminal output
- spinner while waiting for OAuth callback
- friendly messages
- provider selection if none specified

Example flow:

User runs:

lexicon auth login

CLI prompts:

Select provider:

Claude
OpenAI

CLI prints:

Opening browser for authentication...

Browser opens OAuth page.

User logs in.

CLI receives callback.

CLI prints:

✓ Authentication successful.

---

# CALLBACK SERVER

The existing implementation starts a temporary HTTP server listening on localhost.

Preserve this design.

Behavior requirements:

- listen on a local port
- wait for redirect with authorization code
- timeout if user does not complete login
- shut down server after receiving callback

Ports currently used:

Claude: 54321  
OpenAI: 1455

Reuse these unless necessary.

---

# MODULE INTEGRATION

Integrate the authentication system into Lexicon cleanly.

Create modules such as:

crates/ai/src/auth.rs
or
crates/core/src/auth.rs

Move the logic from `src/auth.rs` accordingly.

Ensure that:

- authentication logic is reusable
- CLI command handlers call into this module
- AI features can check authentication status

Add helper functions such as:

ensure_authenticated(provider)

---

# AI FEATURE INTEGRATION

Any AI feature in Lexicon must first check authentication.

For example:

lexicon improve
lexicon contract new
lexicon conformance add

These should call something like:

ensure_authenticated("claude")

If not authenticated:

print message instructing the user to run:

lexicon auth login

Do NOT automatically start login during non-interactive workflows.

---

# ERROR HANDLING

Preserve robust error handling.

Examples:

- browser fails to open
- callback not received
- token exchange fails
- network timeout

Errors must be clear and actionable.

---

# SECURITY REQUIREMENTS

Maintain the same security behavior as `holoarch`.

Do NOT:

- log tokens
- store tokens insecurely
- expose tokens in debug output

Local token storage must be restricted to the user environment.

---

# TESTING

Add tests for:

- login flow setup
- callback parsing
- token exchange logic
- token storage
- auth status detection
- logout behavior

Mock network interactions where necessary.

---

# CODE QUALITY

When porting code:

- keep the logic intact
- rename types only where needed
- adjust imports
- move modules to the correct Lexicon crate
- keep functions small and testable
- ensure the project compiles cleanly

Avoid rewriting working code unless required for integration.

---

# FINAL RESULT

After implementation, the following must work:

lexicon auth login
lexicon auth status
lexicon auth refresh
lexicon auth logout

And any AI command should properly verify authentication before running.

---

# SUMMARY

Your job is to:

1. port the existing auth implementation from `holoarch` (located in `/Users/auser/work/uor/hologram/hologram-architecture-v1/holoarch`)
2. integrate it into Lexicon
3. preserve browser OAuth behavior
4. adapt storage paths
5. wire it into AI features
6. keep the UX polished
7. keep the code maintainable

Do not redesign the system unnecessarily.
Reuse the working implementation as much as possible.