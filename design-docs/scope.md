# Execution scope

The spec runs the opcodes from one compilation unit in a fresh PHP CLI request with fixed initial settings. A unit usually contains one file's top-level code and declarations. A call can resolve at runtime to code in that unit or to a supported builtin. Compiling new source with `eval` and loading another unit through `include`, `require`, or an autoloader are outside scope.

The program can read stdin and write stdout and stderr. It can use supplied path names as strings. Filesystem access, network access, starting processes, and reading the host environment, clock, or random source are outside scope. The spec also excludes host resource limits and allocator statistics.
