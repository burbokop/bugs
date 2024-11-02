#![no_main]

use std::{
    f64::consts::PI,
    time::Duration,
};

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

// Runs small simulation with limited resources until no bugs are left. Uses input data as seed for random generator.
fuzz_target!(|data: &[u8]| {
    let mut rng: Pcg64 = Seeder::from(data).make_rng();
    let the_beginning_of_times = std::time::UNIX_EPOCH;

    let mut environment = Environment::new(
        the_beginning_of_times,
        Food::generate_vec(&mut rng, -50. ..50., -50. ..50., 0. ..1., 1024),
        vec![],
        vec![Bug::give_birth_with_max_energy(
            Chromosome::new_random(256, (-1.)..1., &mut rng),
            (0., 0.).into(),
            Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
            the_beginning_of_times,
        )],
    );

    println!("start. data: {:?}, genes: {:?}", data, environment.bugs().next().unwrap().chromosome().genes);

    let dt = Duration::from_millis(1000 / 30);
    let mut i: usize = 0;
    while environment.bugs_count() > 0 {
        environment.proceed(dt, &mut rng);

        if i % 100 == 0 {
            println!("iteration {}, time: {}s, population: {}, food: {}", i, environment.now().duration_since(the_beginning_of_times).unwrap().as_secs(), environment.bugs_count(), environment.food().len());
        }
        i+=1;
    }
});
