# Agent documentation

This directory is the entry point for agents working on the PHP Zend opcode specification in P4-SpecTec. Keep agent documentation here. Keep the human-facing project overview and build and test commands concise in the [root README](../README.md).

Introduce project-defined versioning only at the user's request. Supply version fields when external tooling requires them.

- [PHP build and optimizer settings](fixture-compilation.md) gives the php-src pin, executable lookup and verified fixture variants.
- [Compiling PHP fixtures to an opcode AST](opcode-input.md) covers the extension hook, required data and scope of a compilation unit.
- [Concrete syntax for opcode fixtures](opcode-syntax.md) describes the notation used by the printer and loader.
- [Zend opcode AST and Rust library](opcode-ast.md) covers the spec types, library API and Cargo tests.
- [Execution semantics scope](semantics-scope.md) records the single-unit boundary, execution outcomes, and how to decide which Zend effects and builtin behaviors to model.
- [Semantics milestones](semantics-roadmap.md) gives cumulative targets, library coverage and acceptance tests. [Opcode assignments](semantics-opcodes.md) maps the pinned mnemonics to those milestones.
- [PHP source corpus](test-corpus.md) lists the local fixtures and how to select and copy more from php-src.
- [Testing the opcode printer and loader](extension-testing.md) describes the test runner and its checks.
- [Differential testing against Zend](differential-testing.md) covers the executable, fixture manifest, spec coverage gate, and future VM observation.

## Dependencies

- Add required Rust dependencies directly.
- Use Python's standard library by default. Make scripts executable, use type hints and keep comments minimal. If a third-party package would significantly simplify the implementation, explain what it would simplify and discuss dependency management with the user before adopting it.
- Discuss any additional build or runtime dependency the native extension needs before adding it.

## Maintaining these files

- Use [design-docs](../design-docs/README.md) for short, human-readable explanations of settled choices about PHP-visible behavior and Zend implementation details. Add an entry when an implemented decision illustrates a lasting principle. Keep milestone plans and temporary coverage gaps in the agent documentation.
- Keep each fact or decision in one place. When it changes, revise that passage and remove obsolete guidance. Use relative links wherever another file needs the same information.
- Keep files focused on what an agent needs for a task. When a topic becomes large or has parts agents consult separately, split it into smaller files and link them where they are used.
- As topic files are added, list them here with a short description so agents can find the relevant file directly.
- Update the relevant documentation during project work. The coordinating agent can assign these edits alongside the work they describe.
- Commit after a meaningful implementation change or at the user's request. Include pending documentation edits in that commit. Use Git history for the commit record.
