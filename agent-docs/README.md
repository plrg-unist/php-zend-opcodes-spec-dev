# Agent documentation

This directory is the entry point for agents working on the PHP Zend opcode specification in P4-SpecTec. Keep agent documentation here. Keep the human-facing project overview and build and test commands concise in the [root README](../README.md).

Backward compatibility with earlier versions of this project is required only when the user explicitly requests it. Introduce project-defined versioning only at the user's request. Supply version fields when external tooling requires them.

Report to the user any proposed limit in the spec or its tools that goes beyond what the pinned Zend engine enforces. Explain which inputs or executions it affects, why it is needed, and how its threshold was chosen so the user can judge whether it is necessary and reasonable.

- [PHP build and optimizer settings](fixture-compilation.md) gives the php-src pin, executable lookup and verified fixture variants.
- The [local PHP manual](../../php-chunked-xhtml/index.html) contains the official language reference, function documentation and migration guides.
- [Compiling PHP fixtures to an opcode AST](opcode-input.md) covers the extension hook, required data and scope of a compilation unit.
- [Concrete syntax for opcode fixtures](opcode-syntax.md) describes the notation used by the printer and loader.
- [Zend opcode AST and Rust library](opcode-ast.md) covers the spec types, library API and Cargo tests.
- [Execution semantics scope](semantics-scope.md) records the single-unit boundary, execution outcomes, and how to decide which Zend effects and builtin behaviors to model.
- [Execution abstraction review](semantics-abstraction-review.md) records the observed outcomes and proposed treatment of build inputs, tool limits and engine optimizations.
- [Semantics milestones](semantics-roadmap.md) gives cumulative targets, library coverage and acceptance tests. [Opcode assignments](semantics-opcodes.md) maps the pinned mnemonics to those milestones.
- [PHP source corpus](test-corpus.md) lists the local fixtures and how to select and copy more from php-src.
- [Testing the opcode printer and loader](extension-testing.md) describes the test runner and its checks.
- [Differential testing against Zend](differential-testing.md) covers the executable, fixture manifest, spec coverage gate, and future VM observation.

## Spec layout

The root syntax file defines the compilation-unit AST. Put operations on PHP values in `01-values/` and rules that execute opcodes and update the frame in `02-vm/`. [Syntax](../spec/00-syntax.watsup) · [Value operations](../spec/01-values/) · [VM rules](../spec/02-vm/)

P4-SpecTec reads each directory in filename order and descends into its subdirectories. Prefix every directory and `.watsup` file under `spec/` with two digits and a hyphen. Number siblings from `00` in dependency order. Renumber as needed when inserting or reordering files, and update affected Rust paths and documentation links in the same change. [P4-SpecTec file discovery](../../p4-spectec/p4spectec/core/src/frontend/parse.rs#L110)

## Dependencies

- Add required Rust dependencies directly.
- Use Python's standard library by default. Make scripts executable, use type hints and keep comments minimal. If a third-party package would significantly simplify the implementation, explain what it would simplify and discuss dependency management with the user before adopting it.
- Discuss any additional build or runtime dependency the native extension needs before adding it.

## Human-facing design records

Maintain four subjects in the human-facing design index. [Design index](../design-docs/README.md)

- **Execution scope.** Update this page when the agreed boundaries for programs or host interactions change. Keep it independent of the set of opcodes implemented in a given milestone. [Execution scope](../design-docs/scope.md)
- **Execution inputs and outputs.** Update this page when the inputs, their source of defaults, the observable outputs, or the reported outcome kinds change. Link to the canonical defaults file. [Execution inputs and outputs](../design-docs/execution-io.md)
- **Intentional discrepancies.** Link each user-approved discrepancy from the index. Give the program and input, the differing stdout bytes, stderr bytes or exit status on both sides, the approved rationale, and a link to the regression case. Obtain the user's approval before implementing any such difference. [Approval policy](semantics-scope.md#choosing-what-to-model) · [Differential test expectations](differential-testing.md#spec-executable-and-differential-suite)
- **Foreign calls.** For every `extern` or `builtin` declaration in `spec/`, identify its provider and what it supplies. Update the inventory when a declaration or its behavior changes, and verify it against the elaborator output using the command on the page. [Foreign call inventory](../design-docs/foreign-calls.md)

Keep useful explanations of how the spec preserves Zend behavior in the relevant agent notes, with links to rules or tests that answer follow-up questions. Milestone plans, gaps in current support, bugs and coverage gaps also belong in agent documentation.

## Maintaining these files

- Keep each fact or decision in one place. When it changes, revise that passage and remove obsolete guidance. Use relative links wherever another file needs the same information.
- Keep files focused on what an agent needs for a task. When a topic becomes large or has parts agents consult separately, split it into smaller files and link them where they are used.
- As topic files are added, list them here with a short description so agents can find the relevant file directly.
- Update the relevant documentation during project work. The coordinating agent can assign these edits alongside the work they describe.
- Commit after a meaningful implementation change or at the user's request. Include pending documentation edits in that commit. Use Git history for the commit record.
