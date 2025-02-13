use rand::RngCore;

use crate::state::AppState;

fn generate_token(app_state: &AppState) -> u8 {
    let mut u128_pool = [0u8; 1];
    app_state.rng.lock().unwrap().fill_bytes(&mut u128_pool);

    u8::from_le_bytes(u128_pool)
}
