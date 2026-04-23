/// Unified game time representation used across all game systems
#[derive(Debug, Clone, Copy)]
pub struct GameTime {
    /// Game tick counter (increments every update cycle)
    pub tick: u64,
    /// Delta time since last update in seconds
    pub delta_secs: f32,
}

impl GameTime {
    pub fn new(tick: u64, delta_secs: f32) -> Self {
        Self { tick, delta_secs }
    }

    /// Calculate tick count from elapsed seconds and frame delta
    /// Useful for tests: `GameTime::ticks_from_secs(120.0, 0.016)`
    pub fn ticks_from_secs(secs: f32, delta_secs: f32) -> u64 {
        (secs / delta_secs).ceil() as u64
    }

    /// Calculate elapsed seconds from tick count and frame delta
    pub fn secs_from_ticks(ticks: u64, delta_secs: f32) -> f32 {
        ticks as f32 * delta_secs
    }
}
