# Changelog

Dates are UTC. One line per release, what changed and why, so a reader of the diff has the
sentence beside it.

## Unreleased

- First version.
- Removed `TwoRoots::resolve_from`, `TwoRoots::apply_from`, `Bindings::resolve_absolute` and
  `Bindings::split_absolute` from the public interface. Only this crate's own tests called them.
  `TwoRoots::resolve` and `Bindings::lookup` remain.
- Rebuilt the refusal message lookup so the compiler proves it covers every scoped-block refusal,
  with no unreachable arm. The messages are unchanged.
- Pinned `byte_glob` to its current main (f4ebfe6), 15 commits newer than before.

## BUGS

- The template's lag check guards the CI pin, not this file; a change that matters to a user of
  the crate belongs in a release line here.
