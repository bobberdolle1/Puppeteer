// Middleware utilities for the admin bot
// Currently using inline filters in mod.rs, but this file is reserved for future middleware

use crate::AppState;

/// Check if a user is an owner
pub fn is_owner(user_id: i64, state: &AppState) -> bool {
    state.config.is_owner(user_id)
}
