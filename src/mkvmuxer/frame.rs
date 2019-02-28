use super::util;
use super::writer::Writer;
use crate::MkvId;

const MAX_TRACK_NUMBER: u64 = 126;

// Class to hold data the will be written to a block.
#[derive(Debug, Clone)]
pub struct Frame {
    // Id of the Additional data.
    add_id_: u64,

    // Pointer to additional data. Owned by this class.
    additional_: Vec<u8>,

    // Duration of the frame in nanoseconds.
    duration_: u64,

    // Flag indicating that |duration_| has been set. Setting duration causes the
    // frame to be written out as a Block with BlockDuration instead of as a
    // SimpleBlock.
    duration_set_: bool,

    // Pointer to the data. Owned by this class.
    frame_: Vec<u8>,

    // Flag telling if the data should set the key flag of a block.
    is_key_: bool,

    // Mkv track number the data is associated with.
    track_number_: u64,

    // Timestamp of the data in nanoseconds.
    timestamp_: u64,

    // Discard padding for the frame.
    discard_padding_: i64,

    // Reference block timestamp.
    reference_block_timestamp_: i64,

    // Flag indicating if |reference_block_timestamp_| has been set.
    reference_block_timestamp_set_: bool,
}

impl Frame {
    pub fn add_id(&self) -> u64 {
        self.add_id_
    }
    pub fn additional(&self) -> &[u8] {
        &self.additional_
    }
    pub fn additional_length(&self) -> u64 {
        self.additional_.len() as u64
    }
    pub fn set_duration(&mut self, duration: u64) {
        self.duration_ = duration;
        self.duration_set_ = true;
    }
    pub fn duration(&self) -> u64 {
        self.duration_
    }
    pub fn duration_set(&self) -> bool {
        self.duration_set_
    }
    pub fn frame(&self) -> &[u8] {
        &self.frame_
    }
    pub fn set_is_key(&mut self, key: bool) {
        self.is_key_ = key;
    }
    pub fn is_key(&self) -> bool {
        self.is_key_
    }
    pub fn length(&self) -> u64 {
        self.frame_.len() as u64
    }
    pub fn set_track_number(&mut self, track_number: u64) {
        self.track_number_ = track_number;
    }
    pub fn track_number(&self) -> u64 {
        self.track_number_
    }
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp_ = timestamp;
    }
    pub fn timestamp(&self) -> u64 {
        self.timestamp_
    }
    pub fn set_discard_padding(&mut self, discard_padding: i64) {
        self.discard_padding_ = discard_padding;
    }
    pub fn discard_padding(&self) -> i64 {
        self.discard_padding_
    }
    pub fn set_reference_block_timestamp(&mut self, reference_block_timestamp: i64) {
        self.reference_block_timestamp_ = reference_block_timestamp;
        self.reference_block_timestamp_set_ = true;
    }
    pub fn reference_block_timestamp(&self) -> i64 {
        self.reference_block_timestamp_
    }
    pub fn reference_block_timestamp_set(&self) -> bool {
        self.reference_block_timestamp_set_
    }

    pub fn new() -> Frame {
        Frame {
            add_id_: 0,
            additional_: Vec::new(),
            duration_: 0,
            duration_set_: false,
            frame_: Vec::new(),
            is_key_: false,
            track_number_: 0,
            timestamp_: 0,
            discard_padding_: 0,
            reference_block_timestamp_: 0,
            reference_block_timestamp_set_: false,
        }
    }

    pub fn Init(&mut self, frame: &[u8], length: u64) -> bool {
        self.frame_ = frame.to_vec();
        true
    }

    pub fn AddAdditionalData(&mut self, additional: &[u8], add_id: u64) -> bool {
        self.additional_ = additional.to_vec();
        self.add_id_ = add_id;
        true
    }

    pub fn IsValid(&self) -> bool {
        if self.frame_.is_empty() {
            return false;
        }
        if !self.additional_.is_empty() {
            return false;
        }
        if self.track_number_ == 0 || self.track_number_ > MAX_TRACK_NUMBER {
            return false;
        }
        if !self.CanBeSimpleBlock() && !self.is_key_ && !self.reference_block_timestamp_set_ {
            return false;
        }
        return true;
    }

    pub fn CanBeSimpleBlock(&self) -> bool {
        self.additional_.len() == 0 && self.discard_padding_ == 0 && self.duration_ == 0
    }

    pub fn WriteSimpleBlock(&self, writer: &mut dyn Writer, timecode: i64) -> u64 {
        if let Err(e) = util::WriteID(writer, MkvId::MkvSimpleBlock) {
            return 0;
        }

        let size = self.length() + 4;
        if let Err(e) = util::WriteUInt(writer, size) {
            return 0;
        }

        if let Err(e) = util::WriteUInt(writer, self.track_number()) {
            return 0;
        }

        if let Err(e) = util::SerializeInt(writer, timecode as u64, 2) {
            return 0;
        }

        let mut flags = 0;
        if self.is_key() {
            flags |= 0x80;
        }

        if let Err(e) = util::SerializeInt(writer, flags, 1) {
            return 0;
        }

        if let Err(e) = writer.write(self.frame()) {
            return 0;
        }

        return util::GetUIntSize(MkvId::MkvSimpleBlock as u64) as u64
            + util::GetCodedUIntSize(size) as u64
            + 4
            + self.length();
    }

    pub fn WriteBlock(&self, writer: &mut dyn Writer, timecode: i64, timecode_scale: u64) -> u64 {
        0
    }
}
