#![no_main]
#![feature(duration_constructors)]
#![feature(duration_millis_float)]

use std::{f64::consts::PI, ops::AddAssign, time::{Duration, SystemTime}};

use bugs::{
    bug::Bug,
    environment::{Environment, Food},
    math::Angle,
};
use chromosome::Chromosome;
use libfuzzer_sys::fuzz_target;
use rand::Rng as _;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

fn pretty_duration(duration: Duration) -> String {
    if duration > Duration::from_hours(1) {
        return format!("{:.2} h", duration.as_secs_f64() / 60. / 60.);
    } else if duration > Duration::from_mins(1) {
        return format!("{:.2} m", duration.as_secs_f64() / 60.);
    } else if duration > Duration::from_secs(1) {
        return format!("{:.2} s", duration.as_millis_f64() / 1000.);
    } else if duration > Duration::from_millis(1) {
        return format!("{:.2} ms", duration.as_micros() as f64 / 1000.);
    } else if duration > Duration::from_micros(1) {
        return format!("{:.2} µs", duration.as_nanos() as f64 / 1000.);
    } else {
        return format!("{} ns", duration.as_nanos());
    }
}

use bugs::time_point::TimePoint;

#[derive(Clone)]
struct FakeTime (SystemTime);

impl TimePoint for FakeTime {
    fn duration_since(&self, other: &Self) -> Duration {
       self.0.duration_since(other.0).unwrap()
    }
}

impl Default for FakeTime {
    fn default() -> Self {
        Self(std::time::UNIX_EPOCH)
    }
}

impl AddAssign<Duration> for FakeTime {
    fn add_assign(&mut self, rhs: Duration) {
        self.0 += rhs
    }
}

// Runs small simulation with limited resources until no bugs are left. Uses input data as seed for random generator.
fuzz_target!(|data: &[u8]| {
    let mut rng: Pcg64 = Seeder::from(data).make_rng();
    let the_beginning_of_times = FakeTime::default();

    let mut environment = Environment::new(
        the_beginning_of_times.clone(),
        Food::generate_vec(&mut rng, -50. ..50., -50. ..50., 0. ..1., 1024),
        vec![],
        vec![Bug::give_birth_with_max_energy(
            Chromosome::new_random(256, (-1.)..1., &mut rng),
            (0., 0.).into(),
            Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
            the_beginning_of_times.clone(),
        )],
    );

    println!(
        "start. data: {:?}, genes: {:?}",
        data,
        environment.bugs().next().unwrap().chromosome().genes
    );

    let dt = Duration::from_millis(1000 / 30);
    let mut i: usize = 0;
    while environment.bugs_count() > 0 {
        environment.proceed(dt, &mut rng);

        if i % 100 == 0 {
            println!(
                "iteration {}, time: {}, population: {}, food: {}",
                i,
                pretty_duration(
                    environment
                        .now()
                        .duration_since(&the_beginning_of_times)
                ),
                environment.bugs_count(),
                environment.food().len()
            );
        }
        i += 1;
    }
});
