use bevy_tasks::TaskPool;
use mcrs_minecraft_brain::MobBrain;
use mcrs_minecraft_brain::TickContext;
use mcrs_minecraft_brain::TileWork;
use mcrs_minecraft_brain::TurnScratch;
use mcrs_minecraft_goal_world::GoalWorld;

pub(crate) fn run_tiles<B, W>(
    pool: &TaskPool,
    work: &mut [TileWork<'_, B>],
    cx: &TickContext<'_, W>,
) where
    B: MobBrain<W> + Send,
    W: GoalWorld + Sync,
{
    let tasks = pool.thread_num().saturating_mul(4).max(1);
    let total: usize = work.iter().map(TileWork::cost).sum();
    let share = total.div_ceil(tasks).max(1);
    pool.scope(|scope| {
        let mut rest = work;
        while !rest.is_empty() {
            let mut cost = 0;
            let mut count = 0;
            for task in rest.iter() {
                if count > 0 && cost + task.cost() > share {
                    break;
                }
                cost += task.cost();
                count += 1;
            }
            let (chunk, tail) = rest.split_at_mut(count);
            rest = tail;
            scope.spawn(async move {
                let mut scratch = TurnScratch::default();
                for task in chunk {
                    task.run(cx, &mut scratch);
                }
            });
        }
    });
}
