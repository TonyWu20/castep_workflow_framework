# No Adapters: Utilities-Based Architecture

After first-principles analysis, the adapter pattern (one trait per DFT code) was rejected in favor of a utilities-based architecture. Layer 2 (`workflow_utils`) provides generic process execution, file I/O, and HPC queue submission with zero software-specific code. All software logic (parsing, modifying, writing input files) lives in parser libraries (`castep-cell-io`, etc.) or Layer 3 project crates.

## Status
Accepted (implemented as of Version 5.0 of the architecture)

## Context

The framework originally considered an adapter-pattern architecture where each DFT code (CASTEP, VASP, Quantum ESPRESSO) would have its own trait implementation providing code-specific logic for input preparation, execution, and output parsing. This is a common pattern in workflow frameworks (e.g., FireWorks, AiiDA).

A first-principles analysis asked: "What does a workflow actually need from DFT software?" The answer was: input preparation, execution, and output parsing. In practice:

- Input preparation is already handled by parser libraries (`castep-cell-io` for CASTEP `.cell`/`.param` files)
- Execution is identical across codes: run binary in workdir with args
- Output parsing is best done by external scripts or consumer crates (Layer 3)

No truly generic interface existed that would meaningfully abstract across DFT codes without leaking implementation details or requiring complex trait hierarchies.

## Decision

Rejected the adapter pattern. Instead:

1. `workflow_core` (Layer 1) defines the abstract contract via traits (`ProcessRunner`, `ProcessHandle`, `HookExecutor`, `StateStore`) -- these are generic execution concepts, not DFT-code concepts
2. `workflow_utils` (Layer 2) provides concrete implementations (`SystemProcessRunner`, `ShellHookExecutor`, `QueuedRunner`) -- all generic, no software-specific code
3. Parser libraries (`castep-cell-io`, future `vasp-io`, `qe-io`) handle software-specific file formats
4. Layer 3 project crates compose everything into actual workflows

## Consequences

- No centralized "supported DFT codes" list in the framework -- each project brings its own parser libraries
- Framework can be used for non-DFT computation pipelines (generic workflow engine)
- The three-layer architecture naturally enforces separation: no software code leaks into workflow_utils
- No need to rebuild the framework when adding support for new software -- just write a new parser library and Layer 3 crate
- The downside: new users must understand the composition pattern (setup closure + parser library) rather than implementing a "CASTEP adapter" trait
