# Agent documentation

This directory is the entry point for agents working on the PHP Zend opcode specification in P4-SpecTec. Keep project documentation here.

- [PHP build and optimizer settings](fixture-compilation.md) gives the php-src pin, proposed executable lookup and verified fixture variants.
- [Compiling PHP fixtures to an opcode AST](opcode-input.md) covers the extension hook, required data and scope of a compilation unit.
- [Concrete syntax for opcode fixtures](opcode-syntax.md) gives the proposed notation for native exports and hand-written programs.
- [PHP source corpus](test-corpus.md) lists the local fixtures and how to select and copy more from php-src.
- [Testing the opcode printer and loader](extension-testing.md) gives the proposed checks for the first extension milestone.
- [Differential testing against Zend](differential-testing.md) covers future spec comparisons and live VM observation.

## Maintaining these files

- Keep each fact or decision in one place. When it changes, revise that passage and remove obsolete guidance. Use relative links wherever another file needs the same information.
- Keep files focused on what an agent needs for a task. When a topic becomes large or has parts agents consult separately, split it into smaller files and link them where they are used.
- As topic files are added, list them here with a short description so agents can find the relevant file directly.
- Update the relevant documentation during project work. The coordinating agent can assign these edits alongside the work they describe.
- Commit after a meaningful implementation change or at the user's request. Include pending documentation edits in that commit. Use Git history for the commit record.
