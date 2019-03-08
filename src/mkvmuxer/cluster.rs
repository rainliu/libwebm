use super::frame::Frame;
use super::util;
use super::writer::Writer;
use crate::MkvId;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashMap;

pub struct Cluster {
    // Number of blocks added to the cluster.
    blocks_added_: i32,

    // Flag telling if the cluster has been closed.
    finalized_: bool,

    // Flag indicating whether the cluster's timecode will always be written out
    // using 8 bytes.
    fixed_size_timecode_: bool,

    // Flag telling if the cluster's header has been written.
    header_written_: bool,

    // The size of the cluster elements in bytes.
    payload_size_: u64,

    // The file position used for cue points.
    position_for_cues_: i64,

    // The file position of the cluster's size element.
    size_position_: i64,

    // The absolute timecode of the cluster.
    timecode_: u64,

    // The timecode scale of the Segment containing the cluster.
    timecode_scale_: u64,

    // Flag indicating whether the last frame of the cluster should be written as
    // a Block with Duration. If set to true, then it will result in holding back
    // of frames and the parameterized version of Finalize() must be called to
    // finish writing the Cluster.
    write_last_frame_with_duration_: bool,

    // Map used to hold back frames, if required. Track number is the key.
    stored_frames_: HashMap<u64, Vec<Frame>>,

    // Map from track number to the timestamp of the last block written for that
    // track.
    last_block_timestamp_: HashMap<u64, u64>,
}

impl Cluster {
    pub fn new(
        timecode: u64,
        cues_pos: i64,
        timecode_scale: u64,
        write_last_frame_with_duration: bool,
        fixed_size_timecode: bool,
    ) -> Cluster {
        Cluster {
            blocks_added_: 0,
            finalized_: false,
            fixed_size_timecode_: fixed_size_timecode,
            header_written_: false,
            payload_size_: 0,
            position_for_cues_: cues_pos,
            size_position_: -1,
            timecode_: timecode,
            timecode_scale_: timecode_scale,
            write_last_frame_with_duration_: write_last_frame_with_duration,
            stored_frames_: HashMap::new(),
            last_block_timestamp_: HashMap::new(),
        }
    }
    pub fn size_position(&self) -> i64 {
        return self.size_position_;
    }
    pub fn blocks_added(&self) -> i32 {
        return self.blocks_added_;
    }
    pub fn payload_size(&self) -> u64 {
        return self.payload_size_;
    }
    pub fn position_for_cues(&self) -> i64 {
        return self.position_for_cues_;
    }
    pub fn timecode(&self) -> u64 {
        return self.timecode_;
    }
    pub fn timecode_scale(&self) -> u64 {
        return self.timecode_scale_;
    }
    pub fn set_write_last_frame_with_duration(&mut self, write_last_frame_with_duration: bool) {
        self.write_last_frame_with_duration_ = write_last_frame_with_duration;
    }
    pub fn write_last_frame_with_duration(&self) -> bool {
        return self.write_last_frame_with_duration_;
    }

    fn AddPayloadSize(&mut self, size: u64) {
        self.payload_size_ += size;
    }

    pub fn Size(&self) -> u64 {
        let element_size =
            util::EbmlMasterElementSize(MkvId::MkvCluster, 0xFFFFFFFFFFFFFFFF) + self.payload_size_;
        element_size
    }

    fn PreWriteBlock(&mut self, writer: &mut dyn Writer) -> bool {
        if self.finalized_ {
            return false;
        }

        if !self.header_written_ {
            if !self.WriteClusterHeader(writer) {
                return false;
            }
        }

        true
    }

    fn WriteClusterHeader(&mut self, writer: &mut dyn Writer) -> bool {
        if self.finalized_ {
            return false;
        }

        if let Err(e) = util::WriteID(writer, MkvId::MkvCluster) {
            return false;
        }

        // Save for later.
        self.size_position_ = writer.get_position() as i64;

        // Write "unknown" (EBML coded -1) as cluster size value. We need to write 8
        // bytes because we do not know how big our cluster will be.
        if let Err(e) = util::SerializeInt(writer, util::EBML_UNKNOWN_VALUE, 8) {
            return false;
        }
        let timecode_size = if self.fixed_size_timecode_ { 8 } else { 0 };
        let timecode = self.timecode();
        if !util::WriteEbmlElementArgsU64(writer, MkvId::MkvTimecode, timecode, timecode_size) {
            return false;
        }
        self.AddPayloadSize(util::EbmlElementSizeArgsU64(
            MkvId::MkvTimecode,
            timecode,
            timecode_size,
        ));
        self.header_written_ = true;

        true
    }

    fn PostWriteBlock(&mut self, element_size: u64) {
        self.AddPayloadSize(element_size);
        self.blocks_added_ += 1;
    }

    fn GetRelativeTimecode(&self, abs_timecode: i64) -> i64 {
        let cluster_timecode = self.timecode() as i64;
        let rel_timecode = abs_timecode - cluster_timecode;

        if rel_timecode < 0 || rel_timecode > util::MAX_BLOCK_TIMECODE {
            return -1;
        }

        return rel_timecode;
    }

    pub fn WriteFrame(&mut self, writer: &mut dyn Writer, frame: &Frame) -> u64 {
        if !frame.IsValid() || self.timecode_scale() == 0 {
            return 0;
        }

        //  Technically the timecode for a block can be less than the
        //  timecode for the cluster itself (remember that block timecode
        //  is a signed, 16-bit integer).  However, as a simplification we
        //  only permit non-negative cluster-relative timecodes for blocks.
        let relative_timecode =
            self.GetRelativeTimecode((frame.timestamp() / self.timecode_scale()) as i64);
        if relative_timecode < 0 || relative_timecode > util::MAX_BLOCK_TIMECODE {
            return 0;
        }

        if frame.CanBeSimpleBlock() {
            frame.WriteSimpleBlock(writer, relative_timecode)
        } else {
            let timecode_scale = self.timecode_scale();
            frame.WriteBlock(writer, relative_timecode, timecode_scale)
        }
    }

    fn DoWriteFrame(&mut self, writer: &mut dyn Writer, frame: &Frame) -> bool {
        if !frame.IsValid() {
            return false;
        }

        if !self.PreWriteBlock(writer) {
            return false;
        }

        let element_size = self.WriteFrame(writer, frame);
        if element_size == 0 {
            return false;
        }

        self.PostWriteBlock(element_size);
        self.last_block_timestamp_
            .insert(frame.track_number(), frame.timestamp());
        true
    }

    fn QueueOrWriteFrame(&mut self, writer: &mut dyn Writer, frame: &Frame) -> bool {
        if !frame.IsValid() {
            return false;
        }

        // If |write_last_frame_with_duration_| is not set, then write the frame right
        // away.
        if !self.write_last_frame_with_duration_ {
            return self.DoWriteFrame(writer, frame);
        }

        // Queue the current frame.
        let track_number = frame.track_number();
        let frame_to_store = frame.clone();

        // Iterate through all queued frames in the current track except the last one
        // and write it if it is okay to do so (i.e.) no other track has an held back
        // frame with timestamp <= the timestamp of the frame in question.
        if let Some(mut frames) = self.stored_frames_.remove(&track_number) {
            frames.retain(|frame_to_write| {
                let mut okay_to_write = true;
                for (key, val) in self.stored_frames_.iter() {
                    if *key == track_number {
                        continue;
                    } else if let Some(frame) = val.first() {
                        if frame.timestamp() < frame_to_write.timestamp() {
                            okay_to_write = false;
                            break;
                        }
                    }
                }
                if okay_to_write {
                    !self.DoWriteFrame(writer, frame_to_write)
                } else {
                    false
                }
            });

            frames.push(frame_to_store);
            self.stored_frames_.insert(track_number, frames);
        } else {
            self.stored_frames_
                .insert(track_number, vec![frame_to_store]);
        }

        return true;
    }

    pub fn AddNewFrame(
        &mut self,
        writer: &mut dyn Writer,
        data: &[u8],
        track_number: u64,
        abs_timecode: u64,
        is_key: bool,
    ) -> bool {
        let mut frame = Frame::new();
        if !frame.Init(data) {
            return false;
        }
        frame.set_track_number(track_number);
        frame.set_timestamp(abs_timecode);
        frame.set_is_key(is_key);
        return self.QueueOrWriteFrame(writer, &frame);
    }

    pub fn AddFrame(&mut self, writer: &mut dyn Writer, frame: &Frame) -> bool {
        return self.QueueOrWriteFrame(writer, frame);
    }

    pub fn AddFrameWithAdditional(
        &mut self,
        writer: &mut dyn Writer,
        data: &[u8],
        additional: &[u8],
        add_id: u64,
        track_number: u64,
        abs_timecode: u64,
        is_key: bool,
    ) -> bool {
        if additional.is_empty() {
            return false;
        }
        let mut frame = Frame::new();
        if !frame.Init(data) || !frame.AddAdditionalData(additional, add_id) {
            return false;
        }
        frame.set_track_number(track_number);
        frame.set_timestamp(abs_timecode);
        frame.set_is_key(is_key);
        return self.QueueOrWriteFrame(writer, &frame);
    }

    pub fn AddFrameWithDiscardPadding(
        &mut self,
        writer: &mut dyn Writer,
        data: &[u8],
        discard_padding: i64,
        track_number: u64,
        abs_timecode: u64,
        is_key: bool,
    ) -> bool {
        let mut frame = Frame::new();
        if !frame.Init(data) {
            return false;
        }
        frame.set_discard_padding(discard_padding);
        frame.set_track_number(track_number);
        frame.set_timestamp(abs_timecode);
        frame.set_is_key(is_key);
        return self.QueueOrWriteFrame(writer, &frame);
    }

    pub fn AddMetadata(
        &mut self,
        writer: &mut dyn Writer,
        data: &[u8],
        track_number: u64,
        abs_timecode: u64,
        duration_timecode: u64,
    ) -> bool {
        let mut frame = Frame::new();
        if !frame.Init(data) {
            return false;
        }
        frame.set_track_number(track_number);
        frame.set_timestamp(abs_timecode);
        frame.set_duration(duration_timecode);
        frame.set_is_key(true); // All metadata blocks are keyframes.
        return self.QueueOrWriteFrame(writer, &frame);
    }

    pub fn finalize(
        &mut self,
        writer: &mut dyn Writer,
        set_last_frame_duration: bool,
        duration: u64,
    ) -> bool {
        if self.finalized_ {
            return false;
        }

        if self.write_last_frame_with_duration_ {
            // Write out held back Frames. This essentially performs a k-way merge
            // across all tracks in the increasing order of timestamps.
            let mut min_heap = BinaryHeap::new();
            for (_, frames) in self.stored_frames_.iter_mut() {
                if !frames.is_empty() {
                    min_heap.push(Reverse(frames.remove(0)));
                }
            }
            while !min_heap.is_empty() {
                let mut frame = min_heap.pop().unwrap().0;

                // Set the duration if it's the last frame for the track.
                if set_last_frame_duration
                    && self.stored_frames_[&frame.track_number()].is_empty()
                    && !frame.duration_set()
                {
                    frame.set_duration(duration - frame.timestamp());
                    if !frame.is_key() && !frame.reference_block_timestamp_set() {
                        frame.set_reference_block_timestamp(
                            self.last_block_timestamp_[&frame.track_number()] as i64,
                        );
                    }
                }

                // Write the frame and remove it from |stored_frames_|.
                let wrote_frame = self.DoWriteFrame(writer, &frame);
                if let Some(frames) = self.stored_frames_.get_mut(&frame.track_number()) {
                    if !frames.is_empty() {
                        min_heap.push(Reverse(frames.remove(0)));
                    }
                }

                if !wrote_frame {
                    return false;
                }
            }
        }

        if self.size_position_ == -1 {
            return false;
        }

        if writer.seekable() {
            let pos = writer.get_position();

            if let Err(e) = writer.set_position(self.size_position_ as u64) {
                return false;
            }

            let payload_size = self.payload_size();
            if let Err(e) = util::WriteUIntSize(writer, payload_size, 8) {
                return false;
            }

            if let Err(e) = writer.set_position(pos) {
                return false;
            }
        }

        self.finalized_ = true;

        return true;
    }

    pub fn Finalize(&mut self, writer: &mut dyn Writer) -> bool {
        !self.write_last_frame_with_duration_ && self.finalize(writer, false, 0)
    }
}
