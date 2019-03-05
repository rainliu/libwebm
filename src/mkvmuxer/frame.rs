use super::util;
use super::writer::Writer;
use crate::MkvId;

const MAX_TRACK_NUMBER: u64 = 126;

// Class to hold data the will be written to a block.
#[derive(Debug, Clone, Eq, PartialEq)]
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

use std::cmp::Ordering;
impl Ord for Frame {
    fn cmp(&self, other: &Frame) -> Ordering {
        self.timestamp_.cmp(&other.timestamp_)
    }
}

impl PartialOrd for Frame {
    fn partial_cmp(&self, other: &Frame) -> Option<Ordering> {
        Some(self.timestamp_.cmp(&other.timestamp_))
    }
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

    pub fn Init(&mut self, frame: &[u8]) -> bool {
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
        let mut block_additional_elem_size = 0;
        let mut block_addid_elem_size = 0;
        let mut block_more_payload_size = 0;
        let mut block_more_elem_size = 0;
        let mut block_additions_payload_size = 0;
        let mut block_additions_elem_size = 0;
        if !self.additional().is_empty() {
            block_additional_elem_size =
                util::EbmlElementSizeArgSlice(MkvId::MkvBlockAdditional, self.additional());
            block_addid_elem_size =
                util::EbmlElementSizeArgU64(MkvId::MkvBlockAddID, self.add_id());

            block_more_payload_size = block_addid_elem_size + block_additional_elem_size;
            block_more_elem_size =
                util::EbmlMasterElementSize(MkvId::MkvBlockMore, block_more_payload_size)
                    + block_more_payload_size;
            block_additions_payload_size = block_more_elem_size;
            block_additions_elem_size =
                util::EbmlMasterElementSize(MkvId::MkvBlockAdditions, block_additions_payload_size)
                    + block_additions_payload_size;
        }

        let mut discard_padding_elem_size = 0;
        if self.discard_padding() != 0 {
            discard_padding_elem_size =
                util::EbmlElementSizeArgI64(MkvId::MkvDiscardPadding, self.discard_padding());
        }

        let reference_block_timestamp = self.reference_block_timestamp() as u64 / timecode_scale;
        let mut reference_block_elem_size = 0;
        if !self.is_key() {
            reference_block_elem_size =
                util::EbmlElementSizeArgU64(MkvId::MkvReferenceBlock, reference_block_timestamp);
        }

        let duration = self.duration() / timecode_scale;
        let mut block_duration_elem_size = 0;
        if duration > 0 {
            block_duration_elem_size =
                util::EbmlElementSizeArgU64(MkvId::MkvBlockDuration, duration);
        }

        let block_payload_size = 4 + self.length();
        let block_elem_size =
            util::EbmlMasterElementSize(MkvId::MkvBlock, block_payload_size) + block_payload_size;

        let block_group_payload_size = block_elem_size
            + block_additions_elem_size
            + block_duration_elem_size
            + discard_padding_elem_size
            + reference_block_elem_size;

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvBlockGroup, block_group_payload_size) {
            return 0;
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvBlock, block_payload_size) {
            return 0;
        }

        if let Err(e) = util::WriteUInt(writer, self.track_number()) {
            return 0;
        }

        if let Err(e) = util::SerializeInt(writer, timecode as u64, 2) {
            return 0;
        }

        // For a Block, flags is always 0.
        if let Err(e) = util::SerializeInt(writer, 0, 1) {
            return 0;
        }

        if let Err(e) = writer.write(self.frame()) {
            return 0;
        }

        if !self.additional().is_empty() {
            if !util::WriteEbmlMasterElement(
                writer,
                MkvId::MkvBlockAdditions,
                block_additions_payload_size,
            ) {
                return 0;
            }

            if !util::WriteEbmlMasterElement(writer, MkvId::MkvBlockMore, block_more_payload_size) {
                return 0;
            }

            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvBlockAddID, self.add_id()) {
                return 0;
            }

            if !util::WriteEbmlElementArgSlice(writer, MkvId::MkvBlockAdditional, self.additional())
            {
                return 0;
            }
        }

        if self.discard_padding() != 0
            && !util::WriteEbmlElementArgI64(
                writer,
                MkvId::MkvDiscardPadding,
                self.discard_padding(),
            )
        {
            return 0;
        }

        if !self.is_key()
            && !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvReferenceBlock,
                reference_block_timestamp,
            )
        {
            return 0;
        }

        if duration > 0 && !util::WriteEbmlElementArgU64(writer, MkvId::MkvBlockDuration, duration)
        {
            return 0;
        }

        util::EbmlMasterElementSize(MkvId::MkvBlockGroup, block_group_payload_size)
            + block_group_payload_size
    }
}
