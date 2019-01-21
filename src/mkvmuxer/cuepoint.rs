use super::util;
use super::writer::Writer;
use crate::common::MkvId;

///////////////////////////////////////////////////////////////
// Class to hold one cue point in a Cues element.
#[derive(Debug, Clone)]
pub struct CuePoint {
    // Absolute timecode according to the segment time base.
    time_: u64,

    // The Track element associated with the CuePoint.
    track_: u64,

    // The position of the Cluster containing the Block.
    cluster_pos_: u64,

    // Number of the Block within the Cluster, starting from 1.
    block_number_: u64,

    // If true the muxer will write out the block number for the cue if the
    // block number is different than the default of 1. Default is set to true.
    output_block_number_: bool,
}

impl CuePoint {
    pub fn set_time(&mut self, time: u64) {
        self.time_ = time;
    }
    pub fn time(&self) -> u64 {
        self.time_
    }
    pub fn set_track(&mut self, track: u64) {
        self.track_ = track;
    }
    pub fn track(&self) -> u64 {
        self.track_
    }
    pub fn set_cluster_pos(&mut self, cluster_pos: u64) {
        self.cluster_pos_ = cluster_pos;
    }
    pub fn cluster_pos(&self) -> u64 {
        self.cluster_pos_
    }
    pub fn set_block_number(&mut self, block_number: u64) {
        self.block_number_ = block_number;
    }
    pub fn block_number(&self) -> u64 {
        self.block_number_
    }
    pub fn set_output_block_number(&mut self, output_block_number: bool) {
        self.output_block_number_ = output_block_number;
    }
    pub fn output_block_number(&self) -> bool {
        self.output_block_number_
    }

    pub fn new() -> CuePoint {
        CuePoint {
            time_: 0,
            track_: 0,
            cluster_pos_: 0,
            block_number_: 1,
            output_block_number_: true,
        }
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        if self.track_ < 1 || self.cluster_pos_ < 1 {
            return false;
        }

        let mut size: u64 =
            util::EbmlElementSizeArgU64(MkvId::MkvCueClusterPosition as u64, self.cluster_pos_);
        size += util::EbmlElementSizeArgU64(MkvId::MkvCueTrack as u64, self.track_);
        if self.output_block_number_ && self.block_number_ > 1 {
            size +=
                util::EbmlElementSizeArgU64(MkvId::MkvCueBlockNumber as u64, self.block_number_);
        }
        let track_pos_size: u64 =
            util::EbmlMasterElementSize(MkvId::MkvCueTrackPositions as u64, size) + size;
        let payload_size: u64 =
            util::EbmlElementSizeArgU64(MkvId::MkvCueTime as u64, self.time_) + track_pos_size;

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvCuePoint as u64, payload_size) {
            return false;
        }

        let payload_position = writer.get_position();

        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvCueTime as u64, self.time_) {
            return false;
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvCueTrackPositions as u64, size) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvCueTrack as u64, self.track_) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(
            writer,
            MkvId::MkvCueClusterPosition as u64,
            self.cluster_pos_,
        ) {
            return false;
        }
        if self.output_block_number_ && self.block_number_ > 1 {
            if !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvCueBlockNumber as u64,
                self.block_number_,
            ) {
                return false;
            }
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != payload_size {
            return false;
        }

        true
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut size: u64 =
            util::EbmlElementSizeArgU64(MkvId::MkvCueClusterPosition as u64, self.cluster_pos_);
        size += util::EbmlElementSizeArgU64(MkvId::MkvCueTrack as u64, self.track_);
        if self.output_block_number_ && self.block_number_ > 1 {
            size +=
                util::EbmlElementSizeArgU64(MkvId::MkvCueBlockNumber as u64, self.block_number_);
        }
        let track_pos_size: u64 =
            util::EbmlMasterElementSize(MkvId::MkvCueTrackPositions as u64, size) + size;
        let payload_size: u64 =
            util::EbmlElementSizeArgU64(MkvId::MkvCueTime as u64, self.time_) + track_pos_size;

        return payload_size;
    }

    pub fn Size(&self) -> u64 {
        let payload_size: u64 = self.PayloadSize();
        util::EbmlElementSizeArgU64(MkvId::MkvCuePoint as u64, payload_size) + payload_size
    }
}
