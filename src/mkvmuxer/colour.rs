use super::util;
use super::writer::Writer;
use crate::MkvId;

///////////////////////////////////////////////////////////////
// Colour element.

#[derive(Debug, Copy, Clone)]
struct PrimaryChromaticity {
    x_: f32,
    y_: f32,
}

impl PrimaryChromaticity {
    const kChromaticityMin: f32 = 0.0;
    const kChromaticityMax: f32 = 1.0;

    pub fn new() -> PrimaryChromaticity {
        PrimaryChromaticity { x_: 0.0, y_: 0.0 }
    }

    pub fn x(&self) -> f32 {
        self.x_
    }
    pub fn set_x(&mut self, new_x: f32) {
        self.x_ = new_x;
    }
    pub fn y(&self) -> f32 {
        self.y_
    }
    pub fn set_y(&mut self, new_y: f32) {
        self.y_ = new_y;
    }

    pub fn Size(&self, x_id: MkvId, y_id: MkvId) -> u64 {
        util::EbmlElementSizeArgF32(x_id, self.x_) + util::EbmlElementSizeArgF32(y_id, self.y_)
    }

    pub fn Write(&self, writer: &mut dyn Writer, x_id: MkvId, y_id: MkvId) -> bool {
        if !self.Valid() {
            return false;
        }
        util::WriteEbmlElementArgF32(writer, x_id, self.x_)
            && util::WriteEbmlElementArgF32(writer, y_id, self.y_)
    }

    pub fn Valid(&self) -> bool {
        self.x_ >= Self::kChromaticityMin
            && self.x_ <= Self::kChromaticityMax
            && self.y_ >= Self::kChromaticityMin
            && self.y_ <= Self::kChromaticityMax
    }
}

#[derive(Debug, Copy, Clone)]
struct MasteringMetadata {
    luminance_max_: f32,
    luminance_min_: f32,
    r_: Option<PrimaryChromaticity>,
    g_: Option<PrimaryChromaticity>,
    b_: Option<PrimaryChromaticity>,
    white_point_: Option<PrimaryChromaticity>,
}

impl MasteringMetadata {
    const kMinLuminance: f32 = 0.0;
    const kMinLuminanceMax: f32 = 999.99;
    const kMaxLuminanceMax: f32 = 9999.99;
    const kValueNotPresent: f32 = std::f32::MAX;

    pub fn r(&self) -> Option<&PrimaryChromaticity> {
        self.r_.as_ref()
    }
    pub fn g(&self) -> Option<&PrimaryChromaticity> {
        self.g_.as_ref()
    }
    pub fn b(&self) -> Option<&PrimaryChromaticity> {
        self.b_.as_ref()
    }
    pub fn white_point(&self) -> Option<&PrimaryChromaticity> {
        self.white_point_.as_ref()
    }

    pub fn luminance_max(&self) -> f32 {
        self.luminance_max_
    }
    pub fn set_luminance_max(&mut self, luminance_max: f32) {
        self.luminance_max_ = luminance_max;
    }
    pub fn luminance_min(&self) -> f32 {
        self.luminance_min_
    }
    pub fn set_luminance_min(&mut self, luminance_min: f32) {
        self.luminance_min_ = luminance_min;
    }

    pub fn new() -> MasteringMetadata {
        MasteringMetadata {
            luminance_max_: Self::kValueNotPresent,
            luminance_min_: Self::kValueNotPresent,
            r_: None,
            g_: None,
            b_: None,
            white_point_: None,
        }
    }

    pub fn Size(&self) -> u64 {
        let mut size = self.PayloadSize();
        if size > 0 {
            size += util::EbmlMasterElementSize(MkvId::MkvMasteringMetadata, size);
        }
        size
    }

    pub fn Valid(&self) -> bool {
        if self.luminance_min_ != Self::kValueNotPresent {
            if self.luminance_min_ < Self::kMinLuminance
                || self.luminance_min_ > Self::kMinLuminanceMax
                || self.luminance_min_ > self.luminance_max_
            {
                return false;
            }
        }
        if self.luminance_max_ != Self::kValueNotPresent {
            if self.luminance_max_ < Self::kMinLuminance
                || self.luminance_max_ > Self::kMaxLuminanceMax
                || self.luminance_max_ < self.luminance_min_
            {
                return false;
            }
        }
        if self.r_.is_some() && !self.r_.as_ref().unwrap().Valid() {
            return false;
        }
        if self.g_.is_some() && !self.g_.as_ref().unwrap().Valid() {
            return false;
        }
        if self.b_.is_some() && !self.b_.as_ref().unwrap().Valid() {
            return false;
        }
        if self.white_point_.is_some() && !self.white_point_.as_ref().unwrap().Valid() {
            return false;
        }

        true
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let size = self.PayloadSize();

        // Don't write an empty element.
        if size == 0 {
            return true;
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvMasteringMetadata, size) {
            return false;
        }
        if self.luminance_max_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgF32(writer, MkvId::MkvLuminanceMax, self.luminance_max_)
        {
            return false;
        }
        if self.luminance_min_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgF32(writer, MkvId::MkvLuminanceMin, self.luminance_min_)
        {
            return false;
        }
        if self.r_.is_some()
            && !self.r_.as_ref().unwrap().Write(
                writer,
                MkvId::MkvPrimaryRChromaticityX,
                MkvId::MkvPrimaryRChromaticityY,
            )
        {
            return false;
        }
        if self.g_.is_some()
            && !self.g_.as_ref().unwrap().Write(
                writer,
                MkvId::MkvPrimaryGChromaticityX,
                MkvId::MkvPrimaryGChromaticityY,
            )
        {
            return false;
        }
        if self.b_.is_some()
            && !self.b_.as_ref().unwrap().Write(
                writer,
                MkvId::MkvPrimaryBChromaticityX,
                MkvId::MkvPrimaryBChromaticityY,
            )
        {
            return false;
        }
        if self.white_point_.is_some()
            && !self.white_point_.as_ref().unwrap().Write(
                writer,
                MkvId::MkvWhitePointChromaticityX,
                MkvId::MkvWhitePointChromaticityY,
            )
        {
            return false;
        }

        true
    }

    pub fn SetChromaticity(
        &mut self,
        r: &PrimaryChromaticity,
        g: &PrimaryChromaticity,
        b: &PrimaryChromaticity,
        white_point: &PrimaryChromaticity,
    ) -> bool {
        self.r_ = Some(r.clone());
        self.g_ = Some(g.clone());
        self.b_ = Some(b.clone());
        self.white_point_ = Some(white_point.clone());
        return true;
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut size = 0;

        if self.luminance_max_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgF32(MkvId::MkvLuminanceMax, self.luminance_max_);
        }
        if self.luminance_min_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgF32(MkvId::MkvLuminanceMin, self.luminance_min_);
        }

        if let Some(r) = self.r_.as_ref() {
            size += r.Size(
                MkvId::MkvPrimaryRChromaticityX,
                MkvId::MkvPrimaryRChromaticityY,
            );
        }
        if let Some(g) = self.g_.as_ref() {
            size += g.Size(
                MkvId::MkvPrimaryGChromaticityX,
                MkvId::MkvPrimaryGChromaticityY,
            );
        }
        if let Some(b) = self.b_.as_ref() {
            size += b.Size(
                MkvId::MkvPrimaryBChromaticityX,
                MkvId::MkvPrimaryBChromaticityY,
            );
        }
        if let Some(w) = self.white_point_.as_ref() {
            size += w.Size(
                MkvId::MkvWhitePointChromaticityX,
                MkvId::MkvWhitePointChromaticityY,
            );
        }

        size
    }
}

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

impl MatrixCoefficients {
    pub fn from_u64(value: u64) -> Option<MatrixCoefficients> {
        match value {
            0 => Some(MatrixCoefficients::kGbr),
            1 => Some(MatrixCoefficients::kBt709),
            2 => Some(MatrixCoefficients::kUnspecifiedMc),
            3 => Some(MatrixCoefficients::kReserved),
            4 => Some(MatrixCoefficients::kFcc),
            5 => Some(MatrixCoefficients::kBt470bg),
            6 => Some(MatrixCoefficients::kSmpte170MMc),
            7 => Some(MatrixCoefficients::kSmpte240MMc),
            8 => Some(MatrixCoefficients::kYcocg),
            9 => Some(MatrixCoefficients::kBt2020NonConstantLuminance),
            10 => Some(MatrixCoefficients::kBt2020ConstantLuminance),
            _ => None,
        }
    }
}
pub enum ChromaSitingHorz {
    kUnspecifiedCsh = 0,
    kLeftCollocated = 1,
    kHalfCsh = 2,
}

impl ChromaSitingHorz {
    pub fn from_u64(value: u64) -> Option<ChromaSitingHorz> {
        match value {
            0 => Some(ChromaSitingHorz::kUnspecifiedCsh),
            1 => Some(ChromaSitingHorz::kLeftCollocated),
            2 => Some(ChromaSitingHorz::kHalfCsh),
            _ => None,
        }
    }
}

pub enum ChromaSitingVert {
    kUnspecifiedCsv = 0,
    kTopCollocated = 1,
    kHalfCsv = 2,
}

impl ChromaSitingVert {
    pub fn from_u64(value: u64) -> Option<ChromaSitingVert> {
        match value {
            0 => Some(ChromaSitingVert::kUnspecifiedCsv),
            1 => Some(ChromaSitingVert::kTopCollocated),
            2 => Some(ChromaSitingVert::kHalfCsv),
            _ => None,
        }
    }
}

pub enum Range {
    kUnspecifiedCr = 0,
    kBroadcastRange = 1,
    kFullRange = 2,
    kMcTcDefined = 3, // Defined by MatrixCoefficients/TransferCharacteristics.
}

impl Range {
    pub fn from_u64(value: u64) -> Option<Range> {
        match value {
            0 => Some(Range::kUnspecifiedCr),
            1 => Some(Range::kBroadcastRange),
            2 => Some(Range::kFullRange),
            3 => Some(Range::kMcTcDefined),
            _ => None,
        }
    }
}

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

impl TransferCharacteristics {
    pub fn from_u64(value: u64) -> Option<TransferCharacteristics> {
        match value {
            1 => Some(TransferCharacteristics::kIturBt709Tc),
            2 => Some(TransferCharacteristics::kUnspecifiedTc),
            3 => Some(TransferCharacteristics::kReservedTc),
            4 => Some(TransferCharacteristics::kGamma22Curve),
            5 => Some(TransferCharacteristics::kGamma28Curve),
            6 => Some(TransferCharacteristics::kSmpte170MTc),
            7 => Some(TransferCharacteristics::kSmpte240MTc),
            8 => Some(TransferCharacteristics::kLinear),
            9 => Some(TransferCharacteristics::kLog),
            10 => Some(TransferCharacteristics::kLogSqrt),
            11 => Some(TransferCharacteristics::kIec6196624),
            12 => Some(TransferCharacteristics::kIturBt1361ExtendedColourGamut),
            13 => Some(TransferCharacteristics::kIec6196621),
            14 => Some(TransferCharacteristics::kIturBt202010bit),
            15 => Some(TransferCharacteristics::kIturBt202012bit),
            16 => Some(TransferCharacteristics::kSmpteSt2084),
            17 => Some(TransferCharacteristics::kSmpteSt4281Tc),
            18 => Some(TransferCharacteristics::kAribStdB67Hlg),
            _ => None,
        }
    }
}

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

impl Primaries {
    pub fn from_u64(value: u64) -> Option<Primaries> {
        match value {
            0 => Some(Primaries::kReservedP0),
            1 => Some(Primaries::kIturBt709P),
            2 => Some(Primaries::kUnspecifiedP),
            3 => Some(Primaries::kReservedP3),
            4 => Some(Primaries::kIturBt470M),
            5 => Some(Primaries::kIturBt470Bg),
            6 => Some(Primaries::kSmpte170MP),
            7 => Some(Primaries::kSmpte240MP),
            8 => Some(Primaries::kFilm),
            9 => Some(Primaries::kIturBt2020),
            10 => Some(Primaries::kSmpteSt4281P),
            22 => Some(Primaries::kJedecP22Phosphors),
            _ => None,
        }
    }
}

struct Colour {
    matrix_coefficients_: u64,
    bits_per_channel_: u64,
    chroma_subsampling_horz_: u64,
    chroma_subsampling_vert_: u64,
    cb_subsampling_horz_: u64,
    cb_subsampling_vert_: u64,
    chroma_siting_horz_: u64,
    chroma_siting_vert_: u64,
    range_: u64,
    transfer_characteristics_: u64,
    primaries_: u64,
    max_cll_: u64,
    max_fall_: u64,

    mastering_metadata_: Option<MasteringMetadata>,
}

impl Colour {
    const kValueNotPresent: u64 = std::u64::MAX;

    pub fn new() -> Colour {
        Colour {
            matrix_coefficients_: Self::kValueNotPresent,
            bits_per_channel_: Self::kValueNotPresent,
            chroma_subsampling_horz_: Self::kValueNotPresent,
            chroma_subsampling_vert_: Self::kValueNotPresent,
            cb_subsampling_horz_: Self::kValueNotPresent,
            cb_subsampling_vert_: Self::kValueNotPresent,
            chroma_siting_horz_: Self::kValueNotPresent,
            chroma_siting_vert_: Self::kValueNotPresent,
            range_: Self::kValueNotPresent,
            transfer_characteristics_: Self::kValueNotPresent,
            primaries_: Self::kValueNotPresent,
            max_cll_: Self::kValueNotPresent,
            max_fall_: Self::kValueNotPresent,
            mastering_metadata_: None,
        }
    }

    pub fn mastering_metadata(&self) -> Option<&MasteringMetadata> {
        self.mastering_metadata_.as_ref()
    }
    pub fn set_mastering_metadata(&mut self, mastering_metadata: &MasteringMetadata) {
        self.mastering_metadata_ = Some(mastering_metadata.clone());
    }
    pub fn matrix_coefficients(&self) -> u64 {
        self.matrix_coefficients_
    }
    pub fn set_matrix_coefficients(&mut self, matrix_coefficients: u64) {
        self.matrix_coefficients_ = matrix_coefficients;
    }
    pub fn bits_per_channel(&self) -> u64 {
        self.bits_per_channel_
    }
    pub fn set_bits_per_channel(&mut self, bits_per_channel: u64) {
        self.bits_per_channel_ = bits_per_channel;
    }
    pub fn chroma_subsampling_horz(&self) -> u64 {
        self.chroma_subsampling_horz_
    }
    pub fn set_chroma_subsampling_horz(&mut self, chroma_subsampling_horz: u64) {
        self.chroma_subsampling_horz_ = chroma_subsampling_horz;
    }
    pub fn chroma_subsampling_vert(&self) -> u64 {
        self.chroma_subsampling_vert_
    }
    pub fn set_chroma_subsampling_vert(&mut self, chroma_subsampling_vert: u64) {
        self.chroma_subsampling_vert_ = chroma_subsampling_vert;
    }
    pub fn cb_subsampling_horz(&self) -> u64 {
        self.cb_subsampling_horz_
    }
    pub fn set_cb_subsampling_horz(&mut self, cb_subsampling_horz: u64) {
        self.cb_subsampling_horz_ = cb_subsampling_horz;
    }
    pub fn cb_subsampling_vert(&self) -> u64 {
        self.cb_subsampling_vert_
    }
    pub fn set_cb_subsampling_vert(&mut self, cb_subsampling_vert: u64) {
        self.cb_subsampling_vert_ = cb_subsampling_vert;
    }
    pub fn chroma_siting_horz(&self) -> u64 {
        self.chroma_siting_horz_
    }
    pub fn set_chroma_siting_horz(&mut self, chroma_siting_horz: u64) {
        self.chroma_siting_horz_ = chroma_siting_horz;
    }
    pub fn chroma_siting_vert(&self) -> u64 {
        self.chroma_siting_vert_
    }
    pub fn set_chroma_siting_vert(&mut self, chroma_siting_vert: u64) {
        self.chroma_siting_vert_ = chroma_siting_vert;
    }
    pub fn range(&self) -> u64 {
        self.range_
    }
    pub fn set_range(&mut self, range: u64) {
        self.range_ = range;
    }
    pub fn transfer_characteristics(&self) -> u64 {
        self.transfer_characteristics_
    }
    pub fn set_transfer_characteristics(&mut self, transfer_characteristics: u64) {
        self.transfer_characteristics_ = transfer_characteristics;
    }
    pub fn primaries(&self) -> u64 {
        self.primaries_
    }
    pub fn set_primaries(&mut self, primaries: u64) {
        self.primaries_ = primaries;
    }
    pub fn max_cll(&self) -> u64 {
        self.max_cll_
    }
    pub fn set_max_cll(&mut self, max_cll: u64) {
        self.max_cll_ = max_cll;
    }
    pub fn max_fall(&self) -> u64 {
        self.max_fall_
    }
    pub fn set_max_fall(&mut self, max_fall: u64) {
        self.max_fall_ = max_fall;
    }

    pub fn Size(&self) -> u64 {
        let mut size = self.PayloadSize();
        if size > 0 {
            size += util::EbmlMasterElementSize(MkvId::MkvColour, size);
        }
        size
    }

    pub fn Valid(&self) -> bool {
        if self.mastering_metadata_.is_some() && !self.mastering_metadata_.as_ref().unwrap().Valid()
        {
            return false;
        }
        if self.matrix_coefficients_ != Self::kValueNotPresent
            && !MatrixCoefficients::from_u64(self.matrix_coefficients_).is_some()
        {
            return false;
        }
        if self.chroma_siting_horz_ != Self::kValueNotPresent
            && !ChromaSitingHorz::from_u64(self.chroma_siting_horz_).is_some()
        {
            return false;
        }
        if self.chroma_siting_vert_ != Self::kValueNotPresent
            && !ChromaSitingVert::from_u64(self.chroma_siting_vert_).is_some()
        {
            return false;
        }
        if self.range_ != Self::kValueNotPresent && !Range::from_u64(self.range_).is_some() {
            return false;
        }
        if self.transfer_characteristics_ != Self::kValueNotPresent
            && !TransferCharacteristics::from_u64(self.transfer_characteristics_).is_some()
        {
            return false;
        }
        if self.primaries_ != Self::kValueNotPresent
            && !Primaries::from_u64(self.primaries_).is_some()
        {
            return false;
        }

        true
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut size = 0;

        if self.matrix_coefficients_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(
                MkvId::MkvMatrixCoefficients,
                self.matrix_coefficients_,
            );
        }
        if self.bits_per_channel_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(MkvId::MkvBitsPerChannel, self.bits_per_channel_);
        }
        if self.chroma_subsampling_horz_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(
                MkvId::MkvChromaSubsamplingHorz,
                self.chroma_subsampling_horz_,
            );
        }
        if self.chroma_subsampling_vert_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(
                MkvId::MkvChromaSubsamplingVert,
                self.chroma_subsampling_vert_,
            );
        }
        if self.cb_subsampling_horz_ != Self::kValueNotPresent {
            size +=
                util::EbmlElementSizeArgU64(MkvId::MkvCbSubsamplingHorz, self.cb_subsampling_horz_);
        }
        if self.cb_subsampling_vert_ != Self::kValueNotPresent {
            size +=
                util::EbmlElementSizeArgU64(MkvId::MkvCbSubsamplingVert, self.cb_subsampling_vert_);
        }
        if self.chroma_siting_horz_ != Self::kValueNotPresent {
            size +=
                util::EbmlElementSizeArgU64(MkvId::MkvChromaSitingHorz, self.chroma_siting_horz_);
        }
        if self.chroma_siting_vert_ != Self::kValueNotPresent {
            size +=
                util::EbmlElementSizeArgU64(MkvId::MkvChromaSitingVert, self.chroma_siting_vert_);
        }
        if self.range_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(MkvId::MkvRange, self.range_);
        }
        if self.transfer_characteristics_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(
                MkvId::MkvTransferCharacteristics,
                self.transfer_characteristics_,
            );
        }
        if self.primaries_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(MkvId::MkvPrimaries, self.primaries_);
        }
        if self.max_cll_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(MkvId::MkvMaxCLL, self.max_cll_);
        }
        if self.max_fall_ != Self::kValueNotPresent {
            size += util::EbmlElementSizeArgU64(MkvId::MkvMaxFALL, self.max_fall_);
        }

        if let Some(mastering_metadata) = self.mastering_metadata_.as_ref() {
            size += mastering_metadata.Size();
        }

        size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let size = self.PayloadSize();

        // Don't write an empty element.
        if size == 0 {
            return true;
        }

        // Don't write an invalid element.
        if !self.Valid() {
            return false;
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvColour, size) {
            return false;
        }

        if self.matrix_coefficients_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvMatrixCoefficients,
                self.matrix_coefficients_,
            )
        {
            return false;
        }
        if self.bits_per_channel_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvBitsPerChannel,
                self.bits_per_channel_,
            )
        {
            return false;
        }
        if self.chroma_subsampling_horz_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvChromaSubsamplingHorz,
                self.chroma_subsampling_horz_,
            )
        {
            return false;
        }
        if self.chroma_subsampling_vert_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvChromaSubsamplingVert,
                self.chroma_subsampling_vert_,
            )
        {
            return false;
        }

        if self.cb_subsampling_horz_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvCbSubsamplingHorz,
                self.cb_subsampling_horz_,
            )
        {
            return false;
        }
        if self.cb_subsampling_vert_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvCbSubsamplingVert,
                self.cb_subsampling_vert_,
            )
        {
            return false;
        }
        if self.chroma_siting_horz_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvChromaSitingHorz,
                self.chroma_siting_horz_,
            )
        {
            return false;
        }
        if self.chroma_siting_vert_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvChromaSitingVert,
                self.chroma_siting_vert_,
            )
        {
            return false;
        }
        if self.range_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(writer, MkvId::MkvRange, self.range_)
        {
            return false;
        }
        if self.transfer_characteristics_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvTransferCharacteristics,
                self.transfer_characteristics_,
            )
        {
            return false;
        }
        if self.primaries_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(writer, MkvId::MkvPrimaries, self.primaries_)
        {
            return false;
        }
        if self.max_cll_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(writer, MkvId::MkvMaxCLL, self.max_cll_)
        {
            return false;
        }
        if self.max_fall_ != Self::kValueNotPresent
            && !util::WriteEbmlElementArgU64(writer, MkvId::MkvMaxFALL, self.max_fall_)
        {
            return false;
        }

        if self.mastering_metadata_.is_some()
            && !self.mastering_metadata_.as_ref().unwrap().Write(writer)
        {
            return false;
        }

        true
    }
}
