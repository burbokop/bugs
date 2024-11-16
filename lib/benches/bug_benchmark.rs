use std::{
    f64::consts::PI,
    ops::AddAssign,
    time::{Duration, SystemTime},
};

use bugs_lib::{
    environment::{benchmark_internals, BugCreateInfo, Environment, FoodCreateInfo},
    math::{noneg_float, Angle},
    time_point::TimePoint,
};
use chromosome::Chromosome;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

fn find_nearest_food(c: &mut Criterion) {
    let mut rng: Pcg64 = Seeder::from(&[0xff]).make_rng();
    let the_beginning_of_times = FakeTime::default();
    {
        let environment = Environment::new(
            the_beginning_of_times.clone(),
            FoodCreateInfo::generate_vec(&mut rng, -50. ..50., -50. ..50., 0. ..1., 1024),
            vec![],
            vec![BugCreateInfo {
                chromosome: Chromosome::new_random(256, 1. ..1.01, &mut rng),
                position: (0., 0.).into(),
                rotation: Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
            }],
        );

        let bug = environment.bugs().next().unwrap();

        c.bench_function("find_nearest_food (small)", |b| {
            b.iter(|| black_box(bug.find_nearest_food_in_vision_arc(&environment)))
        });
    }
    {
        let environment = Environment::new(
            the_beginning_of_times.clone(),
            FoodCreateInfo::generate_vec(&mut rng, -50. ..50., -50. ..50., 0. ..1., 16384),
            vec![],
            vec![BugCreateInfo {
                chromosome: Chromosome::new_random(256, 1. ..1.01, &mut rng),
                position: (0., 0.).into(),
                rotation: Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
            }],
        );

        let bug = environment.bugs().next().unwrap();

        c.bench_function("find_nearest_food (big)", |b| {
            b.iter(|| black_box(bug.find_nearest_food_in_vision_arc(&environment)))
        });
    }
    {
        let environment = Environment::new(
            the_beginning_of_times.clone(),
            FoodCreateInfo::generate_vec(
                &mut rng,
                -10000. ..10000.,
                -10000. ..10000.,
                0. ..1.,
                16384,
            ),
            vec![],
            vec![BugCreateInfo {
                chromosome: Chromosome::new_random(256, 1. ..1.01, &mut rng),
                position: (0., 0.).into(),
                rotation: Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
            }],
        );

        let bug = environment.bugs().next().unwrap();

        c.bench_function("find_nearest_food (big, far)", |b| {
            b.iter(|| black_box(bug.find_nearest_food_in_vision_arc(&environment)))
        });
    }
}

fn find_nearest_bug(c: &mut Criterion) {
    let mut rng: Pcg64 = Seeder::from(&[0xff]).make_rng();
    let the_beginning_of_times = FakeTime::default();

    {
        let environment = Environment::new(
            the_beginning_of_times.clone(),
            vec![],
            vec![],
            BugCreateInfo::generate_vec(
                &mut rng,
                1. ..1.01,
                -50. ..50.,
                -50. ..50.,
                0. ..(PI * 2.),
                1024,
            ),
        );

        let bug = environment.bugs().next().unwrap();

        c.bench_function("find_nearest_bug (small)", |b| {
            b.iter(|| black_box(bug.find_nearest_bug_in_vision_arc(&environment)))
        });
    }
    {
        let environment = Environment::new(
            the_beginning_of_times.clone(),
            vec![],
            vec![],
            BugCreateInfo::generate_vec(
                &mut rng,
                1. ..1.01,
                -50. ..50.,
                -50. ..50.,
                0. ..(PI * 2.),
                16384,
            ),
        );

        let bug = environment.bugs().next().unwrap();

        c.bench_function("find_nearest_bug (big)", |b| {
            b.iter(|| black_box(bug.find_nearest_bug_in_vision_arc(&environment)))
        });
    }
    {
        let environment = Environment::new(
            the_beginning_of_times.clone(),
            vec![],
            vec![],
            BugCreateInfo::generate_vec(
                &mut rng,
                1. ..1.01,
                -10000. ..10000.,
                -10000. ..10000.,
                0. ..(PI * 2.),
                16384,
            ),
        );

        let bug = environment.bugs().next().unwrap();

        c.bench_function("find_nearest_bug (big, far)", |b| {
            b.iter(|| black_box(bug.find_nearest_bug_in_vision_arc(&environment)))
        });
    }
}

fn transfer_energy_from_food_to_bug(c: &mut Criterion) {
    let mut rng: Pcg64 = Seeder::from(&[0xff]).make_rng();
    let the_beginning_of_times = FakeTime::default();

    {
        let mut environment = Environment::new(
            the_beginning_of_times.clone(),
            FoodCreateInfo::generate_vec(&mut rng, -50. ..50., -50. ..50., 0. ..1., 1024),
            vec![],
            BugCreateInfo::generate_vec(
                &mut rng,
                1. ..1.01,
                -50. ..50.,
                -50. ..50.,
                0. ..(PI * 2.),
                1024,
            ),
        );

        let bug = benchmark_internals::find_bug_by_id(&mut environment, 512).unwrap();

        c.bench_function("transfer_energy_from_food_to_bug (small)", |b| {
            b.iter(|| {
                black_box(benchmark_internals::transfer_energy_from_food_to_bug(
                    &mut environment,
                    512,
                    &mut bug.borrow_mut(),
                    noneg_float(0.00001),
                ))
            })
        });
    }
    {
        let mut environment = Environment::new(
            the_beginning_of_times.clone(),
            FoodCreateInfo::generate_vec(&mut rng, -50. ..50., -50. ..50., 0. ..1., 16384),
            vec![],
            BugCreateInfo::generate_vec(
                &mut rng,
                1. ..1.01,
                -50. ..50.,
                -50. ..50.,
                0. ..(PI * 2.),
                16384,
            ),
        );

        let bug = benchmark_internals::find_bug_by_id(&mut environment, 512).unwrap();

        c.bench_function("transfer_energy_from_food_to_bug (big)", |b| {
            b.iter(|| {
                black_box(benchmark_internals::transfer_energy_from_food_to_bug(
                    &mut environment,
                    8192,
                    &mut bug.borrow_mut(),
                    noneg_float(0.00001),
                ))
            })
        });
    }
    {
        let mut environment = Environment::new(
            the_beginning_of_times.clone(),
            FoodCreateInfo::generate_vec(
                &mut rng,
                -10000. ..10000.,
                -10000. ..10000.,
                0. ..1.,
                16384,
            ),
            vec![],
            BugCreateInfo::generate_vec(
                &mut rng,
                1. ..1.01,
                -10000. ..10000.,
                -10000. ..10000.,
                0. ..(PI * 2.),
                16384,
            ),
        );

        let bug = benchmark_internals::find_bug_by_id(&mut environment, 512).unwrap();

        c.bench_function("transfer_energy_from_food_to_bug (big, far)", |b| {
            b.iter(|| {
                black_box(benchmark_internals::transfer_energy_from_food_to_bug(
                    &mut environment,
                    8192,
                    &mut bug.borrow_mut(),
                    noneg_float(0.00001),
                ))
            })
        });
    }
}

criterion_group!(
    benches,
    find_nearest_food,
    find_nearest_bug,
    transfer_energy_from_food_to_bug,
);
criterion_main!(benches);

// #1
// find_nearest_food (small) time:   [4.7495 µs]
// find_nearest_food (big)   time:   [70.691 µs]

// #2
// find_nearest_food (small) time:   [1.6804 µs]
// find_nearest_food (big)   time:   [24.740 µs]

// #3
// find_nearest_food (small)         [1.7837 µs]
// find_nearest_food (big)           [28.850 µs]
// find_nearest_food (big, far)      [26.800 µs]
// find_nearest_bug  (small)         [4.9104 µs]
// find_nearest_bug  (big)           [127.32 µs]
// find_nearest_bug  (big, far)      [102.88 µs]

// #4 returning dst from `find_nearest_bug`
// find_nearest_food (small)         [1.8035 µs]
// find_nearest_food (big)           [30.688 µs]
// find_nearest_food (big, far)      [28.254 µs]
// find_nearest_bug  (small)         [3.1962 µs]
// find_nearest_bug  (big)           [112.86 µs]
// find_nearest_bug  (big, far)      [99.102 µs]

// #5 chunk system applied to food
// find_nearest_food (small)         [348.60 ns]
// find_nearest_food (big)           [6.3736 µs]
// find_nearest_food (big, far)      [46.268 ns]
// find_nearest_bug (small)          [3.8314 µs]
// find_nearest_bug (big)            [99.788 µs]
// find_nearest_bug (big, far)       [115.90 µs]
