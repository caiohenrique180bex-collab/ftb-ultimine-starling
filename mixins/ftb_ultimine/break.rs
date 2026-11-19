// Native gameplay bridge. Every extra block is routed through Game::break_block,
// so Horizon remains responsible for permissions, drops, XP, tool wear and events.
fn before_break(game: &mut Game<'_>, pos: [i32; 3]) {
    use crate::game::ftb_ultimine::{
        block_name_key, multibreak_is_active, select_excavation, select_tree, select_vein,
        tree_family_key, with_multibreak_guard, AreaSize, BlockPos, SelectionLimits,
    };

    if multibreak_is_active() { return; }
    let shift_down = game.keys.contains(&KeyCode::ShiftLeft) || game.keys.contains(&KeyCode::ShiftRight);
    if !shift_down { return; }

    let origin = BlockPos(pos);
    let origin_state = game.world.block(pos[0], pos[1], pos[2]);
    let origin_name = game.res.facts().get(origin_state).name.clone();
    if matches!(origin_name.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") { return; }
    let Some(eye) = game.eye_position() else { return; };
    let reach = game.reach();
    let within_reach = |candidate: BlockPos| {
        let [x, y, z] = candidate.0;
        let dx = f64::from(x) + 0.5 - eye[0];
        let dy = f64::from(y) + 0.5 - eye[1];
        let dz = f64::from(z) + 0.5 - eye[2];
        dx * dx + dy * dy + dz * dz <= reach * reach
    };
    let limits = SelectionLimits { radius: 4, max_blocks: 64 };
    let control_down = game.keys.contains(&KeyCode::ControlLeft) || game.keys.contains(&KeyCode::ControlRight);
    let selected = if control_down {
        select_excavation(origin, |candidate| {
            let [x, y, z] = candidate.0;
            let state = game.world.block(x, y, z);
            let name = &game.res.facts().get(state).name;
            !matches!(name.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") && within_reach(candidate)
        }, AreaSize { width: 3, height: 3, depth: 3 }, limits.max_blocks)
    } else if let Some(family) = tree_family_key(&origin_name) {
        select_tree(origin, |candidate| {
            let [x, y, z] = candidate.0;
            let state = game.world.block(x, y, z);
            tree_family_key(&game.res.facts().get(state).name).filter(|key| *key == family)
        }, limits).into_iter().filter(|candidate| within_reach(*candidate)).collect()
    } else {
        let key = block_name_key(&origin_name);
        select_vein(origin, |candidate| {
            let [x, y, z] = candidate.0;
            let state = game.world.block(x, y, z);
            let name = &game.res.facts().get(state).name;
            (name == &origin_name).then_some(key)
        }, limits).into_iter().filter(|candidate| within_reach(*candidate)).collect()
    };
    let extra: Vec<[i32; 3]> = selected.into_iter().filter(|candidate| *candidate != origin).map(|candidate| candidate.0).collect();
    if extra.is_empty() { return; }
    with_multibreak_guard(|| { for candidate in extra { let _ = game.break_block(candidate); } });
}
