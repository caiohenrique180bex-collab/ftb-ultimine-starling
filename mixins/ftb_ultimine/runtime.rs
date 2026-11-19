// Testable core for Ultimine. Runtime logic is dependency-free.

use std::collections::{HashSet, VecDeque};

/// Integer block coordinate in the world.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlockPos(pub [i32; 3]);

impl BlockPos {
    fn neighbors(self) -> [Option<Self>; 6] {
        let [x, y, z] = self.0;
        [
            x.checked_add(1).map(|v| Self([v, y, z])),
            x.checked_sub(1).map(|v| Self([v, y, z])),
            y.checked_add(1).map(|v| Self([x, v, z])),
            y.checked_sub(1).map(|v| Self([x, v, z])),
            z.checked_add(1).map(|v| Self([x, y, v])),
            z.checked_sub(1).map(|v| Self([x, y, v])),
        ]
    }

    fn within_radius(self, origin: Self, radius: u32) -> bool {
        let [x, y, z] = self.0;
        let [ox, oy, oz] = origin.0;
        x.abs_diff(ox).max(y.abs_diff(oy)).max(z.abs_diff(oz)) <= radius
    }
}

/// Limits applied to one selection. The hard cap prevents accidental huge scans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionLimits {
    pub radius: u32,
    pub max_blocks: usize,
}

impl Default for SelectionLimits {
    fn default() -> Self {
        Self { radius: 8, max_blocks: 64 }
    }
}

impl SelectionLimits {
    pub const HARD_MAX_BLOCKS: usize = 256;

    fn cap(self) -> usize {
        self.max_blocks.min(Self::HARD_MAX_BLOCKS)
    }
}

/// Selects a deterministic, six-face-connected group matching the origin key.
/// The lookup is read-only: this function never mutates the world.
pub fn connected_blocks<F>(
    origin: BlockPos,
    block_key_at: F,
    limits: SelectionLimits,
) -> Vec<BlockPos>
where
    F: Fn(BlockPos) -> Option<u32>,
{
    let cap = limits.cap();
    if cap == 0 {
        return Vec::new();
    }
    let Some(target_key) = block_key_at(origin) else {
        return Vec::new();
    };

    let mut queue = VecDeque::from([origin]);
    let mut visited = HashSet::from([origin]);
    let mut selected = Vec::with_capacity(cap.min(64));

    while let Some(pos) = queue.pop_front() {
        if selected.len() >= cap {
            break;
        }
        if block_key_at(pos) != Some(target_key) {
            continue;
        }
        selected.push(pos);

        for next in pos.neighbors().into_iter().flatten() {
            if next.within_radius(origin, limits.radius) && visited.insert(next) {
                queue.push_back(next);
            }
        }
    }
    selected
}

/// Ultimine modes supported by the pure selection core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MiningMode {
    /// Connected blocks with the same block key (ores, stone variants, etc.).
    Vein,
    /// Connected log blocks of the same wood family. Leaves are intentionally
    /// excluded; callers must classify only log blocks as matching keys.
    Tree,
    /// A cuboid around the target, useful for tunnels and area excavation.
    Excavation,
}

/// Dimensions of an excavation cuboid. Each dimension is clamped to a safe size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AreaSize {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Default for AreaSize {
    fn default() -> Self {
        Self { width: 3, height: 3, depth: 1 }
    }
}

/// Select a vein. The lookup should return a key only for blocks eligible for
/// vein mining; equal keys are considered the same block family.
pub fn select_vein<F>(origin: BlockPos, key_at: F, limits: SelectionLimits) -> Vec<BlockPos>
where
    F: Fn(BlockPos) -> Option<u32>,
{
    connected_blocks(origin, key_at, limits)
}

/// Select connected logs. The lookup must return a key only for logs, never
/// leaves, roots, or decorative blocks.
pub fn select_tree<F>(origin: BlockPos, log_family_at: F, limits: SelectionLimits) -> Vec<BlockPos>
where
    F: Fn(BlockPos) -> Option<u32>,
{
    connected_blocks(origin, log_family_at, limits)
}

/// Select eligible blocks inside a cuboid centered on the target. The closure
/// decides which world blocks are non-air and safe to mine. Results are sorted
/// by distance, then coordinates, so the selection is deterministic.
pub fn select_excavation<F>(
    origin: BlockPos,
    eligible_at: F,
    size: AreaSize,
    max_blocks: usize,
) -> Vec<BlockPos>
where
    F: Fn(BlockPos) -> bool,
{
    let width = size.width.clamp(1, 9) as i32;
    let height = size.height.clamp(1, 9) as i32;
    let depth = size.depth.clamp(1, 9) as i32;
    let cap = max_blocks.min(SelectionLimits::HARD_MAX_BLOCKS);
    if cap == 0 {
        return Vec::new();
    }

    let start_x = -(width / 2);
    let start_y = -(height / 2);
    let start_z = -(depth / 2);
    let mut selected = Vec::with_capacity(cap.min((width * height * depth) as usize));

    for dx in start_x..(start_x + width) {
        for dy in start_y..(start_y + height) {
            for dz in start_z..(start_z + depth) {
                let Some(x) = origin.0[0].checked_add(dx) else { continue };
                let Some(y) = origin.0[1].checked_add(dy) else { continue };
                let Some(z) = origin.0[2].checked_add(dz) else { continue };
                let pos = BlockPos([x, y, z]);
                if eligible_at(pos) {
                    selected.push(pos);
                }
            }
        }
    }

    selected.sort_by_key(|p| {
        let [x, y, z] = p.0;
        let [ox, oy, oz] = origin.0;
        (
            x.abs_diff(ox) + y.abs_diff(oy) + z.abs_diff(oz),
            p.0,
        )
    });
    selected.truncate(cap);
    selected
}

/// Called by the native break hook. Multi-break is not activated until the
/// native input, permission, and re-entrancy contracts are wired.
pub fn on_break_attempt(pos: [i32; 3]) {
    eprintln!(
        "[ftb_ultimine] break hook reached at {}, {}, {}",
        pos[0], pos[1], pos[2]
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vein_selection_matches_family_and_face_connectivity() {
        let ore = HashSet::from([
            BlockPos([0, 0, 0]),
            BlockPos([1, 0, 0]),
            BlockPos([1, 1, 0]),
            BlockPos([4, 0, 0]),
        ]);
        let selected = select_vein(
            BlockPos([0, 0, 0]),
            |p| ore.contains(&p).then_some(7),
            SelectionLimits { radius: 8, max_blocks: 64 },
        );
        assert_eq!(selected, vec![
            BlockPos([0, 0, 0]),
            BlockPos([1, 0, 0]),
            BlockPos([1, 1, 0])
        ]);
    }

    #[test]
    fn tree_selection_uses_only_caller_classified_logs() {
        let logs = HashSet::from([
            BlockPos([0, 0, 0]),
            BlockPos([0, 1, 0]),
            BlockPos([0, 2, 0]),
            BlockPos([1, 2, 0]),
        ]);
        let selected = select_tree(
            BlockPos([0, 0, 0]),
            |p| logs.contains(&p).then_some(3),
            SelectionLimits::default(),
        );
        assert_eq!(selected.len(), 4);
        assert!(!selected.contains(&BlockPos([1, 1, 0])));
    }

    #[test]
    fn excavation_selects_eligible_area_in_deterministic_order() {
        let origin = BlockPos([0, 0, 0]);
        let selected = select_excavation(
            origin,
            |p| p.0[0].abs() <= 1 && p.0[1] == 0 && p.0[2] == 0,
            AreaSize::default(),
            64,
        );
        assert_eq!(selected, vec![
            BlockPos([0, 0, 0]),
            BlockPos([-1, 0, 0]),
            BlockPos([1, 0, 0]),
        ]);
    }

    #[test]
    fn hard_cap_and_radius_are_enforced() {
        let selected = connected_blocks(
            BlockPos([0, 0, 0]),
            |_| Some(1),
            SelectionLimits { radius: 1, max_blocks: 10_000 },
        );
        assert_eq!(selected.len(), 27);
        assert!(selected.iter().all(|p| p.within_radius(BlockPos([0, 0, 0]), 1)));
    }

    #[test]
    fn coordinate_overflow_is_ignored_safely() {
        let origin = BlockPos([i32::MAX, 0, 0]);
        let selected = connected_blocks(origin, |_| Some(1), SelectionLimits::default());
        assert!(selected.contains(&origin));
        assert!(!selected.contains(&BlockPos([i32::MIN, 0, 0])));
    }

    #[test]
    fn empty_origin_and_zero_cap_select_nothing() {
        let origin = BlockPos([0, 0, 0]);
        assert!(connected_blocks(origin, |_| None, SelectionLimits::default()).is_empty());
        assert!(connected_blocks(
            origin,
            |_| Some(1),
            SelectionLimits { radius: 8, max_blocks: 0 },
        ).is_empty());
    }
}
