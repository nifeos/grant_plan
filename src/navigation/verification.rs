//! Machine-checked proofs, with Kani, of the two promises a walk makes: it moves all the way or not
//! at all, and it never goes above the holder's own root.
//!
//! Each choice (the depth, each step, where the walk ends) is a branch, and the walk is checked
//! inside the branch, so on every path the position and the steps are known and the 512-byte packed
//! path is never merged across paths. Merging it at every branch is what makes a walk over symbolic
//! steps run out of memory.

use super::*;

/// The longest walk the harnesses take.
const MAXIMUM_STEPS: usize = 3;

/// A walk's step choices, and how many steps it may take.
struct Walks<'a> {
    choices: &'a [Step<'static>],
    longest: usize,
}

/// Whether two positions are the same, read over the first 16 packed bytes, which hold every
/// position these harnesses reach. The derived `==` reads all 512 and costs Kani a loop as long.
fn is_same_position(a: &DirectoryPath, b: &DirectoryPath) -> bool {
    a.depth == b.depth && a.used == b.used && a.packed[..16] == b.packed[..16]
}

/// Calls `check` on any position up to two levels below the root, each level named `bc`, and any
/// walk of up to `walks.longest` steps (at most [`MAXIMUM_STEPS`]), each one of `walks.choices`.
fn for_any_walk(walks: &Walks<'_>, check: &dyn Fn(DirectoryPath, &[Step<'static>])) {
    // Each depth walks inside its own branch, so no position is ever a merge of several.
    let mut steps = [Step::Up; MAXIMUM_STEPS];
    let mut here = DirectoryPath::root();
    for _ in 0..2 {
        if kani::any() {
            extend(here, &mut steps, 0, walks, check);
            return;
        }
        assert!(here.descend(b"bc"));
    }
    extend(here, &mut steps, 0, walks, check);
}

/// Ends the walk at `taken` steps, or takes one more step, each choice its own branch.
fn extend(
    here: DirectoryPath,
    steps: &mut [Step<'static>; MAXIMUM_STEPS],
    taken: usize,
    walks: &Walks<'_>,
    check: &dyn Fn(DirectoryPath, &[Step<'static>]),
) {
    if taken == walks.longest || kani::any() {
        check(here, &steps[..taken]);
        return;
    }
    let pick: usize = kani::any();
    kani::assume(pick < walks.choices.len());
    for (i, &choice) in walks.choices.iter().enumerate() {
        if pick == i {
            steps[taken] = choice;
            extend(here, steps, taken + 1, walks, check);
            return;
        }
    }
}

/// Falsification: replayable `falsifications/navigation.verification.a_walk_moves_all_the_way_or_not_at_all.patch`
/// For any position up to two levels deep and any walk of up to two steps, each `..`, a descent
/// into a valid name or a descent into a name that is not one, [`DirectoryPath::apply`] either
/// takes every step or leaves the position exactly as it was. A refused walk is never half taken.
#[kani::proof]
#[kani::unwind(18)]
fn a_walk_moves_all_the_way_or_not_at_all() {
    let walks = Walks {
        choices: &[Step::Up, Step::Down(b"a"), Step::Down(b"")],
        longest: 2,
    };
    for_any_walk(&walks, &|mut here, steps| {
        let before = here;
        match here.apply(steps) {
            Ok(()) => {
                let mut depth = before.depth();
                for step in steps {
                    match step {
                        Step::Up => depth -= 1,
                        Step::Down(_) => depth += 1,
                    }
                }
                assert_eq!(here.depth(), depth);
            }
            Err(_) => {
                kani::cover!(steps.len() == 2, "a walk refused at its last step");
                assert!(is_same_position(&here, &before));
            }
        }
    });
}

/// Falsification: replayable `falsifications/navigation.verification.no_walk_goes_above_your_root.patch`
/// For any position up to two levels deep and any walk of up to three valid steps, a walk that
/// would pass above the root is refused as [`PathRefusal::AtYourRoot`], and every other walk lands
/// at the depth its steps count to. There is no level above the root to reach, and no `..` is
/// silently ignored.
#[kani::proof]
#[kani::unwind(6)]
fn no_walk_goes_above_your_root() {
    let walks = Walks {
        choices: &[Step::Up, Step::Down(b"a")],
        longest: MAXIMUM_STEPS,
    };
    for_any_walk(&walks, &|mut here, steps| {
        let mut depth = here.depth() as isize;
        let mut above = false;
        for step in steps {
            depth += match step {
                Step::Up => -1,
                Step::Down(_) => 1,
            };
            above |= depth < 0;
        }
        let result = here.apply(steps);
        if above {
            kani::cover!(true, "a walk past the root");
            assert_eq!(result, Err(PathRefusal::AtYourRoot));
        } else {
            assert_eq!(result, Ok(()));
            assert_eq!(here.depth() as isize, depth);
        }
    });
}
