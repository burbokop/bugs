use std::{
    ops::AddAssign,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

pub trait TimePoint: AddAssign<Duration> {
    fn duration_since(&self, other: &Self) -> Duration;
}

impl TimePoint for Instant {
    fn duration_since(&self, other: &Self) -> Duration {
        *self - *other
    }
}

// nanoseconds stored in u64 can be max ~ 600 years
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaticTimePoint(u64);

impl Default for StaticTimePoint {
    fn default() -> Self {
        Self(0)
    }
}

impl AddAssign<Duration> for StaticTimePoint {
    fn add_assign(&mut self, rhs: Duration) {
        let nanos = rhs.as_nanos();
        assert!(self.0 as u128 + nanos <= u64::MAX as u128);
        self.0 += nanos as u64;
    }
}

impl TimePoint for StaticTimePoint {
    fn duration_since(&self, other: &Self) -> Duration {
        assert!(self.0 >= other.0);
        Duration::from_nanos(self.0 - other.0)
    }
}
