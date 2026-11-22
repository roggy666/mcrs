#!/usr/bin/env bash
set -euo pipefail
dir=${1:?usage: summary.sh <folder of run reports>}
{
    echo "run,server,client,threads,mob,layout,phase,skipped,pigs_before,pigs_after,mspt_samples,mspt_mean,mspt_p50,mspt_p95,mspt_p99,mspt_worst,console_tps,console_ms_per_tick,watcher_set_time_p50,watcher_set_time_p95,watcher_set_time_p99,sprint_wall_ms,sprint_server_ms,motion_console_pigs,motion_console_moved,motion_console_displaced,motion_console_net_blocks,motion_console_path_bps,motion_watcher_pigs,motion_watcher_moved,motion_watcher_displaced,motion_watcher_net_blocks,motion_watcher_path_bps,stir_mode,stir_ticks,stir_lines,panic_lines,damage_answers,damage_answers_per_s,watcher_moves_per_s,busy_cores,server_cores,harness_cores,other_cores,rss_mib_mean,rss_mib_max,load_1m_max,contended,ai_windows,ai_mobs,ai_prepare_mean_us,ai_stage_mean_us,ai_stage_max_us,ai_wait_mean_us,ai_wait_max_us,ai_waits_in_windows,ai_lag_max,ai_evaluations_per_tick,ai_decisions_per_tick,ai_pops_per_tick,ai_deferred,ai_slices,ai_projection_sections,ai_projection_kib"
    for file in "$dir"/*.json; do
        jq -r --arg run "$(basename "$file" .json)" '
            . as $r | .phases[] | [
                $run, $r.server, $r.client, ($r.threads // ""), ($r.mob // "pig"), ($r.layout // "spread"), .phase, (.skipped // ""),
                .pigs_before, .pigs_after,
                .server_mspt.samples, .server_mspt.mean, .server_mspt.p50, .server_mspt.p95,
                .server_mspt.p99, .server_mspt.worst,
                .external.console_game_time.tps, .external.console_game_time.mean_ms_per_tick,
                .external.watcher_set_time.p50_ms, .external.watcher_set_time.p95_ms,
                .external.watcher_set_time.p99_ms,
                .external.sprint.wall_ms_per_tick, .external.sprint.server_ms_per_tick,
                .motion.console.pigs, .motion.console.moved_fraction,
                .motion.console.displaced_fraction, .motion.console.mean_net_blocks,
                .motion.console.mean_path_blocks_per_second,
                .motion.watcher.pigs, .motion.watcher.moved_fraction,
                .motion.watcher.displaced_fraction, .motion.watcher.mean_net_blocks,
                .motion.watcher.mean_path_blocks_per_second,
                .stir.mode, .stir.ticks_served, .stir.lines_sent, .stir.panic_lines_sent,
                .stir.damage_answers, .stir.damage_answers_per_second,
                .motion.watcher_move_packets_per_second,
                .cpu.busy_cores, .cpu.server_cores, .cpu.harness_cores, .cpu.other_cores,
                .server_rss_mib_mean, .server_rss_mib_max, .load_average_1m_max, .contended,
                .ai.windows, .ai.mobs, .ai.prepare_mean_us, .ai.stage_mean_us, .ai.stage_max_us,
                .ai.wait_mean_us, .ai.wait_max_us, .ai.waits_in_windows, .ai.lag_max,
                .ai.evaluations_per_tick, .ai.decisions_per_tick, .ai.pops_per_tick, .ai.deferred,
                .ai.slices, .ai.projection_sections, .ai.projection_kib
            ] | map(if . == null then "" else . end) | @csv' "$file"
    done
} >"$dir/summary.csv"
echo "$dir/summary.csv"
