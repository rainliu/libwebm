use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Primitive)]
pub enum MatrixCoefficients {
    kGbr = 0,
    kBt709 = 1,
    kUnspecifiedMc = 2,
    kReserved = 3,
    kFcc = 4,
    kBt470bg = 5,
    kSmpte170MMc = 6,
    kSmpte240MMc = 7,
    kYcocg = 8,
    kBt2020NonConstantLuminance = 9,
    kBt2020ConstantLuminance = 10,
}

#[derive(Primitive)]
pub enum ChromaSitingHorz {
    kUnspecifiedCsh = 0,
    kLeftCollocated = 1,
    kHalfCsh = 2,
}

#[derive(Primitive)]
pub enum ChromaSitingVert {
    kUnspecifiedCsv = 0,
    kTopCollocated = 1,
    kHalfCsv = 2,
}

#[derive(Primitive)]
pub enum Range {
    kUnspecifiedCr = 0,
    kBroadcastRange = 1,
    kFullRange = 2,
    kMcTcDefined = 3, // Defined by MatrixCoefficients/TransferCharacteristics.
}

#[derive(Primitive)]
pub enum TransferCharacteristics {
    kIturBt709Tc = 1,
    kUnspecifiedTc = 2,
    kReservedTc = 3,
    kGamma22Curve = 4,
    kGamma28Curve = 5,
    kSmpte170MTc = 6,
    kSmpte240MTc = 7,
    kLinear = 8,
    kLog = 9,
    kLogSqrt = 10,
    kIec6196624 = 11,
    kIturBt1361ExtendedColourGamut = 12,
    kIec6196621 = 13,
    kIturBt202010bit = 14,
    kIturBt202012bit = 15,
    kSmpteSt2084 = 16,
    kSmpteSt4281Tc = 17,
    kAribStdB67Hlg = 18,
}

#[derive(Primitive)]
pub enum Primaries {
    kReservedP0 = 0,
    kIturBt709P = 1,
    kUnspecifiedP = 2,
    kReservedP3 = 3,
    kIturBt470M = 4,
    kIturBt470Bg = 5,
    kSmpte170MP = 6,
    kSmpte240MP = 7,
    kFilm = 8,
    kIturBt2020 = 9,
    kSmpteSt4281P = 10,
    kJedecP22Phosphors = 22,
}
