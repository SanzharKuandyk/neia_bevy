pub mod handler;
pub mod validator;

#[derive(Debug, Clone, Copy)]
pub enum MessageTarget {
    Single(u64),
    AllExceptOne(u64),
    All,
    /// Team-specific broadcast by team_id (only meaningful when teams are enabled).
    Team(u8),
}
