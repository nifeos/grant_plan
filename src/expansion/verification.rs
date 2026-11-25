//! Machine-checked proofs, with Kani, of a name set's bounds and of the batch it selects: it never
//! holds more than [`MAXIMUM_NAMES`] names or [`MAXIMUM_NAME_SET_BYTES`] bytes, never holds a name
//! twice, and a batch is always the front of the byte order.

use super::*;

/// The set's own bookkeeping holds: at most [`MAXIMUM_NAMES`] names, each ending after the one
/// before it, together within [`MAXIMUM_NAME_SET_BYTES`].
fn is_bounded(set: &NameSet) -> bool {
    let mut ordered = true;
    for i in 1..set.n {
        ordered &= set.ends[i - 1] < set.ends[i];
    }
    set.n <= MAXIMUM_NAMES
        && (set.n == 0 || set.ends[0] > 0)
        && ordered
        && set.name_bytes() <= MAXIMUM_NAME_SET_BYTES
}

/// Falsification: replayable `falsifications/expansion.verification.a_set_never_holds_more_names_than_it_counts.patch`
/// From a set of any number of names up to [`MAXIMUM_NAMES`], one more push, of a name the set
/// holds, a name it does not, or a name that cannot travel in a grant: [`NameSet::try_push`]
/// takes a name only when it can travel in a grant, is not held already and fits, and then holds
/// it last, so the set never holds more names than [`MAXIMUM_NAMES`] nor a name twice. A refused
/// push leaves the set as it was.
#[kani::proof]
#[kani::unwind(11)]
fn a_set_never_holds_more_names_than_it_counts() {
    // Each count goes on inside its own branch.
    for held in 0..MAXIMUM_NAMES {
        if kani::any() {
            push_any(held_names(held));
            return;
        }
    }
    push_any(held_names(MAXIMUM_NAMES));
}

/// A set holding the first `held` names of `a`, `b`, and so on, written straight into its fields
/// rather than pushed, so that Kani does not run one push per name before the one it checks.
fn held_names(held: usize) -> NameSet {
    let mut set = NameSet::empty();
    for k in 0..held {
        set.bytes[k] = b'a' + k as u8;
        set.ends[k] = (k + 1) as u16;
    }
    set.n = held;
    set
}

/// The names a push draws from: one the set holds once it holds any, one it never starts with,
/// and one that cannot travel in a grant.
const PUSHED_NAMES: [&[u8]; 3] = [b"a", b"z", b"."];

/// Pushes one of [`PUSHED_NAMES`].
///
/// Each name goes on inside its own branch, so on every path the set and the name are known and
/// [`NameSet::try_push`] takes one way out. A symbolic name merges its two ways out, the count
/// of names turns symbolic, and Kani runs out of memory.
fn push_any(set: NameSet) {
    let pick: usize = kani::any();
    kani::assume(pick < PUSHED_NAMES.len());
    for (i, &name) in PUSHED_NAMES.iter().enumerate() {
        if pick == i {
            push_one(set, name);
            return;
        }
    }
}

/// Pushes `name` onto `set` and checks what the push did.
fn push_one(mut set: NameSet, name: &[u8]) {
    let is_directory: bool = kani::any();
    let before = set;
    match set.try_push(name, is_directory) {
        Ok(()) => {
            assert!(crate::is_valid_file_name(name) && !before.contains(name));
            assert_eq!(set.n, before.n + 1);
            assert_eq!(set.last(), Some(name));
            assert_eq!(set.is_directory[set.n - 1], is_directory);
        }
        Err(NameSetFull) => {
            assert!(
                !crate::is_valid_file_name(name)
                    || before.contains(name)
                    || before.n == MAXIMUM_NAMES
            );
            assert_eq!(set.n, before.n);
            // Element by element: the array `==` is a comparison loop over every slot.
            for k in 0..MAXIMUM_NAMES {
                assert_eq!(set.ends[k], before.ends[k]);
                assert_eq!(set.is_directory[k], before.is_directory[k]);
            }
            assert!(set.name_bytes() == before.name_bytes());
        }
    }
    assert!(is_bounded(&set));
    kani::cover!(set.n == MAXIMUM_NAMES, "a full set");
}

/// Falsification: replayable `falsifications/expansion.verification.a_set_never_holds_more_bytes_than_it_has.patch`
/// From a set holding no name, one name of the longest length a grant can carry, two long names, or
/// three names that leave four bytes, one more push, of a name that exactly fills the room left or
/// of one byte more: [`NameSet::try_push`] takes a name exactly when it fits beside what is held,
/// and the names held never pass [`MAXIMUM_NAME_SET_BYTES`] together.
#[kani::proof]
#[kani::unwind(254)]
fn a_set_never_holds_more_bytes_than_it_has() {
    // Each set and each push goes on inside its own branch, as in [`push_any`]. No two names in
    // one set are of one length, so none is held twice.
    match kani::any::<u8>() {
        0 => push_to_fill(NameSet::empty()),
        1 => push_to_fill(long_names([MAXIMUM_NAME_BYTES])),
        2 => push_to_fill(long_names([MAXIMUM_NAME_BYTES, MAXIMUM_NAME_BYTES - 1])),
        _ => push_to_fill(long_names([MAXIMUM_NAME_BYTES, MAXIMUM_NAME_BYTES - 1, 5])),
    }
}

/// A set holding names of the byte `a` of each of `lengths`, written directly rather than pushed.
fn long_names<const NAMES: usize>(lengths: [usize; NAMES]) -> NameSet {
    let mut set = NameSet::empty();
    set.bytes = [b'a'; MAXIMUM_NAME_SET_BYTES];
    let mut end = 0;
    for (k, length) in lengths.into_iter().enumerate() {
        end += length;
        set.ends[k] = end as u16;
    }
    set.n = NAMES;
    set
}

/// Pushes a name of the byte `b` that exactly fills the room `set` has left, or is one byte longer,
/// each no longer than a grant can carry.
fn push_to_fill(set: NameSet) {
    let room = MAXIMUM_NAME_SET_BYTES - set.name_bytes();
    if room > MAXIMUM_NAME_BYTES {
        push_long(set, MAXIMUM_NAME_BYTES);
    } else if kani::any() {
        push_long(set, room);
    } else {
        push_long(set, room + 1);
    }
}

/// Pushes `length` bytes of `b` onto `set` and checks what the push did.
fn push_long(mut set: NameSet, length: usize) {
    let bytes = [b'b'; MAXIMUM_NAME_BYTES];
    let name = &bytes[..length];
    let fits = set.name_bytes() + length <= MAXIMUM_NAME_SET_BYTES;
    let held = set.n;
    let pushed = set.try_push(name, false).is_ok();
    assert_eq!(pushed, fits);
    assert_eq!(set.n, held + usize::from(pushed));
    assert!(is_bounded(&set));
    kani::cover!(
        pushed && set.name_bytes() == MAXIMUM_NAME_SET_BYTES,
        "a set filled to the last byte"
    );
}

/// Falsification: replayable `falsifications/expansion.verification.a_batch_is_the_front_of_the_byte_order.patch`
/// From a full first batch of every name, holding [`MAXIMUM_NAMES`] one-byte names from `a` up,
/// that has or has not left a name out already, one more offer: a name between two held names,
/// which goes in and pushes the largest out, or a name above every one, which is left out.
/// [`Expander::finish_batch`] answers the smallest names offered, in ascending byte order, so every
/// name it leaves out, whether the one offered or one it held, sorts after its last, and it says
/// there is more. So the next batch, which resumes after that last name, skips none.
#[kani::proof]
#[kani::unwind(10)]
fn a_batch_is_the_front_of_the_byte_order() {
    // Each batch and each offer is its own arm, as in [`push_any`], with every name spelled out
    // rather than looked up in a table, so each is a constant.
    match kani::any::<u8>() {
        0 => offer_one(false, b"cc", b"abcccdefg", [1, 2, 3, 5, 6, 7, 8, 9]),
        1 => offer_one(false, b"z", b"abcdefgh", [1, 2, 3, 4, 5, 6, 7, 8]),
        2 => offer_one(true, b"cc", b"abcccdefg", [1, 2, 3, 5, 6, 7, 8, 9]),
        _ => offer_one(true, b"z", b"abcdefgh", [1, 2, 3, 4, 5, 6, 7, 8]),
    }
}

/// Offers `name` to a full first batch of one-byte names from `a` up, which has left a name out
/// already when `more` says so, and checks that the batch then holds exactly the names `bytes`
/// spells, each ending where `ends` says, and says there is more. Each expected batch is listed in
/// ascending byte order, so matching it byte for byte checks the order too.
fn offer_one(more: bool, name: &[u8], bytes: &[u8], ends: [u16; MAXIMUM_NAMES]) {
    let mut expander = Expander::batch(b"*", Resume::Start);
    expander.set = held_names(MAXIMUM_NAMES);
    expander.more = more;
    expander.offer(name, false);
    let Ok(batch) = expander.finish_batch() else {
        panic!("a batch that holds names finishes");
    };
    let names = &batch.names;
    assert_eq!(names.n, MAXIMUM_NAMES);
    for (i, &end) in ends.iter().enumerate() {
        assert_eq!(names.ends[i], end);
    }
    for (i, &b) in bytes.iter().enumerate() {
        assert_eq!(names.bytes[i], b);
    }
    kani::cover!(names.ends[3] == 5, "a name displaces another");
    assert!(batch.more);
}
