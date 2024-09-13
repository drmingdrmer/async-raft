use std::sync::Arc;

use maplit::btreeset;
use pretty_assertions::assert_eq;

use crate::engine::testing::UTConfig;
use crate::engine::Command;
use crate::engine::Engine;
use crate::engine::LogIdList;
use crate::entry::RaftEntry;
use crate::progress::entry::ProgressEntry;
use crate::progress::Inflight;
use crate::testing::log_id;
use crate::utime::UTime;
use crate::EffectiveMembership;
use crate::Entry;
use crate::Membership;
use crate::ServerState;
use crate::TokioInstant;
use crate::Vote;

fn m_empty() -> Membership<u64, ()> {
    Membership::<u64, ()>::new(vec![btreeset! {}], None)
}

fn m23() -> Membership<u64, ()> {
    Membership::<u64, ()>::new(vec![btreeset! {2,3}], None)
}

fn m34() -> Membership<u64, ()> {
    Membership::<u64, ()>::new(vec![btreeset! {3,4}], None)
}

fn eng() -> Engine<UTConfig> {
    let mut eng = Engine::testing_default(0);
    eng.state.enable_validation(false);
    eng.config.id = 2;
    // This will be overridden
    eng.state.server_state = ServerState::default();
    eng
}

/// It is a Leader but not yet append any logs.
#[test]
fn test_startup_as_leader_without_logs() -> anyhow::Result<()> {
    let mut eng = eng();
    // self.id==2 is a voter:
    eng.state
        .membership_state
        .set_effective(Arc::new(EffectiveMembership::new(Some(log_id(2, 1, 3)), m23())));
    // Committed vote makes it a leader at startup.
    eng.state.vote = UTime::new(TokioInstant::now(), Vote::new_committed(1, 2));

    eng.startup();

    assert_eq!(ServerState::Leader, eng.state.server_state);
    assert_eq!(
        vec![
            //
            Command::BecomeLeader,
            Command::RebuildReplicationStreams {
                targets: vec![(3, ProgressEntry {
                    matching: None,
                    curr_inflight_id: 0,
                    inflight: Inflight::None,
                    searching_end: 0
                })]
            },
            Command::AppendInputEntries {
                vote: Vote::new_committed(1, 2),
                entries: vec![Entry::<UTConfig>::new_blank(log_id(1, 2, 0))],
            },
            Command::Replicate {
                target: 3,
                req: Inflight::logs(None, Some(log_id(1, 2, 0))).with_id(1)
            }
        ],
        eng.output.take_commands()
    );

    Ok(())
}

#[test]
fn test_startup_as_leader_with_proposed_logs() -> anyhow::Result<()> {
    tracing::info!("--- a leader proposed logs and restarted, reuse noop_log_id");
    let mut eng = eng();
    // self.id==2 is a voter:
    eng.state
        .membership_state
        .set_effective(Arc::new(EffectiveMembership::new(Some(log_id(2, 1, 3)), m23())));
    // Fake existing log ids
    eng.state.log_ids = LogIdList::new([log_id(1, 1, 2), log_id(1, 2, 4), log_id(1, 2, 6)]);
    // Committed vote makes it a leader at startup.
    eng.state.vote = UTime::new(TokioInstant::now(), Vote::new_committed(1, 2));

    eng.startup();

    assert_eq!(ServerState::Leader, eng.state.server_state);
    assert_eq!(eng.leader.as_ref().unwrap().noop_log_id(), Some(&log_id(1, 2, 4)));
    assert_eq!(eng.leader.as_ref().unwrap().last_log_id(), Some(&log_id(1, 2, 6)));
    assert_eq!(
        vec![
            //
            Command::BecomeLeader,
            Command::RebuildReplicationStreams {
                targets: vec![(3, ProgressEntry {
                    matching: None,
                    curr_inflight_id: 0,
                    inflight: Inflight::None,
                    searching_end: 7
                })]
            },
            Command::Replicate {
                target: 3,
                req: Inflight::logs(None, Some(log_id(1, 2, 6))).with_id(1)
            }
        ],
        eng.output.take_commands()
    );

    Ok(())
}

/// When starting up, a leader that is not a voter should not panic.
#[test]
fn test_startup_as_leader_not_voter_issue_920() -> anyhow::Result<()> {
    let mut eng = eng();
    // self.id==2 is a voter:
    eng.state
        .membership_state
        .set_effective(Arc::new(EffectiveMembership::new(Some(log_id(2, 1, 3)), m_empty())));
    // Committed vote makes it a leader at startup.
    eng.state.vote = UTime::new(TokioInstant::now(), Vote::new_committed(1, 2));

    eng.startup();

    assert_eq!(ServerState::Learner, eng.state.server_state);
    assert_eq!(eng.output.take_commands(), vec![]);

    Ok(())
}

#[test]
fn test_startup_candidate_becomes_follower() -> anyhow::Result<()> {
    let mut eng = eng();
    // self.id==2 is a voter:
    eng.state
        .membership_state
        .set_effective(Arc::new(EffectiveMembership::new(Some(log_id(2, 1, 3)), m23())));
    // Non-committed vote makes it a candidate at startup.
    eng.state.vote = UTime::new(TokioInstant::now(), Vote::new(1, 2));

    eng.startup();

    assert_eq!(ServerState::Follower, eng.state.server_state);
    assert_eq!(0, eng.output.take_commands().len());

    Ok(())
}
#[test]
fn test_startup_as_follower() -> anyhow::Result<()> {
    let mut eng = eng();
    // self.id==2 is a voter:
    eng.state
        .membership_state
        .set_effective(Arc::new(EffectiveMembership::new(Some(log_id(2, 1, 3)), m23())));

    eng.startup();

    assert_eq!(ServerState::Follower, eng.state.server_state);
    assert_eq!(0, eng.output.take_commands().len());

    Ok(())
}

#[test]
fn test_startup_as_learner() -> anyhow::Result<()> {
    let mut eng = eng();
    // self.id==2 is not a voter:
    eng.state
        .membership_state
        .set_effective(Arc::new(EffectiveMembership::new(Some(log_id(2, 1, 3)), m34())));

    eng.startup();

    assert_eq!(ServerState::Learner, eng.state.server_state);
    assert_eq!(0, eng.output.take_commands().len());

    Ok(())
}
