//! Machine-checked proofs, with Kani, of what reading a token's quotes promises: it never panics,
//! it never rewrites a byte, and a word is quoted only when one pair of quotes wraps all of it.

use super::*;

/// Falsification: replayable `falsifications/word.verification.a_word_is_a_slice_of_its_token_and_quoted_only_when_wrapped.patch`
/// For any token of up to five bytes, [`read`] returns without panicking. A word it answers is a
/// slice of the token. A quoted word is the token less its two end bytes, which are the same quote
/// character, and that quote appears nowhere between them. A bare word is the whole token, and the
/// token carries no quote byte at all.
#[kani::proof]
#[kani::unwind(7)]
fn a_word_is_a_slice_of_its_token_and_quoted_only_when_wrapped() {
    const LENGTH_BYTES: usize = 5;
    let bytes: [u8; LENGTH_BYTES] = kani::any();
    let length: usize = kani::any();
    kani::assume(length <= LENGTH_BYTES);
    let token = &bytes[..length];
    let Ok(w) = read(token) else {
        return;
    };
    let base = token.as_ptr() as usize;
    let at = w.text.as_ptr() as usize;
    assert!(at >= base && at + w.text.len() <= base + token.len());
    if w.quoted {
        kani::cover!(true, "a quoted word");
        assert!(token.len() >= 2);
        let quote = token[0];
        assert!(quote == SINGLE || quote == DOUBLE);
        assert_eq!(token[token.len() - 1], quote);
        assert_eq!(w.text, &token[1..token.len() - 1]);
        assert!(!w.text.contains(&quote));
    } else {
        assert_eq!(w.text, token);
        assert!(!token.contains(&SINGLE) && !token.contains(&DOUBLE));
    }
}
