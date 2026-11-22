# Agent documentation

This directory is the entry point for agents working on the PHP Zend opcode specification in P4-SpecTec. Keep agent documentation here. Keep the human-facing project overview and build and test commands concise in the [root README](../README.md).

Add project-defined version numbers only when the user explicitly requests them.

- [PHP build and optimizer settings](fixture-compilation.md) gives the php-src pin, executable lookup and verified fixture variants.
- [Compiling PHP fixtures to an opcode AST](opcode-input.md) covers the extension hook, required data and scope of a compilation unit.
- [Concrete syntax for opcode fixtures](opcode-syntax.md) describes the notation used by the printer and loader.
- [PHP source corpus](test-corpus.md) lists the local fixtures and how to select and copy more from php-src.
- [Testing the opcode printer and loader](extension-testing.md) describes the test runner and its checks.
- [Differential testing against Zend](differential-testing.md) covers future spec comparisons and live VM observation.

## Dependencies

Use Python's standard library by default. If a third-party package would significantly simplify the implementation, explain what it would simplify and discuss dependency management with the user before adopting it. Discuss any additional build or runtime dependency the native extension needs before adding it.

## Maintaining these files

- Keep each fact or decision in one place. When it changes, revise that passage and remove obsolete guidance. Use relative links wherever another file needs the same information.
- Keep files focused on what an agent needs for a task. When a topic becomes large or has parts agents consult separately, split it into smaller files and link them where they are used.
- As topic files are added, list them here with a short description so agents can find the relevant file directly.
- Update the relevant documentation during project work. The coordinating agent can assign these edits alongside the work they describe.
- Commit after a meaningful implementation change or at the user's request. Include pending documentation edits in that commit. Use Git history for the commit record.
