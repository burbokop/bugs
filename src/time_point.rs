use std::{ops::AddAssign, time::{Duration, Instant}};

pub trait TimePoint: AddAssign<Duration> {
    fn duration_since(&self, other: &Self) -> Duration;
}

impl TimePoint for Instant {
    fn duration_since(&self, other: &Self) -> Duration {
        *self - *other
    }
}
