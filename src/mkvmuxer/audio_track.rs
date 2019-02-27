use super::track::Track;
use super::util;
use super::writer::Writer;
use crate::MkvId;

use std::ops::Deref;

pub struct AudioTrack {
    track_: Track,

    // Audio track element names.
    bit_depth_: u64,
    channels_: u64,
    sample_rate_: f64,
}

impl Deref for AudioTrack {
    type Target = Track;

    fn deref(&self) -> &Track {
        &self.track_
    }
}

impl AudioTrack {
    pub fn new() -> AudioTrack {
        AudioTrack {
            track_: Track::new(),
            bit_depth_: 0,
            channels_: 1,
            sample_rate_: 0.0,
        }
    }

    pub fn set_bit_depth(&mut self, bit_depth: u64) {
        self.bit_depth_ = bit_depth;
    }
    pub fn bit_depth(&self) -> u64 {
        return self.bit_depth_;
    }
    pub fn set_channels(&mut self, channels: u64) {
        self.channels_ = channels;
    }
    pub fn channels(&self) -> u64 {
        return self.channels_;
    }
    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate_ = sample_rate;
    }
    pub fn sample_rate(&self) -> f64 {
        return self.sample_rate_;
    }

    pub fn PayloadSize(&self) -> u64 {
        let parent_size = self.track_.PayloadSize();

        let mut size =
            util::EbmlElementSizeArgF32(MkvId::MkvSamplingFrequency, self.sample_rate_ as f32);
        size += util::EbmlElementSizeArgU64(MkvId::MkvChannels, self.channels_);
        if self.bit_depth_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvBitDepth, self.bit_depth_);
        }
        size += util::EbmlMasterElementSize(MkvId::MkvAudio, size);

        return parent_size + size;
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        if !self.track_.Write(writer) {
            return false;
        }

        // Calculate AudioSettings size.
        let mut size =
            util::EbmlElementSizeArgF32(MkvId::MkvSamplingFrequency, self.sample_rate_ as f32);
        size += util::EbmlElementSizeArgU64(MkvId::MkvChannels, self.channels_);
        if self.bit_depth_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvBitDepth, self.bit_depth_);
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvAudio, size) {
            return false;
        }

        let payload_position = writer.get_position();

        if !util::WriteEbmlElementArgF32(
            writer,
            MkvId::MkvSamplingFrequency,
            self.sample_rate_ as f32,
        ) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvChannels, self.channels_) {
            return false;
        }
        if self.bit_depth_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvBitDepth, self.bit_depth_) {
                return false;
            }
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != size {
            return false;
        }

        return true;
    }
}
