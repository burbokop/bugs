use chromosome::{Chromosome, Superposition};
use rand::Rng;
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub(crate) trait ExtendedChromosome<T> {
    fn mutated_ext<D: Superposition<T> + Clone, F: Fn(usize) -> D, R: rand::RngCore>(
        self: &Self,
        delta: F,
        chance: f64,
        rng: &mut R,
    ) -> Chromosome<T>;

    fn mutate<D: Superposition<T> + Clone, F: Fn(usize) -> D, R: rand::RngCore>(
        self: &mut Self,
        delta: F,
        chance: f64,
        rng: &mut R,
    );
}

impl<T: Add<Output = T> + Sub<Output = T> + AddAssign + SubAssign + Clone> ExtendedChromosome<T>
    for Chromosome<T>
{
    /// get random mutated chromosome
    fn mutated_ext<D: Superposition<T> + Clone, F: Fn(usize) -> D, R: rand::RngCore>(
        self: &Self,
        delta: F,
        chance: f64,
        rng: &mut R,
    ) -> Chromosome<T> {
        Chromosome::new(
            self.genes
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, gene)| {
                    if rng.gen_bool(chance) {
                        if rng.gen_bool(0.5) {
                            gene + delta(i).clone().collapse(rng)
                        } else {
                            gene - delta(i).clone().collapse(rng)
                        }
                    } else {
                        gene
                    }
                })
                .collect(),
        )
    }

    fn mutate<D: Superposition<T> + Clone, F: Fn(usize) -> D, R: rand::RngCore>(
        self: &mut Self,
        delta: F,
        chance: f64,
        rng: &mut R,
    ) {
        self.genes.iter_mut().enumerate().for_each(|(i, gene)| {
            if rng.gen_bool(chance) {
                if rng.gen_bool(0.5) {
                    *gene += delta(i).clone().collapse(rng);
                } else {
                    *gene -= delta(i).clone().collapse(rng);
                }
            }
        })
    }
}
