use super::frame::Frame;
use super::util;
use super::writer::Writer;
use crate::MkvId;

use std::collections::HashMap;

struct Cluster<'a> {
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
    size_position_: u64,

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

    // Pointer to the writer object. Not owned by this class.
    writer_: &'a mut dyn Writer,
}

impl<'a> Cluster<'a> {
    pub fn new(
        timecode: u64,
        cues_pos: i64,
        timecode_scale: u64,
        write_last_frame_with_duration: bool,
        fixed_size_timecode: bool,
        writer: &'a mut dyn Writer,
    ) -> Cluster {
        Cluster {
            blocks_added_: 0,
            finalized_: false,
            fixed_size_timecode_: fixed_size_timecode,
            header_written_: false,
            payload_size_: 0,
            position_for_cues_: cues_pos,
            size_position_: 0,
            timecode_: timecode,
            timecode_scale_: timecode_scale,
            write_last_frame_with_duration_: write_last_frame_with_duration,
            stored_frames_: HashMap::new(),
            last_block_timestamp_: HashMap::new(),
            writer_: writer,
        }
    }
    pub fn size_position(&self) -> u64 {
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

    fn PreWriteBlock(&mut self) -> bool {
        if self.finalized_ {
            return false;
        }

        if !self.header_written_ {
            if !self.WriteClusterHeader() {
                return false;
            }
        }

        true
    }

    fn WriteClusterHeader(&mut self) -> bool {
        if self.finalized_ {
            return false;
        }

        if let Err(e) = util::WriteID(self.writer_, MkvId::MkvCluster) {
            return false;
        }

        // Save for later.
        self.size_position_ = self.writer_.get_position();

        // Write "unknown" (EBML coded -1) as cluster size value. We need to write 8
        // bytes because we do not know how big our cluster will be.
        if let Err(e) = util::SerializeInt(self.writer_, util::EBML_UNKNOWN_VALUE, 8) {
            return false;
        }
        let timecode_size = if self.fixed_size_timecode_ { 8 } else { 0 };
        let timecode = self.timecode();
        if !util::WriteEbmlElementArgsU64(self.writer_, MkvId::MkvTimecode, timecode, timecode_size)
        {
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

    pub fn WriteFrame(&mut self, frame: &Frame) -> u64 {
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
            frame.WriteSimpleBlock(self.writer_, relative_timecode)
        } else {
            let timecode_scale = self.timecode_scale();
            frame.WriteBlock(self.writer_, relative_timecode, timecode_scale)
        }
    }

    fn DoWriteFrame(&mut self, frame: &Frame) -> bool {
        if !frame.IsValid() {
            return false;
        }

        if !self.PreWriteBlock() {
            return false;
        }

        let element_size = self.WriteFrame(frame);
        if element_size == 0 {
            return false;
        }

        self.PostWriteBlock(element_size);
        self.last_block_timestamp_.insert(frame.track_number(), frame.timestamp());
        true
    }
    /*
        fn QueueOrWriteFrame(&mut self, frame:&Frame) ->bool {
            if !frame.IsValid() {
                return false;
            }

            // If |write_last_frame_with_duration_| is not set, then write the frame right
            // away.
            if !self.write_last_frame_with_duration_ {
                return self.DoWriteFrame(frame);
            }

            // Queue the current frame.
            let track_number = frame.track_number();
            let frame_to_store = frame.clone();
            self.stored_frames_[track_number].push(frame_to_store);

            // Iterate through all queued frames in the current track except the last one
            // and write it if it is okay to do so (i.e.) no other track has an held back
            // frame with timestamp <= the timestamp of the frame in question.
            std::vector<std::list<Frame*>::iterator> frames_to_erase;
            for (std::list<Frame*>::iterator
            current_track_iterator = stored_frames_[track_number].begin(),
            end = --stored_frames_[track_number].end();
            current_track_iterator != end; ++current_track_iterator) {
            const Frame* const frame_to_write = *current_track_iterator;
            bool okay_to_write = true;
            for (FrameMapIterator track_iterator = stored_frames_.begin();
            track_iterator != stored_frames_.end(); ++track_iterator) {
            if (track_iterator->first == track_number) {
            continue;
            }
            if (track_iterator->second.front()->timestamp() <
            frame_to_write->timestamp()) {
            okay_to_write = false;
            break;
            }
            }
            if (okay_to_write) {
            const bool wrote_frame = DoWriteFrame(frame_to_write);
            delete frame_to_write;
            if (!wrote_frame)
            return false;
            frames_to_erase.push_back(current_track_iterator);
            } else {
            break;
            }
            }
            for (std::vector<std::list<Frame*>::iterator>::iterator iterator =
            frames_to_erase.begin();
            iterator != frames_to_erase.end(); ++iterator) {
            stored_frames_[track_number].erase(*iterator);
            }
            return true;
        }
    */
    /*
    pub fn AddFrame(data: &[u8],
         track_number:u64,  abs_timecode:u64,
         is_key:bool) ->bool {
        let mut frame = Frame::new();
        if !frame.Init(data, length) {
            return false;
        }
        frame.set_track_number(track_number);
        frame.set_timestamp(abs_timecode);
        frame.set_is_key(is_key);
        return QueueOrWriteFrame(&frame);
    }*/
}
