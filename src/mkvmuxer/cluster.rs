use super::util;
use super::writer::Writer;
use crate::MkvId;
/*
struct Cluster{
    // Number of blocks added to the cluster.
    int32_t blocks_added_;

    // Flag telling if the cluster has been closed.
    bool finalized_;

    // Flag indicating whether the cluster's timecode will always be written out
    // using 8 bytes.
    bool fixed_size_timecode_;

    // Flag telling if the cluster's header has been written.
    bool header_written_;

    // The size of the cluster elements in bytes.
    uint64_t payload_size_;

    // The file position used for cue points.
    const int64_t position_for_cues_;

    // The file position of the cluster's size element.
    int64_t size_position_;

    // The absolute timecode of the cluster.
    const uint64_t timecode_;

    // The timecode scale of the Segment containing the cluster.
    const uint64_t timecode_scale_;

    // Flag indicating whether the last frame of the cluster should be written as
    // a Block with Duration. If set to true, then it will result in holding back
    // of frames and the parameterized version of Finalize() must be called to
    // finish writing the Cluster.
    bool write_last_frame_with_duration_;

    // Map used to hold back frames, if required. Track number is the key.
    std::map<uint64_t, std::list<Frame*> > stored_frames_;

    // Map from track number to the timestamp of the last block written for that
    // track.
    std::map<uint64_t, uint64_t> last_block_timestamp_;

    // Pointer to the writer object. Not owned by this class.
    IMkvWriter* writer_;

}*/