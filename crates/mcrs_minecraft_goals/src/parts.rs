use mcrs_minecraft_brain::Part;

pub const FLOAT_JUMP: Part = Part(0x0100);
pub const PANIC_SEARCH: Part = Part(0x0200);
pub const STROLL_WAIT: Part = Part(0x0300);
pub const STROLL_SEARCH: Part = Part(0x0301);
pub const LOOK_AT_PLAYER_WAIT: Part = Part(0x0400);
pub const LOOK_AT_PLAYER_COUNT: Part = Part(0x0401);
pub const LOOK_AROUND_WAIT: Part = Part(0x0500);
pub const LOOK_AROUND_START: Part = Part(0x0501);
pub const EAT_WAIT: Part = Part(0x0600);
pub const MELEE_PATH: Part = Part(0x0700);
pub const LEAP_WAIT: Part = Part(0x0800);
pub const AVOID_SEARCH: Part = Part(0x0900);
pub const AVOID_STRENGTH: Part = Part(0x0901);
pub const BEG_TIME: Part = Part(0x0A00);
pub const TARGET_WAIT: Part = Part(0x0B00);
pub const OWNER_TELEPORT: Part = Part(0x0C00);
pub const FLEE_SUN_SEARCH: Part = Part(0x0D00);
pub const FLY_START: Part = Part(0x0E00);
pub const WANDER_WAIT: Part = Part(0x0F00);
pub const WANDER_SEARCH: Part = Part(0x0F01);

#[must_use]
pub const fn slot_part(part: Part, slot: usize) -> Part {
    Part(part.0 + (slot as u32 & 0xFF) * 0x10_000)
}
