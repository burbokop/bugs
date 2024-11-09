#![no_main]

use std::{
    f64::consts::PI,
    ops::AddAssign,
    time::{Duration, Instant, SystemTime},
};

use bugs_lib::utils::pretty_duration;
use bugs_lib::{environment::Environment, math::Angle};
use bugs_lib::{
    environment::{BugCreateInfo, FoodCreateInfo},
    time_point::TimePoint,
};
use chromosome::Chromosome;
use libfuzzer_sys::fuzz_target;
use memory_stats::memory_stats;
use rand::Rng as _;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

#[derive(Clone)]
struct FakeTime(SystemTime);

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
        FoodCreateInfo::generate_vec(&mut rng, -50. ..50., -50. ..50., 0. ..1., 512),
        vec![],
        vec![BugCreateInfo {
            chromosome: Chromosome::new_random(256, (-1.)..1., &mut rng),
            position: (0., 0.).into(),
            rotation: Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
        }],
    );

    println!(
        "start. data: {:?}, genes: {:?}",
        data,
        environment.bugs().next().unwrap().chromosome().genes
    );

    std::thread::spawn(|| loop {
        if let Some(usage) = memory_stats() {
            if usage.physical_mem > 1024 * 1024 * 1024 {
                panic!("Current memory usage exceeds limit: {:?}", usage);
            }
        } else {
            panic!("Couldn't get the current memory usage");
        }
        std::thread::sleep(Duration::from_secs(1));
    });

    let dt = Duration::from_millis(1000 / 30);
    let mut i: usize = 0;

    let mut last_log_instant = Instant::now();
    while environment.bugs_count() > 0 {
        environment.proceed(dt, &mut rng);
        let now = Instant::now();
        if i % 100 == 0 || now - last_log_instant > Duration::from_secs(5) {
            println!(
                "iteration {}, time: {}, population: {}, food: {}",
                i,
                pretty_duration(environment.now().duration_since(&the_beginning_of_times)),
                environment.bugs_count(),
                environment.food().len()
            );
            last_log_instant = now
        }
        i += 1;
    }
});
