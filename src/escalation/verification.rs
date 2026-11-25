//! Machine-checked proofs, with Kani, of the escalation policy's promise to a shell: a job that
//! ignores the cooperative interrupt is torn down without a second keystroke, and torn down once.

use super::*;

/// Whether the policy can be in state `e`: before any interrupt it holds nothing; once the job is
/// asked to stop and until it is torn down, one interrupt is counted and between one and
/// [`COOPERATIVE_GRACE_POLLS`] ticks of the window are left; a torn down job was asked first.
fn is_reachable(e: &Escalation) -> bool {
    if !e.coop_sent {
        e.interrupts == 0 && !e.done && e.grace_left == 0
    } else if !e.done {
        e.interrupts == 1 && e.grace_left >= 1 && e.grace_left <= COOPERATIVE_GRACE_POLLS
    } else {
        e.interrupts == 1 || e.interrupts == 2
    }
}

/// Falsification: replayable `falsifications/escalation.verification.the_first_interrupt_brings_one_teardown_within_the_grace_window.patch`
/// By induction over sequences of interrupts and ticks of any length: a fresh policy is in a
/// reachable state, and from any reachable state an interrupt or a tick leads to another. On the
/// way, the first interrupt alone asks the job to stop and arms a window of
/// [`COOPERATIVE_GRACE_POLLS`] ticks; while the window is open, each tick runs it down by one and
/// the tick that runs it out, or a second interrupt, tears the job down; a teardown comes only
/// after the job was asked to stop and before nothing else, so it comes once, within the window.
/// [`Escalation::is_spent`] turns true exactly with the teardown.
#[kani::proof]
fn the_first_interrupt_brings_one_teardown_within_the_grace_window() {
    assert!(is_reachable(&Escalation::new()));
    let mut e = Escalation {
        interrupts: kani::any(),
        grace_left: kani::any(),
        coop_sent: kani::any(),
        done: kani::any(),
    };
    kani::assume(is_reachable(&e));
    let before = e;
    let interrupt: bool = kani::any();
    let action = if interrupt {
        e.on_interrupt()
    } else {
        e.on_tick()
    };
    assert!(is_reachable(&e));
    let window_open = before.coop_sent && !before.done;
    match action {
        InterruptAction::Cooperative => {
            assert!(
                interrupt && !before.coop_sent,
                "the job is asked to stop once"
            );
            assert_eq!(e.grace_left, COOPERATIVE_GRACE_POLLS);
        }
        InterruptAction::Forcible => {
            assert!(
                window_open,
                "a teardown comes once, after the job was asked to stop"
            );
            assert!(interrupt || before.grace_left == 1);
        }
        InterruptAction::None => {
            if window_open {
                assert!(!interrupt && e.grace_left == before.grace_left - 1);
            } else {
                assert_eq!(e, before);
            }
        }
    }
    assert_eq!(
        e.is_spent(),
        before.is_spent() || action == InterruptAction::Forcible
    );
    kani::cover!(
        window_open && !interrupt && action == InterruptAction::Forcible,
        "the window runs out"
    );
}
