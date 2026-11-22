use std::sync::LazyLock;

use mcrs_minecraft_block_support::{BlockConnections, ConnectionIssue};
use mcrs_minecraft_core::Direction;
use mcrs_minecraft_game_registries::registry_load::test_registries;
use mcrs_voxel::{BlockPos, BlockStateId};

use crate::block_support::{definitions, state};

const P: BlockPos = BlockPos::new(0, 64, 0);
const FIXTURE: &str = include_str!("../fixtures/vanilla/connections.txt");

static CONNECTIONS: LazyLock<(BlockConnections, Vec<ConnectionIssue>)> =
    LazyLock::new(|| BlockConnections::bake(definitions(), test_registries()));

fn connections() -> &'static BlockConnections {
    &CONNECTIONS.0
}

struct Neighbourhood {
    placed: Vec<(Direction, BlockStateId)>,
}

impl Neighbourhood {
    fn read(&self) -> impl FnMut(BlockPos) -> BlockStateId + '_ {
        move |pos| {
            self.placed
                .iter()
                .find(|(direction, _)| P + direction.normal() == pos)
                .map_or(BlockStateId::AIR, |(_, state)| *state)
        }
    }

    fn put(&mut self, direction: Direction, block: BlockStateId) {
        self.placed.retain(|(placed, _)| *placed != direction);
        self.placed.push((direction, block));
    }
}

fn direction(name: &str) -> Direction {
    match name {
        "north" => Direction::North,
        "east" => Direction::East,
        "south" => Direction::South,
        "west" => Direction::West,
        "up" | "above" => Direction::Up,
        "down" | "below" => Direction::Down,
        other => panic!("{other} is not a direction"),
    }
}

fn candidate(text: &str) -> BlockStateId {
    let text = text.split(" (").next().unwrap_or(text);
    let text = text.split(" +").next().unwrap_or(text).trim();
    if let Some(properties) = text.strip_prefix('[') {
        return state(&format!("cobblestone_wall[{properties}"));
    }
    state(text)
}

fn result(text: &str) -> BlockStateId {
    let text = text.split(" ; ").next().unwrap_or(text);
    let text = text.split(" (").next().unwrap_or(text).trim();
    let text = text.strip_prefix("wall ").unwrap_or(text);
    state(text)
}

fn steps(text: &str) -> Vec<(Direction, BlockStateId)> {
    let mut steps = Vec::new();
    for part in text.split(", ") {
        let Some((direction, block)) = part.split_once(": ") else {
            continue;
        };
        if direction == "sides(walls)" {
            steps.extend(
                block
                    .split(',')
                    .filter(|side| *side != "none")
                    .map(|side| (self::direction(side), state("cobblestone_wall"))),
            );
        } else {
            steps.push((self::direction(direction), candidate(block)));
        }
    }
    steps
}

fn apply(subject: BlockStateId, steps: &[(Direction, BlockStateId)]) -> BlockStateId {
    let mut neighbourhood = Neighbourhood { placed: Vec::new() };
    let mut current = subject;
    for (direction, block) in steps {
        neighbourhood.put(*direction, *block);
        current = connections().after_update(current, P, *direction, *block, neighbourhood.read());
    }
    current
}

fn describe(state: BlockStateId) -> String {
    definitions().properties_of(state.number()).map_or_else(
        || format!("{state:?}"),
        |(name, properties)| {
            let properties: Vec<String> = properties
                .iter()
                .map(|(property, value)| format!("{property}={value}"))
                .collect();
            format!("{name}[{}]", properties.join(","))
        },
    )
}

#[test]
fn the_connection_rules_read_no_tag_the_data_pack_lacks() {
    assert_eq!(CONNECTIONS.1, Vec::new());
}

#[test]
fn each_record_of_the_vanilla_fixture_gives_the_recorded_state() {
    let mut checked = 0;
    let mut wrong = Vec::new();
    for line in FIXTURE.lines() {
        let parts: Vec<&str> = line.split(" | ").map(str::trim).collect();
        let Some((head, rest)) = parts.split_first() else {
            continue;
        };
        let Some((kind, subject)) = head.split_once(' ') else {
            continue;
        };
        if !matches!(kind, "fence" | "pane" | "bars" | "wall" | "gate") || subject == "column" {
            continue;
        }
        let Some((step, outcomes)) = rest.split_first() else {
            continue;
        };
        if step.contains("after") || step.contains("/setblock") {
            continue;
        }
        let (step, removed) = match step.strip_suffix(" -> removed") {
            Some(step) => (step, true),
            None => (*step, false),
        };
        if step.contains("->") {
            continue;
        }
        let mut steps = steps(step);
        if steps.is_empty() {
            continue;
        }
        if removed {
            let (direction, _) = steps[0];
            steps.push((direction, BlockStateId::AIR));
        }
        let subject = state(subject);
        let (replayed, expected) = match outcomes {
            [after] if !after.contains("->") => (apply(subject, &steps), result(after)),
            [before, after] if before.starts_with("before ") && after.starts_with("after ") => {
                let (sides, _) = steps.split_at(steps.len() - 1);
                let built = apply(subject, sides);
                assert_eq!(
                    describe(built),
                    describe(result(&before["before ".len()..])),
                    "the sides of {line}"
                );
                (apply(built, &steps), result(&after["after ".len()..]))
            }
            [placed, removed] if removed.starts_with("removed -> ") => {
                let with = apply(subject, &steps);
                let (direction, _) = steps[0];
                let without = apply(with, &[(direction, BlockStateId::AIR)]);
                checked += 1;
                if describe(without) != describe(result(&removed["removed -> ".len()..])) {
                    wrong.push(format!("{line}\n    removal gave {}", describe(without)));
                }
                (with, result(placed))
            }
            _ => continue,
        };
        checked += 1;
        if describe(replayed) != describe(expected) {
            wrong.push(format!("{line}\n    gave {}", describe(replayed)));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {checked} records differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    assert!(checked > 1400, "only {checked} records were checked");
}
