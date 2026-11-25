//! Machine-checked proofs, with Kani, of the tokenizer's bounds: every token is a slice of the line,
//! and the caller's array is never overrun.

use super::*;

/// Falsification: replayable `falsifications/parsing.verification.every_token_is_a_nonempty_slice_of_the_line_in_order.patch`
/// For any line of up to six bytes and an array of three slots, the tokenizer behind [`tokenize`] yields at most three
/// tokens. Each is non-empty and a slice of the line, and each starts after the one before it ends.
/// When the array was not filled, the tokenizer reached the end of the line, so nothing was left
/// unread.
#[kani::proof]
#[kani::unwind(8)]
fn every_token_is_a_nonempty_slice_of_the_line_in_order() {
    const LENGTH_BYTES: usize = 6;
    let bytes: [u8; LENGTH_BYTES] = kani::any();
    let length: usize = kani::any();
    kani::assume(length <= LENGTH_BYTES);
    let line = &bytes[..length];
    let mut out: [&[u8]; 3] = [&[]; 3];
    let (tokens, rest) = split_tokens(line, &mut out);
    assert!(tokens.len() <= 3);
    let base = line.as_ptr() as usize;
    let mut floor = base;
    for t in tokens {
        assert!(!t.is_empty());
        let at = t.as_ptr() as usize;
        assert!(at >= floor && at + t.len() <= base + line.len());
        floor = at + t.len();
    }
    if tokens.len() < 3 {
        assert!(rest.is_empty());
    }
    kani::cover!(tokens.len() == 3, "the array filled");
}
