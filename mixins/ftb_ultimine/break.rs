// Hook point validated against the pinned Horizon source.
fn before_break(pos: [i32; 3]) {
    crate::game::ftb_ultimine::on_break_attempt(pos);
}
