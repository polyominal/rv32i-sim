# rv32i-sim

This is a toy simulator for a 5-stage
pipelined RV32I processor, written in Rust.
The document is organized as follows:

- [Usage](#usage)
- [Implementation](#implementation)
- [Credits](#credits)

## Usage

This simulator is implemented with Rust.
To build the project, run

```bash
cargo build --release
```

To run the simulator:

```bash
cargo run --release -- [PATH_TO_ELF_FILE] [OPTION...]
```

For example, you can do

```bash
cargo run --release -- test-riscv/ackermann.riscv --history
```

Alternatively, you can do a single-cycle simulation:

```bash
cargo r --release -- test-riscv/ackermann.riscv --history -i S
```

## Implementation

To generate a documentation that provides an
(somewhat) organized overview, execute ``cargo doc``.

This section is structured as follows:

- [Memory management unit](#memory-management-unit)
- [Loading ELF files](#loading-elf-files)
- [Pipeline overview](#pipeline-overview)
- [Data hazards](#data-hazards)

### Memory management unit

The `mmu.rs` file implements the memory management unit.
It references the starter code, which includes a two-level page
table that allocates on-demand. We used Rust's `Option<>` module
to represent the entries. Each operation on this MMU takes $O(1)$.

### Loading ELF files

We used the black-box utilities from the `object` crate
to parse ELF files. The primary task is to load the segments
into the memory and initiate the simulation from the specified
entry point. The `elf_helper.rs` module implements the necessary
wrapper functions.

### Pipeline overview

The simulator didn't replicate the exact CPU due to the complexity of
implementing all control logics and signals in a high-level language.
Instead, it adopts a state-machine approach. A "state" comprises:

- CPU state: the program counter and $32$ general-purpose registers (`cpu.rs`)
- Memory state: encapsulated in the memory management unit (`mmu.rs`)
- Pipeline state: the $4$ pipeline registers (`pipelined/pipeline.rs`)

The simulator progresses the state with each cycle
(`pipelined/mod.rs`). During a transition, we read from the
previous state and write to the upcoming state, strictly,prohibiting the reverse. This simplifies modularization
as it essentially defines a mapping between states.

To start with, the lower-level implementaions for
the $5$ stages are given in `stages_simple.rs`:

- IF stage: `instruction_fetch` that returns the instruction at the PC
- ID stage: `instruction_decode` that decodes the raw 32b
instruction into a wrapped `Instruction` unit, and `register_read`
returns the $2$ values from register reads
- EX stage: `execute` that performs the ALU operations
or system calls and returns a corresponding result
- MEM stage: `memory_access` that operates on the memory.
Note that at this stage, we can determine the actual write-back
result, and this function returns the WB result directly
- WB stage: `write_back` that writes the WB result
to a register

In the pipelined implementation (`pipelined/stages.rs`),
we implement $5$ pipeline
stages which are essentially wrappers around the lower-level
functions. They read from the memory/previous pipeline register
and writes to the pipeline register/some CPU register.

Note that branching is handled outside of the stages
(`pipelined/mod.rs`).
This is for more secure control over the CPU state.
It checks for the execution result in the EX/MEM register,
updates the PC and flushes the pipeline accordingly.

### Data hazards

The data hazards, as
described in P&H p. 300 - 301, are handled
in the predicate functions in `pipelined/pipeline.rs`.

- EX/MEM hazard: `ex_hazard_op1` and `ex_hazard_op2`
- MEM/WB hazard: `mem_hazard_op1` and `mem_hazard_op2`
Both are called during the EX stage, and you will
have to prioritize the EX/MEM forwarding over MEM/WB forwarding.

To detect load-use hazards, we use the `load_hazard`
functions. A NOP is inserted if it is detected.
Note that with such an extra stall, we have to introduce
an extra forwarding path that passes the value from the
current MEM/WB register to the ID stage. The predicate
function is implemented as `wb_hazard_op1` and `wb_hazard_op2` (
I'm sorry for the poor naming).

### Control hazards (optimized via branch prediction)

The branch predictor is implemented
as a black-box module in `pipelined/branch_predictor.rs`,
that supports

- Predicting the branch result given a PC
- Updating the predictor based on an actual branch result

We implemented a $4$-state buffered predictor,
which maintains an ordered state on each entry
of the buffer. We increment/decrement the state
based on the given branch results. For a prediction,
we predict the branch taken if the state is one of
the higher $2$ levels, and not taken otherwise.

To inject the predictor logic into the pipeline,
we introduce a `predicted_branch_taken` predicate
variable in the main loop that records the prediction
from the last cycle (in the ID stage). If in the next
cycle we find that the prediction is incorrect (EX stage),
we update the PC and flush the pipeline, like what
we do with usual branches/jumps.

## Credits

- Patterson & Hennessy book
- [riscv-5stage-simulator](https://github.com/djanderson/riscv-5stage-simulator) by user `djanderson` on GitHub:
provided an elegant reference implementation
for instruction decoding
- [RISCV-Simulator](https://github.com/hehao98/RISCV-Simulator) by
user `hehao98` on GitHub: provided a reference implementation for
the buffered predictor, and sample test ELF files
- [CS61C Reference Card](https://inst.eecs.berkeley.edu/~cs61c/sp22/pdfs/resources/reference-card.pdf): a standard reference for the
implementation
