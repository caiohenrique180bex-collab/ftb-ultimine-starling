//! Testable core for Ultimine. The runtime stays dependency-free.

use std::collections::{HashSet, VecDeque};

/// Integer block coordinate in the world.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BlockPos(pub [i32; 3]);

impl BlockPos {
    fn neighbors(self) -> [Self; 6] {
        let [x, y, z] = self.0;
        [
            Self([x + 1, y, z]),
            Self([x - 1, y, z]),
            Self([x, y + 1, z]),
            Self([x, y - 1, z]),
            Self([x, y, z + 1]),
            Self([x, y, z - 1]),
        ]
    }

    fn within_radius(self, origin: Self, radius: u32) -> bool {
        let [x, y, z] = self.0;
        let [ox, oy, oz] = origin.0;
        x.abs_diff(ox).max(y.abs_diff(oy)).max(z.abs_diff(oz)) <= radius
    }
}

/// Limits applied to one connected-block selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionLimits {
    pub radius: u32,
    pub max_blocks: usize,
}

impl Default for SelectionLimits {
    fn default() -> Self {
        Self {
            radius: 8,
            max_blocks: 64,
        }
    }
}

/// Selects a deterministic, six-face-connected group matching the origin's
/// block key. The caller supplies a read-only lookup; no world mutation occurs.
pub fn connected_blocks<F>(
    origin: BlockPos,
    block_key_at: F,
    limits: SelectionLimits,
) -> Vec<BlockPos>
where
    F: Fn(BlockPos) -> Option<u32>,
{
    if limits.max_blocks == 0 {
        return Vec::new();
    }
    let Some(target_key) = block_key_at(origin) else {
        return Vec::new();
    };

    let mut queue = VecDeque::from([origin]);
    let mut visited = HashSet::from([origin]);
    let mut selected = Vec::with_capacity(limits.max_blocks.min(64));

    while let Some(pos) = queue.pop_front() {
        if selected.len() >= limits.max_blocks {
            break;
        }
        if block_key_at(pos) != Some(target_key) {
            continue;
        }
        selected.push(pos);

        for next in pos.neighbors() {
            if next.within_radius(origin, limits.radius) && visited.insert(next) {
                queue.push_back(next);
            }
        }
    }
    selected
}

/// Called by the native break hook. The actual multi-break action is deliberately
/// not enabled until the game-side tool, drop and server contracts are wired.
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
    fn selects_only_face_connected_matching_blocks() {
        let ore = HashSet::from([
            BlockPos([0, 0, 0]),
            BlockPos([1, 0, 0]),
            BlockPos([1, 1, 0]),
            BlockPos([4, 0, 0]),
        ]);
        let selected = connected_blocks(
            BlockPos([0, 0, 0]),
            |p| ore.contains(&p).then_some(7),
            SelectionLimits { radius: 8, max_blocks: 64 },
        );
        assert_eq!(
            selected,
            vec![BlockPos([0, 0, 0]), BlockPos([1, 0, 0]), BlockPos([1, 1, 0])]
        );
    }

    #[test]
    fn enforces_radius_and_block_cap() {
        let selected = connected_blocks(
            BlockPos([0, 0, 0]),
            |_| Some(1),
            SelectionLimits { radius: 1, max_blocks: 3 },
        );
        assert_eq!(selected.len(), 3);
        assert!(selected.iter().all(|p| p.within_radius(BlockPos([0, 0, 0]), 1)));
    }

    #[test]
    fn air_origin_and_zero_cap_select_nothing() {
        let origin = BlockPos([0, 0, 0]);
        assert!(connected_blocks(origin, |_| None, SelectionLimits::default()).is_empty());
        assert!(connected_blocks(
            origin,
            |_| Some(1),
            SelectionLimits { radius: 8, max_blocks: 0 },
        ).is_empty());
    }
}
