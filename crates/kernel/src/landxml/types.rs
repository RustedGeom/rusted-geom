use crate::RgmPoint3 as Vec3;

pub(crate) trait Vec3Ops {
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn scale(self, t: f64) -> Self;
    fn distance(self, other: Self) -> f64;
    fn normalize(self) -> Self;
}

impl Vec3Ops for Vec3 {
    fn add(self, other: Self) -> Self {
        Vec3 { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }
    fn sub(self, other: Self) -> Self {
        Vec3 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z }
    }
    fn scale(self, t: f64) -> Self {
        Vec3 { x: self.x * t, y: self.y * t, z: self.z * t }
    }
    fn distance(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    fn normalize(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len < 1e-15 { return self; }
        Vec3 { x: self.x / len, y: self.y / len, z: self.z / len }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LandXmlParseMode {
    #[default]
    Strict,
    Lenient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LandXmlPointOrder {
    #[default]
    Nez,
    Enz,
    Ezn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LandXmlUnitsPolicy {
    #[default]
    NormalizeToMeters,
    PreserveSource,
}

#[derive(Clone, Debug)]
pub struct LandXmlParseOptions {
    pub mode: LandXmlParseMode,
    pub point_order: LandXmlPointOrder,
    pub units_policy: LandXmlUnitsPolicy,
    pub angular_unit_override: Option<String>,
}

impl Default for LandXmlParseOptions {
    fn default() -> Self {
        Self {
            mode: LandXmlParseMode::Strict,
            point_order: LandXmlPointOrder::Nez,
            units_policy: LandXmlUnitsPolicy::NormalizeToMeters,
            angular_unit_override: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LandXmlUnits {
    pub source_linear_unit: String,
    pub source_angular_unit: Option<String>,
    pub linear_to_meters: f64,
    pub normalized_to_meters: bool,
}

impl Default for LandXmlUnits {
    fn default() -> Self {
        Self {
            source_linear_unit: "meter".to_string(),
            source_angular_unit: None,
            linear_to_meters: 1.0,
            normalized_to_meters: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LandXmlWarning {
    pub code: String,
    pub message: String,
    pub path: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanLinearKind {
    FeatureLine,
    Breakline,
}

#[derive(Clone, Debug)]
pub struct PlanLinear {
    pub name: String,
    pub kind: PlanLinearKind,
    pub points: Vec<Vec3>,
}

#[derive(Clone, Debug)]
pub struct LandXmlDocument {
    pub units: LandXmlUnits,
    pub alignments: Vec<AlignmentRecord>,
    pub surfaces: Vec<TerrainTin>,
    pub plan_linears: Vec<PlanLinear>,
    pub warnings: Vec<LandXmlWarning>,
}

#[derive(Clone, Debug)]
pub struct AlignmentRecord {
    pub name: String,
    pub sta_start_m: f64,
    pub length_m: f64,
    pub station_map: StationMap,
    pub horizontal: HorizontalAlignment,
    pub profiles: Vec<ProfileSeries>,
}

#[derive(Clone, Debug)]
pub struct HorizontalAlignment {
    pub segments: Vec<HorizontalSegment>,
    pub total_length_m: f64,
}

#[derive(Clone, Debug)]
pub enum HorizontalSegment {
    Line(LineSegment),
    CircularArc(CircularArcSegment),
    Spiral(SpiralSegment),
}

#[derive(Clone, Debug)]
pub struct LineSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub start_station_m: f64,
    pub length_m: f64,
    pub start_heading_rad: f64,
}

#[derive(Clone, Debug)]
pub struct CircularArcSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub center: Vec3,
    pub start_station_m: f64,
    pub length_m: f64,
    pub radius_m: f64,
    pub start_angle_rad: f64,
    pub sweep_rad: f64,
    pub clockwise: bool,
    pub start_heading_rad: f64,
}

#[derive(Clone, Debug)]
pub struct SpiralSegment {
    pub spi_type: SpiralType,
    pub start: Vec3,
    pub end: Vec3,
    pub start_station_m: f64,
    pub length_m: f64,
    pub radius_start_m: Option<f64>,
    pub radius_end_m: Option<f64>,
    pub k0_per_m: f64,
    pub k1_per_m: f64,
    pub start_heading_rad: f64,
    pub clockwise: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpiralType {
    Biquadratic,
    Bloss,
    Clothoid,
    Cosine,
    Cubic,
    Sinusoid,
    RevBiquadratic,
    RevBloss,
    RevCosine,
    RevSinusoid,
    SineHalfWave,
    BiquadraticParabola,
    CubicParabola,
    JapaneseCubic,
    Radioid,
    WeinerBogen,
}

impl SpiralType {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "biquadratic" => Some(Self::Biquadratic),
            "bloss" => Some(Self::Bloss),
            "clothoid" => Some(Self::Clothoid),
            "cosine" => Some(Self::Cosine),
            "cubic" => Some(Self::Cubic),
            "sinusoid" => Some(Self::Sinusoid),
            "revBiquadratic" => Some(Self::RevBiquadratic),
            "revBloss" => Some(Self::RevBloss),
            "revCosine" => Some(Self::RevCosine),
            "revSinusoid" => Some(Self::RevSinusoid),
            "sineHalfWave" => Some(Self::SineHalfWave),
            "biquadraticParabola" => Some(Self::BiquadraticParabola),
            "cubicParabola" => Some(Self::CubicParabola),
            "japaneseCubic" => Some(Self::JapaneseCubic),
            "radioid" => Some(Self::Radioid),
            "weinerBogen" => Some(Self::WeinerBogen),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileKind {
    ProfAlign,
    ProfSurf,
}

#[derive(Clone, Debug)]
pub struct ProfileSeries {
    pub id: String,
    pub kind: ProfileKind,
    pub station_start_m: f64,
    pub station_end_m: f64,
    pub vertical_model: VerticalModel,
    pub sampled_profile: Vec<Vec3>,
}

#[derive(Clone, Debug)]
pub enum VerticalModel {
    Designed(DesignedVerticalModel),
    Sampled(SampledVerticalModel),
}

#[derive(Clone, Debug)]
pub struct DesignedVerticalModel {
    pub nodes: Vec<VerticalNode>,
    pub tangents: Vec<TangentInterval>,
    pub curves: Vec<VerticalCurveInterval>,
}

#[derive(Clone, Debug)]
pub struct VerticalNode {
    pub station_m: f64,
    pub elevation_m: f64,
}

#[derive(Clone, Debug)]
pub struct TangentInterval {
    pub s0: f64,
    pub s1: f64,
    pub z0: f64,
    pub grade: f64,
}

#[derive(Clone, Debug)]
pub enum VerticalCurveInterval {
    SymmetricParabola(ParabolaVerticalCurve),
    Circular(CircularVerticalCurve),
    AsymmetricParabola(AsymmetricParabolaVerticalCurve),
}

#[derive(Clone, Debug)]
pub struct ParabolaVerticalCurve {
    pub s0: f64,
    pub s1: f64,
    pub z0: f64,
    pub g0: f64,
    pub a: f64,
}

#[derive(Clone, Debug)]
pub struct CircularVerticalCurve {
    pub s0: f64,
    pub s1: f64,
    pub z0: f64,
    pub theta0_rad: f64,
    pub b_per_m: f64,
    pub radius_m: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct AsymmetricParabolaVerticalCurve {
    pub s_bvc: f64,
    pub s_pvi: f64,
    pub s_evc: f64,
    pub z_bvc: f64,
    pub z_pvi: f64,
    pub g0: f64,
    pub g_mid: f64,
    pub g1: f64,
}

#[derive(Clone, Debug)]
pub struct SampledVerticalModel {
    pub samples: Vec<VerticalNode>,
}

#[derive(Clone, Debug)]
pub struct Alignment3DTrack {
    pub alignment_name: String,
    pub profile_id: String,
    pub profile_kind: ProfileKind,
}

#[derive(Clone, Debug)]
pub struct TerrainTin {
    pub name: String,
    pub vertices_m: Vec<Vec3>,
    pub triangles: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StationIncrementDirection {
    Increasing,
    Decreasing,
}

#[derive(Clone, Debug)]
pub struct StationEquation {
    pub sta_internal_m: f64,
    pub sta_ahead_m: f64,
    pub sta_back_m: Option<f64>,
    pub increment: StationIncrementDirection,
}

#[derive(Clone, Debug)]
pub struct StationMap {
    pub sta_start_m: f64,
    pub length_m: f64,
    pub equations: Vec<StationEquation>,
}

#[derive(Clone, Copy, Debug)]
pub struct Alignment2DSample {
    pub point: Vec3,
    pub heading_rad: f64,
    pub curvature_per_m: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Alignment3DSample {
    pub point: Vec3,
    pub tangent: Vec3,
    pub grade: f64,
    pub horizontal_curvature_per_m: f64,
    pub vertical_curvature_per_m: f64,
}

#[derive(Clone, Debug)]
pub enum VerticalControlCurve {
    None,
    SymmetricParabola { length_m: f64 },
    Circular { length_m: f64, radius_m: f64 },
    AsymmetricParabola { length_in_m: f64, length_out_m: f64 },
}

#[derive(Clone, Debug)]
pub struct VerticalControlNode {
    pub station_m: f64,
    pub elevation_m: f64,
    pub curve: VerticalControlCurve,
    pub source_path: String,
}
