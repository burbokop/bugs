use core::range::Range;
use chromosome::Chromosome;
use simple_neural_net::{normalizers, Arr, Layer as _};
use crate::utils::{Color, Float};

simple_neural_net::compose_layers!(
    Net,
    16, 8, 8
);

pub(crate) struct Brain {
    net: Net<Float>
}

#[derive(Debug, Clone)]
pub(crate) struct Input {
    pub(crate) energy_level: Float,
    pub(crate) proximity_to_food: Float,
    pub(crate) direction_to_nearest_food: Float,
    pub(crate) age: Float,
    pub(crate) proximity_to_bug: Float,
    pub(crate) direction_to_nearest_bug: Float,
    pub(crate) color_of_nearest_bug: Color,
    pub(crate) baby_charge: Float,
}

#[derive(Debug, Clone)]
pub(crate) struct Output {
    /// in pixels per second
    pub(crate) velocity: Float,
    /// radians per second
    pub(crate) rotation_velocity: Float,
    /// energy per second
    pub(crate) baby_charging_rate: Float,
}

impl From<Input> for [Float; 16] {
    fn from(value: Input) -> Self {
        [
          value.energy_level,
          value.proximity_to_food,
          value.direction_to_nearest_food,
          value.age,
          value.proximity_to_bug,
          value.direction_to_nearest_bug,
          value.color_of_nearest_bug.a,
          value.color_of_nearest_bug.r,
          value.color_of_nearest_bug.g,
          value.color_of_nearest_bug.b,
          value.baby_charge,
          0.,
          0.,
          0.,
          0.,
          0.,
        ]
    }
}

impl From<Arr<Float, 8>> for Output {
    fn from(value: Arr<Float, 8>) -> Self {
        Self {
            velocity: value[0],
            rotation_velocity: value[1],
            baby_charging_rate: value[2],
        }
    }
}

impl Brain {
    pub(crate) fn new<R: Into<Range<usize>>>(chromosome: &Chromosome<Float>, range: R) -> Self {
        let range = range.into();
        let genes = &chromosome.genes[range.start..range.end];
        assert_eq!(genes.len(), 208);

        let l0w_genes= &genes[0..128];
        let l1w_genes= &genes[128..192];

        let l0b_genes= &genes[192..200];
        let l1b_genes= &genes[200..208];

        let net: Net<f64> = Net::new(
            [
                (l0w_genes[000..016].try_into().unwrap(), l0b_genes[0]).into(),
                (l0w_genes[016..032].try_into().unwrap(), l0b_genes[1]).into(),
                (l0w_genes[032..048].try_into().unwrap(), l0b_genes[2]).into(),
                (l0w_genes[048..064].try_into().unwrap(), l0b_genes[3]).into(),
                (l0w_genes[064..080].try_into().unwrap(), l0b_genes[4]).into(),
                (l0w_genes[080..096].try_into().unwrap(), l0b_genes[5]).into(),
                (l0w_genes[096..112].try_into().unwrap(), l0b_genes[6]).into(),
                (l0w_genes[112..128].try_into().unwrap(), l0b_genes[7]).into(),
            ].into(),
            [
                (l1w_genes[00..08].try_into().unwrap(), l1b_genes[0]).into(),
                (l1w_genes[08..16].try_into().unwrap(), l1b_genes[1]).into(),
                (l1w_genes[16..24].try_into().unwrap(), l1b_genes[2]).into(),
                (l1w_genes[24..32].try_into().unwrap(), l1b_genes[3]).into(),
                (l1w_genes[32..40].try_into().unwrap(), l1b_genes[4]).into(),
                (l1w_genes[40..48].try_into().unwrap(), l1b_genes[5]).into(),
                (l1w_genes[48..56].try_into().unwrap(), l1b_genes[6]).into(),
                (l1w_genes[56..64].try_into().unwrap(), l1b_genes[7]).into(),
            ].into(),
        );

        Brain { net }
    }

    pub(crate) fn proceed(&self, input: Input) -> Output {
        self.net.proceed(&input.into(), normalizers::sigmoid).into()
    }
}
