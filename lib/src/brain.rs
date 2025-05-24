use crate::{
    color::Color,
    math::{self, clamp_into_range, noneg_float, Angle, DeltaAngle, NoNeg},
    range::Range,
    utils::{Float, RequiredToBeInRange as _},
};
use chromosome::Chromosome;
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

fn relative_radius_to_activation(relative_radius: NoNeg<Float>) -> Float {
    clamp_into_range(
        relative_radius.unwrap(),
        MIN_RELATIVE_RADIUS.unwrap()..=MAX_RELATIVE_RADIUS.unwrap(),
        0. ..=1.,
    )
}

const MAX_RELATIVE_RADIUS: NoNeg<Float> = noneg_float(64.);
const MIN_RELATIVE_RADIUS: NoNeg<Float> = noneg_float(0.);

#[derive(Clone)]
pub struct Brain {
    net: Net<Float>,
}

#[derive(Debug, Clone)]
pub struct FoodInfo {
    pub dst: NoNeg<Float>,
    pub direction: Angle<Float>,
    pub relative_radius: NoNeg<Float>,
}

#[derive(Debug, Clone)]
pub struct BugInfo {
    pub dst: NoNeg<Float>,
    pub direction: Angle<Float>,
    pub color: Color,
    pub relative_radius: NoNeg<Float>,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub energy_level: NoNeg<Float>,
    pub energy_capacity: NoNeg<Float>,
    pub rotation: Angle<Float>,
    pub age: NoNeg<Float>,
    pub baby_charge_level: NoNeg<Float>,
    pub baby_charge_capacity: NoNeg<Float>,
    pub vision_range: NoNeg<Float>,
    pub nearest_food: Option<FoodInfo>,
    pub nearest_bug: Option<BugInfo>,
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
                .nearest_food
                .as_ref()
                .map(|x| (x.dst / value.vision_range).unwrap())
                .unwrap_or(1.),
            value
                .nearest_food
                .as_ref()
                .map(|d| delta_angle_to_activation(d.direction.signed_distance(value.rotation)))
                .unwrap_or(0.),
            value
                .nearest_food
                .map(|x| relative_radius_to_activation(x.relative_radius))
                .unwrap_or(1.),
            value.age.unwrap(),
            value
                .nearest_bug
                .as_ref()
                .map(|p| (p.dst / value.vision_range).unwrap())
                .unwrap_or(1.),
            value
                .nearest_bug
                .as_ref()
                .map(|d| delta_angle_to_activation(d.direction.signed_distance(value.rotation)))
                .unwrap_or(0.),
            value.nearest_bug.as_ref().map(|x| x.color.a).unwrap_or(0.),
            value.nearest_bug.as_ref().map(|x| x.color.r).unwrap_or(0.),
            value.nearest_bug.as_ref().map(|x| x.color.g).unwrap_or(0.),
            value.nearest_bug.as_ref().map(|x| x.color.b).unwrap_or(0.),
            value
                .nearest_bug
                .map(|x| relative_radius_to_activation(x.relative_radius))
                .unwrap_or(1.),
            value.baby_charge_level.unwrap() / value.baby_charge_capacity.unwrap(),
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
        self.net
            .proceed(&input.into(), normalizers::fast_fake_sigmoid)
            .into()
    }

    pub(crate) fn proceed_verbosely(&self, input: Input) -> VerboseOutput {
        let i = input.into();
        let (r0, r1) = self
            .net
            .proceed_verbosely(&i, normalizers::fast_fake_sigmoid);
        VerboseOutput {
            output: r1.clone().into(),
            activations: (i, *r0, *r1),
        }
    }
}
