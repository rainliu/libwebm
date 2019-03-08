use super::chapter::Chapter;
use super::cue_point::CuesPosition;
use super::cues::Cues;
use super::seek_head::SeekHead;
use super::segment_info::SegmentInfo;
use super::util;
use super::write::MkvWriter;
use super::writer::Writer;
use crate::MkvId;

#[derive(Debug, Copy, Clone)]
enum Mode {
    kLive = 0x1,
    kFile = 0x2,
}

#[derive(Debug, Copy, Clone)]
enum CuesPosition {
    kAfterClusters = 0x0,  // Position Cues after Clusters - Default
    kBeforeClusters = 0x1, // Position Cues before Clusters
}

const kDefaultDocTypeVersion: u32 = 4;
const kDefaultMaxClusterDuration: u64 = 30000000000;

struct Segment<'a> {
    // Seeds the random number generator used to make UIDs.
    seed_: usize,

    // WebM elements
    cues_: Cues,
    seek_head_: SeekHead,
    segment_info_: SegmentInfo,
    tracks_: Tracks,
    chapters_: Chapters,
    tags_: Tags,

    // Number of chunks written.
    chunk_count_: isize,

    // Current chunk filename.
    chunk_name_: String,

    // Default MkvWriter object created by this class used for writing clusters
    // out in separate files.
    chunk_writer_cluster_: Option<MkvWriter>,

    // Default MkvWriter object created by this class used for writing Cues
    // element out to a file.
    chunk_writer_cues_: Option<MkvWriter>,

    // Default MkvWriter object created by this class used for writing the
    // Matroska header out to a file.
    chunk_writer_header_: Option<MkvWriter>,

    // Flag telling whether or not the muxer is chunking output to multiple
    // files.
    chunking_: bool,

    // Base filename for the chunked files.
    chunking_base_name_: String,

    // File position offset where the Clusters end.
    cluster_end_offset_: i64,

    // List of clusters.
    cluster_list_: Vec<&'a Cluster>,

    // Indicates whether Cues should be written before or after Clusters
    cues_position_: CuesPosition,

    // Track number that is associated with the cues element for this segment.
    cues_track_: u64,

    // Tells the muxer to force a new cluster on the next Block.
    force_new_cluster_: bool,

    // List of stored audio frames. These variables are used to store frames so
    // the muxer can follow the guideline "Audio blocks that contain the video
    // key frame's timecode should be in the same cluster as the video key frame
    // block."
    frames_: Vec<&'a Frame>,

    // Number of frame pointers allocated in the frame list.
    frames_capacity_: i32,

    // Number of frames in the frame list.
    frames_size_: i32,

    // Flag telling if a video track has been added to the segment.
    has_video_: bool,

    // Flag telling if the segment's header has been written.
    header_written_: bool,

    // Duration of the last block in nanoseconds.
    last_block_duration_: u64,

    // Last timestamp in nanoseconds added to a cluster.
    last_timestamp_: u64,

    // Last timestamp in nanoseconds by track number added to a cluster.
    last_track_timestamp_: [u64; kMaxTrackNumber],

    // Number of frames written per track.
    track_frames_written_: [u64; kMaxTrackNumber],

    // Maximum time in nanoseconds for a cluster duration. This variable is a
    // guideline and some clusters may have a longer duration. Default is 30
    // seconds.
    max_cluster_duration_: u64,

    // Maximum size in bytes for a cluster. This variable is a guideline and
    // some clusters may have a larger size. Default is 0 which signifies that
    // the muxer will decide the size.
    max_cluster_size_: u64,

    // The mode that segment is in. If set to |kLive| the writer must not
    // seek backwards.
    mode_: Mode,

    // Flag telling the muxer that a new cue point should be added.
    new_cuepoint_: bool,

    // Flag whether or not the muxer should output a Cues element.
    output_cues_: bool,

    // Flag whether or not the last frame in each Cluster will have a Duration
    // element in it.
    accurate_cluster_duration_: bool,

    // Flag whether or not to write the Cluster Timecode using exactly 8 bytes.
    fixed_size_cluster_timecode_: bool,

    // Flag whether or not to estimate the file duration.
    estimate_file_duration_: bool,

    // The size of the EBML header, used to validate the header if
    // WriteEbmlHeader() is called more than once.
    ebml_header_size_: i32,

    // The file position of the segment's payload.
    payload_pos_: i64,

    // The file position of the element's size.
    size_position_: i64,

    // Current DocTypeVersion (|doc_type_version_|) and that written in
    // WriteSegmentHeader().
    // WriteEbmlHeader() will be called from Finalize() if |doc_type_version_|
    // differs from |doc_type_version_written_|.
    doc_type_version_: u32,
    doc_type_version_written_: u32,

    // If |duration_| is > 0, then explicitly set the duration of the segment.
    duration_: f64,
    // Pointer to the writer objects. Not owned by this class.
    //writer_cluster_: &'a mut dyn Writer,
    //writer_cues_: &'a mut dyn Writer,
    //writer_header_: &'a mut dyn Writer,
}

impl Segment {
    pub fn new() -> Segment {
        Segment {
            chunk_count_: 0,
            chunk_name_: String::new(),
            chunk_writer_cluster_: None,
            chunk_writer_cues_: None,
            chunk_writer_header_: None,
            chunking_: false,
            chunking_base_name_: String::new(),
            cluster_list_: vec![],
            cues_position_: CuesPosition::kAfterClusters,
            cues_track_: 0,
            force_new_cluster_: false,
            frames_: vec![],
            has_video_: false,
            header_written_: false,
            last_block_duration_: 0,
            last_timestamp_: 0,
            max_cluster_duration_: kDefaultMaxClusterDuration,
            max_cluster_size_: 0,
            mode_: Mode::kFile,
            new_cuepoint_: false,
            output_cues_: true,
            accurate_cluster_duration_: false,
            fixed_size_cluster_timecode_: false,
            estimate_file_duration_: false,
            payload_pos_: 0,
            size_position_: 0,
            doc_type_version_: kDefaultDocTypeVersion,
            doc_type_version_written_: 0,
            duration_: 0.0,
            //writer_cluster_(NULL),
            //writer_cues_(NULL),
            //writer_header_(NULL)
        }
    }

    // Returns the Cues object.
    pub fn GetCues(&self) -> &Cues {
        return &self.cues_;
    }

    // Returns the Segment Information object.
    pub fn GetSegmentInfo(&self) -> &SegmentInfo {
        return &self.segment_info_;
    }

    pub fn chunking(&self) -> bool {
        return self.chunking_;
    }
    pub fn cues_track(&self) -> u64 {
        return self.cues_track_;
    }
    pub fn set_max_cluster_duration(&mut self, max_cluster_duration: u64) {
        self.max_cluster_duration_ = max_cluster_duration;
    }
    pub fn max_cluster_duration(&self) -> u64 {
        return self.max_cluster_duration_;
    }
    pub fn set_max_cluster_size(&mut self, max_cluster_size: u64) {
        self.max_cluster_size_ = max_cluster_size;
    }
    pub fn max_cluster_size(&self) -> u64 {
        return self.max_cluster_size_;
    }
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode_ = mode;
    }
    pub fn mode(&self) -> Mode {
        return self.mode_;
    }
    pub fn cues_position(&self) -> CuesPosition {
        return self.cues_position_;
    }
    pub fn output_cues(&self) -> bool {
        return self.output_cues_;
    }
    pub fn set_estimate_file_duration(&mut self, estimate_duration: bool) {
        self.estimate_file_duration_ = estimate_duration;
    }
    pub fn estimate_file_duration(&self) -> bool {
        return self.estimate_file_duration_;
    }
    pub fn segment_info(&self) -> &SegmentInfo {
        return &self.segment_info_;
    }
    pub fn set_duration(&mut self, duration: f64) {
        self.duration_ = duration;
    }
    pub fn duration(&self) -> f64 {
        return self.duration_;
    }


}
