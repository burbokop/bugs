use crate::{
    math::{self, Angle, DeltaAngle, NoNeg},
    range::Range,
    utils::{Color, Float, RequiredToBeInRange as _},
};
use chromosome::Chromosome;
use serde::Serialize;
use simple_neural_net::{normalizers, Arr, Layer as _, PerceptronLayer};
use std::f64::consts::PI;

simple_neural_net::compose_layers!(Net, 16, 8, 8);

fn angle_to_activation(a: Angle<Float>) -> Float {
    math::fit_into_range(a.radians(), 0. ..PI * 2., -1. ..1.).unwrap()
}

fn activation_to_angle(a: Float) -> Angle<Float> {
    Angle::from_radians(math::fit_into_range_inclusive(a, -1. ..=1., 0. ..=PI * 2.).unwrap())
}

fn delta_angle_to_activation(a: DeltaAngle<Float>) -> Float {
    math::fit_into_range(a.radians(), (-PI * 2.)..PI * 2., -1. ..1.).unwrap()
}

fn activation_to_delta_angle(a: Float) -> DeltaAngle<Float> {
    DeltaAngle::from_radians(
        math::fit_into_range_inclusive(a, -1. ..=1., (-PI * 2.)..=PI * 2.).unwrap(),
    )
}

fn activation_to_noneg_delta_angle(a: Float) -> DeltaAngle<NoNeg<Float>> {
    DeltaAngle::from_radians(
        NoNeg::wrap(math::fit_into_range_inclusive(a.abs(), 0. ..=1., 0. ..=PI * 2.).unwrap())
            .unwrap(),
    )
}

#[derive(Clone)]
pub struct Brain {
    net: Net<Float>,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub energy_level: NoNeg<Float>,
    pub energy_capacity: NoNeg<Float>,
    pub rotation: Angle<Float>,
    pub proximity_to_food: Option<NoNeg<Float>>,
    pub direction_to_nearest_food: Option<Angle<Float>>,
    pub size_of_nearest_food: Option<NoNeg<Float>>,
    pub age: NoNeg<Float>,
    pub proximity_to_bug: Option<NoNeg<Float>>,
    pub direction_to_nearest_bug: Option<Angle<Float>>,
    pub color_of_nearest_bug: Option<Color>,
    pub baby_charge_level: NoNeg<Float>,
    pub baby_charge_capacity: NoNeg<Float>,
    pub vision_range: NoNeg<Float>,
}

#[derive(Debug, Clone)]
pub struct Output {
    /// in pixels per second
    pub velocity: Float,
    /// rotaion relative to self rotation
    pub relative_desired_rotation: DeltaAngle<Float>,
    /// per second
    pub rotation_velocity: DeltaAngle<NoNeg<Float>>,
    /// energy per second
    pub baby_charging_rate: NoNeg<Float>,
}

pub(crate) struct VerboseOutput {
    pub output: Output,
    pub activations: ([Float; 16], [Float; 8], [Float; 8]),
}

impl From<Input> for [Float; 16] {
    fn from(value: Input) -> Self {
        [
            (value.energy_level / value.energy_capacity).unwrap(),
            value
                .proximity_to_food
                .map(|x| (x / value.vision_range).unwrap())
                .unwrap_or(1.),
            value
                .direction_to_nearest_food
                .map(|d| delta_angle_to_activation(d.signed_distance(value.rotation)))
                .unwrap_or(0.),
            value
                .size_of_nearest_food
                .map(|x| x.unwrap() / 32.)
                .unwrap_or(1.),
            value.age.unwrap(),
            value
                .proximity_to_bug
                .map(|p| (p / value.vision_range).unwrap())
                .unwrap_or(1.),
            value
                .direction_to_nearest_bug
                .map(|d| delta_angle_to_activation(d.signed_distance(value.rotation)))
                .unwrap_or(0.),
            value
                .color_of_nearest_bug
                .as_ref()
                .map(|x| x.a)
                .unwrap_or(0.),
            value
                .color_of_nearest_bug
                .as_ref()
                .map(|x| x.r)
                .unwrap_or(0.),
            value
                .color_of_nearest_bug
                .as_ref()
                .map(|x| x.g)
                .unwrap_or(0.),
            value
                .color_of_nearest_bug
                .as_ref()
                .map(|x| x.b)
                .unwrap_or(0.),
            value.baby_charge_level.unwrap() / value.baby_charge_capacity.unwrap(),
            0.,
            0.,
            0.,
            0.,
        ]
        .required_to_be_in_range(-1. ..=1.)
        .unwrap()
    }
}

impl From<Arr<Float, 8>> for Output {
    fn from(value: Arr<Float, 8>) -> Self {
        Self {
            velocity: value[0] * 10.,
            relative_desired_rotation: activation_to_delta_angle(value[1]),
            rotation_velocity: activation_to_noneg_delta_angle(value[2]),
            baby_charging_rate: NoNeg::wrap(
                math::fit_into_range_inclusive(value[3].abs(), 0. ..=1., 0. ..=10.).unwrap(),
            )
            .unwrap(),
        }
    }
}

impl Brain {
    pub fn layers(
        &self,
    ) -> (
        &PerceptronLayer<Float, 16, 8>,
        &PerceptronLayer<Float, 8, 8>,
    ) {
        (&self.net.l0, &self.net.l1)
    }

    pub(crate) fn new<R: Into<Range<usize>>>(chromosome: &Chromosome<Float>, range: R) -> Self {
        let range = range.into();
        let genes = &chromosome.genes[range.start..range.end];
        assert_eq!(genes.len(), 208);

        let l0w_genes = &genes[0..128];
        let l1w_genes = &genes[128..192];

        let l0b_genes = &genes[192..200];
        let l1b_genes = &genes[200..208];

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
            ]
            .into(),
            [
                (l1w_genes[00..08].try_into().unwrap(), l1b_genes[0]).into(),
                (l1w_genes[08..16].try_into().unwrap(), l1b_genes[1]).into(),
                (l1w_genes[16..24].try_into().unwrap(), l1b_genes[2]).into(),
                (l1w_genes[24..32].try_into().unwrap(), l1b_genes[3]).into(),
                (l1w_genes[32..40].try_into().unwrap(), l1b_genes[4]).into(),
                (l1w_genes[40..48].try_into().unwrap(), l1b_genes[5]).into(),
                (l1w_genes[48..56].try_into().unwrap(), l1b_genes[6]).into(),
                (l1w_genes[56..64].try_into().unwrap(), l1b_genes[7]).into(),
            ]
            .into(),
        );

        Brain { net }
    }

    pub(crate) fn proceed(&self, input: Input) -> Output {
        self.net.proceed(&input.into(), normalizers::sigmoid).into()
    }

    pub(crate) fn proceed_verbosely(&self, input: Input) -> VerboseOutput {
        let i = input.into();
        let (r0, r1) = self
            .net
            .proceed_verbosely(&i, |x| normalizers::sigmoid(x) * 2. - 1.);
        VerboseOutput {
            output: r1.clone().into(),
            activations: (i, *r0, *r1),
        }
    }
}