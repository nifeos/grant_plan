# grant_plan

[![check](https://github.com/nifeos/grant_plan/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/nifeos/grant_plan/actions/workflows/ci.yml?query=branch%3Amain)
[![proofs](https://github.com/nifeos/grant_plan/actions/workflows/proofs.yml/badge.svg?branch=main)](https://github.com/nifeos/grant_plan/actions/workflows/proofs.yml?query=branch%3Amain)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/nifeos/grant_plan#license)
[![minimum Rust version](https://img.shields.io/badge/rust-1.88%2B-blue.svg)](https://github.com/nifeos/grant_plan/blob/main/Cargo.toml)

The command-line logic of a capability shell, where naming a resource on the line is what grants
it. `parse` turns a line of bytes into a builtin or a program invocation. `plan_against` places an
invocation's words into the grants a program's manifest declares and answers an `Endowment`, which
is exactly what to hand the child. A line the manifest does not allow gets a typed `Refusal` the
shell prints at the prompt, before anything runs.

The manifest types come from `grant_contract`; this crate knows no program, so the caller resolves
a program's name to its manifest. Patterns on the line expand against a directory listing the
caller supplies, through the `byte_glob` matcher. `no_std`, no allocation, no input or output and
no system call.

## EXAMPLES

A program that reads one file named on the line and takes one option, `-v`:

```rust
use grant_contract::*;
use grant_plan::expansion::Expansion;
use grant_plan::{CommandLine, Refusal, ShellHoldings, parse, plan_against};

const COUNT: Manifest = Manifest {
    program_argument: ProgramArgumentDeclaration::Forbidden,
    memory_grant: MemoryGrantDeclaration::Forbidden,
    file: FileGrantDeclaration::Required { writable: false },
    directory: DirectoryGrantDeclaration::Forbidden,
    short_options: ShortOptions::new(b"v"),
    output: OutputDeclaration::Bytes,
    input: InputDeclaration::Forbidden,
    reports: true,
    interruptible: false,
    clock: false,
    domain: false,
    configuration: false,
    entropy: false,
    network: false,
    machine: false,
    share: false,
    reboot: false,
    sync: false,
    installer: false,
    catalog: false,
    indexes: IndexStoreDeclaration::Forbidden,
    activation_directory: false,
    console_input: false,
    startup: StartupContract::Native,
};

// A shell that holds one directory to grant from, standing at its root.
let holdings = ShellHoldings { directory: true, ..ShellHoldings::default() };

let CommandLine::Run(run) = parse(b"count -v notes.txt") else { unreachable!() };
let plan = plan_against(&run, COUNT, &holdings, &Expansion::none()).unwrap();
let file = plan.file.unwrap();
assert_eq!(file.name.as_bytes(), b"notes.txt");
assert!(!file.writable);
assert_ne!(plan.short_options, 0);

// An option the manifest does not declare is refused, not passed along unread.
let CommandLine::Run(run) = parse(b"count -x notes.txt") else { unreachable!() };
assert_eq!(
    plan_against(&run, COUNT, &holdings, &Expansion::none()),
    Err(Refusal::NoSuchOption),
);
```

`cargo run --example plan_a_line` plans a whole line, with its pipes and redirections.

## What is proven

Kani, a model checker that proves properties for every input up to a stated size, checks the
properties below. Each harness has a falsification patch under `falsifications/`: a small defect
that the harness must catch. Host tests cover the rest: the parser, the planner, the expansion of
patterns into name sets, navigation across two directory grants and every refusal.

- Reading a token's quotes never panics. A word is a slice of its token, and it is quoted only
  when one pair of the same quote wraps the whole token.
- Tokenizing a line yields only non-empty slices of the line, in order, and never more than the
  caller's array holds.
- Splitting a line on its pipes and redirections never indexes past the line, for the empty line
  and every line of one byte: each byte the splitter treats specially, and one letter that stands
  for every other byte.
- A name set never holds more than eight names or 512 bytes of name, and never holds a name twice;
  a refused push leaves it as it was.
- A batch of names is always the front of the byte order: every matching name it leaves out sorts
  after its last, so the next batch skips none.
- A walk through directories moves all the way or not at all, and no walk goes above the holder's
  own root.
- The first interrupt asks a job to stop once, and the shell tears the job down once, within the
  grace window, whether a second interrupt or the window brings it.

The proofs need Kani 0.68.0, which `rustup` does not install: run
`cargo install --locked kani-verifier --version 0.68.0`, then `cargo kani setup`, which downloads
a model checker. After that, `cargo kani` runs all 9 harnesses, and `cargo kani --harness <name>`
runs one. A test fails when that count and the source differ.

## Building and testing

```sh
cargo test
rustup target add x86_64-unknown-none aarch64-unknown-none-softfloat riscv64imac-unknown-none-elf
cargo build --lib --target x86_64-unknown-none   # also aarch64-unknown-none-softfloat, riscv64imac-unknown-none-elf
```

## BUGS

- A shell holds at most two directory grants, and a pattern expands to at most eight names in one
  grant. The planner refuses or batches a larger match, and never truncates it.
- This crate states the name and set bounds as its own constants, so the planner does not link a
  filesystem contract. A consumer that links both pins the two together in its own tests.

## Minimum Rust version

Rust 1.88. Check it with `rustup toolchain install 1.88 --profile minimal`, then `cargo +1.88 test`.

## License

Licensed under either of the MIT license (`LICENSE-MIT`) or the Apache License, Version 2.0
(`LICENSE-APACHE`), at your option.
