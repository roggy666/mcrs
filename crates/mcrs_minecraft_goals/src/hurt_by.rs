use mcrs_minecraft_brain::EntityRef;
use mcrs_minecraft_brain::KindSet;
use mcrs_minecraft_brain::Proposal;
use mcrs_minecraft_brain::Receivers;
use mcrs_minecraft_goal_world::GoalWorld;

use crate::Alert;
use crate::HurtByState;
use crate::Sight;
use crate::nearest_target::base_continue;
use crate::query::Combat;
use crate::query::Cx;
use crate::query::Ray;
use crate::query::Visibility;

const UNSEEN_LIMIT: u8 = 150;

pub(crate) fn can_start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    ignored: &KindSet,
    state: &HurtByState,
) -> bool {
    let Some((attacker, tick)) = cx.me().hurt_by.get() else {
        return false;
    };
    if tick == state.kept_tick {
        return false;
    }
    cx.busy();
    let Some(candidate) = cx.find(attacker) else {
        return false;
    };
    if candidate.kind.is_some_and(|kind| ignored.contains(kind)) {
        return false;
    }
    cx.combat_test(
        &candidate,
        Combat {
            range: 0.0,
            ray: Ray::Skip,
            visibility: Visibility::Ignore,
        },
    )
}

pub(crate) fn start<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    alert: Alert,
    state: &mut HurtByState,
) {
    let Some((attacker, tick)) = cx.me().hurt_by.get() else {
        return;
    };
    cx.set_target(Some(attacker));
    state.goal_target = EntityRef::of(attacker);
    state.kept_tick = tick;
    state.unseen = 0;
    let receivers = match alert {
        Alert::None => None,
        Alert::SameKind => Some(Receivers::SameKind),
        Alert::Kinds(kinds) => Some(Receivers::Kinds(kinds)),
    };
    if let Some(receivers) = receivers {
        cx.intents.proposal = Some(Proposal::Alert {
            target: attacker,
            reach: cx.me().follow_range,
            receivers,
        });
    }
}

pub(crate) fn can_continue<W: GoalWorld + ?Sized>(
    cx: &mut Cx<'_, '_, '_, W>,
    state: &mut HurtByState,
) -> bool {
    cx.busy();
    let follow = cx.follow_range();
    base_continue(
        cx,
        Sight::MustSee,
        follow,
        &mut state.unseen,
        UNSEEN_LIMIT,
        state.goal_target,
    )
}

pub(crate) fn stop<W: GoalWorld + ?Sized>(cx: &mut Cx<'_, '_, '_, W>, state: &mut HurtByState) {
    cx.set_target(None);
    state.goal_target = EntityRef::NONE;
}
