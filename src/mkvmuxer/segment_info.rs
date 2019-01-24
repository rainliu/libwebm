use super::util;
use super::writer::Writer;
use crate::MkvId;

struct SegmentInfo {
    // Segment Information element names.
    // Initially set to -1 to signify that a duration has not been set and should
    // not be written out.
    duration_: f64,
    // Set to libwebm-%d.%d.%d.%d, major, minor, build, revision.
    muxing_app_: String,
    timecode_scale_: u64,
    // Initially set to libwebm-%d.%d.%d.%d, major, minor, build, revision.
    writing_app_: String,
    // LLONG_MIN when DateUTC is not set.
    date_utc_: i64,

    // The file position of the duration element.
    duration_pos_: i64,
}

impl SegmentInfo {
    pub fn new() -> SegmentInfo {
        SegmentInfo {
            duration_: -1.0,
            muxing_app_: String::new(),
            timecode_scale_: 1000000,
            writing_app_: String::new(),
            date_utc_: std::i64::MIN,
            duration_pos_: -1,
        }
    }
    pub fn set_duration(&mut self, duration: f64) {
        self.duration_ = duration;
    }
    pub fn duration(&self) -> f64 {
        self.duration_
    }
    pub fn set_muxing_app(&mut self, app: &str) {
        self.muxing_app_ = app.to_string();
    }
    pub fn muxing_app(&self) -> &str {
        &self.muxing_app_
    }
    pub fn set_timecode_scale(&mut self, scale: u64) {
        self.timecode_scale_ = scale;
    }
    pub fn timecode_scale(&self) -> u64 {
        self.timecode_scale_
    }
    pub fn set_writing_app(&mut self, app: &str) {
        self.writing_app_ = app.to_string();
    }
    pub fn writing_app(&self) -> &str {
        &self.writing_app_
    }
    pub fn set_date_utc(&mut self, date_utc: i64) {
        self.date_utc_ = date_utc;
    }
    pub fn date_utc(&self) -> i64 {
        self.date_utc_
    }

    pub fn Init(&mut self) -> bool {
        let mut major = 0;
        let mut minor = 0;
        let mut build = 0;
        let mut revision = 0;
        util::GetVersion(&mut major, &mut minor, &mut build, &mut revision);

        let temp = format!("libwebm-{}.{}.{}.{}", major, minor, build, revision);
        self.set_muxing_app(&temp);
        self.set_writing_app(&temp);
        return true;
    }

    pub fn Finalize(&self, writer: &mut dyn Writer) -> bool {
        if self.duration_ > 0.0 {
            if writer.seekable() {
                if self.duration_pos_ == -1 {
                    return false;
                }

                let pos = writer.get_position();

                if writer.set_position(self.duration_pos_ as u64).is_err() {
                    return false;
                }

                if !util::WriteEbmlElementArgF32(writer, MkvId::MkvDuration, self.duration_ as f32)
                {
                    return false;
                }

                if writer.set_position(pos).is_err() {
                    return false;
                }
            }
        }

        true
    }

    pub fn Write(&mut self, writer: &mut dyn Writer) -> bool {
        if self.muxing_app_.is_empty() || self.writing_app_.is_empty() {
            return false;
        }

        let mut size = util::EbmlElementSizeArgU64(MkvId::MkvTimecodeScale, self.timecode_scale_);
        if self.duration_ > 0.0 {
            size += util::EbmlElementSizeArgF32(MkvId::MkvDuration, self.duration_ as f32);
        }
        if self.date_utc_ != std::i64::MIN {
            size += util::EbmlDateElementSize(MkvId::MkvDateUTC);
        }
        size += util::EbmlElementSizeArgStr(MkvId::MkvMuxingApp, &self.muxing_app_);
        size += util::EbmlElementSizeArgStr(MkvId::MkvWritingApp, &self.writing_app_);

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvInfo, size) {
            return false;
        }

        let payload_position = writer.get_position();

        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvTimecodeScale, self.timecode_scale_) {
            return false;
        }

        if self.duration_ > 0.0 {
            // Save for later
            self.duration_pos_ = writer.get_position() as i64;

            if !util::WriteEbmlElementArgF32(writer, MkvId::MkvDuration, self.duration_ as f32) {
                return false;
            }
        }

        if self.date_utc_ != std::i64::MIN {
            util::WriteEbmlDateElement(writer, MkvId::MkvDateUTC, self.date_utc_);
        }

        if !util::WriteEbmlElementArgStr(writer, MkvId::MkvMuxingApp, &self.muxing_app_) {
            return false;
        }
        if !util::WriteEbmlElementArgStr(writer, MkvId::MkvWritingApp, &self.writing_app_) {
            return false;
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != size {
            return false;
        }

        true
    }
}
