//! Machine-checked proofs, with Kani, that splitting a line on its operators never reads past it.

use super::*;

/// Falsification: replayable `falsifications/line.verification.a_split_never_indexes_past_the_line.patch`
/// For the empty line and every line of one byte, a byte the splitter treats specially (a space,
/// `|`, `<`, `>`, `2`, either quote, the pattern byte `*`) or a letter standing for every other
/// byte, [`split`] returns without panicking: no index, no subtraction and no assertion inside it
/// fails, so an operator with nothing after it is refused rather than read past. A line it accepts
/// has between one and [`MAXIMUM_STAGES`] stages. A longer line costs more than the proof budget
/// allows; the host tests cover those.
#[kani::proof]
#[kani::unwind(4)]
fn a_split_never_indexes_past_the_line() {
    let mut line = [0u8; 1];
    for b in &mut line {
        let k: usize = kani::any();
        kani::assume(k < ALPHABET.len());
        *b = ALPHABET[k];
    }
    let length_bytes: usize = kani::any();
    kani::assume(length_bytes <= line.len());
    check_split(&line[..length_bytes]);
}

/// The bytes a line is drawn from: a letter, standing for every byte the splitter does not treat
/// specially, then every byte it does.
const ALPHABET: [u8; 9] = *b"a |<>2'\"*";

/// Splits `line` and checks what comes back.
fn check_split(line: &[u8]) {
    let Ok(l) = split(line) else {
        kani::cover!(line == b">", "an operator with nothing after it is refused");
        return;
    };
    assert!(l.stage_count() >= 1 && l.stage_count() <= MAXIMUM_STAGES);
    kani::cover!(l.stage_count() == 1, "a command");
}
