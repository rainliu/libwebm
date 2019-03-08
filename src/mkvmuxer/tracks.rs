use super::track::Track;
use super::util;
use super::writer::Writer;
use crate::MkvId;

pub const kOpusCodecId: &'static str = "A_OPUS";
pub const kVorbisCodecId: &'static str = "A_VORBIS";
pub const kAv1CodecId: &'static str = "V_AV1";
pub const kVp8CodecId: &'static str = "V_VP8";
pub const kVp9CodecId: &'static str = "V_VP9";
pub const kWebVttCaptionsId: &'static str = "D_WEBVTT/CAPTIONS";
pub const kWebVttDescriptionsId: &'static str = "D_WEBVTT/DESCRIPTIONS";
pub const kWebVttMetadataId: &'static str = "D_WEBVTT/METADATA";
pub const kWebVttSubtitlesId: &'static str = "D_WEBVTT/SUBTITLES";

enum TrackType {
    kVideo = 0x1,
    kAudio = 0x2,
}

pub struct Tracks {
    // Track element list.
    track_entries_: Vec<Track>,

    // Whether or not Tracks element has already been written via IMkvWriter.
    wrote_tracks_: bool,
}

impl Tracks {
    pub fn new() -> Tracks {
        Tracks {
            track_entries_: Vec::new(),
            wrote_tracks_: false,
        }
    }

    pub fn AddTrack(&mut self, track: Track, number: i32) -> bool {
        if number < 0 || self.wrote_tracks_ {
            return false;
        }

        // This muxer only supports track numbers in the range [1, 126], in
        // order to be able (to use Matroska integer representation) to
        // serialize the block header (of which the track number is a part)
        // for a frame using exactly 4 bytes.

        if number > 0x7E {
            return false;
        }

        let mut track_num = number as u64;

        if track_num > 0 {
            // Check to make sure a track does not already have |track_num|.
            for t in &self.track_entries_ {
                if t.number() == track_num {
                    return false;
                }
            }
        }

        let count = self.track_entries_.len() as u64 + 1;

        // Find the lowest availible track number > 0.
        if track_num == 0 {
            track_num = count;

            // Check to make sure a track does not already have |track_num|.
            let mut exit = false;
            while !exit {
                exit = true;
                for t in &self.track_entries_ {
                    if t.number() == track_num {
                        track_num += 1;
                        exit = false;
                        break;
                    }
                }
            }
        }

        let mut track = track;
        track.set_number(track_num);
        self.track_entries_.push(track);
        return true;
    }

    pub fn GetTrackByIndex(&self, index: usize) -> Option<&Track> {
        if index >= self.track_entries_.len() {
            None
        } else {
            Some(&self.track_entries_[index])
        }
    }

    pub fn GetTrackByNumber(&self, track_number: u64) -> Option<&Track> {
        for t in &self.track_entries_ {
            if t.number() == track_number {
                return Some(t);
            }
        }
        None
    }

    pub fn TrackIsAudio(&self, track_number: u64) -> bool {
        let track = self.GetTrackByNumber(track_number);

        if let Some(t) = track {
            if t.track_type() == TrackType::kAudio as u64 {
                return true;
            }
        }
        false
    }

    pub fn TrackIsVideo(&self, track_number: u64) -> bool {
        let track = self.GetTrackByNumber(track_number);

        if let Some(t) = track {
            if t.track_type() == TrackType::kVideo as u64 {
                return true;
            }
        }
        false
    }

    pub fn Write(&mut self, writer: &mut dyn Writer) -> bool {
        let mut size = 0;
        for track in &self.track_entries_ {
            size += track.Size();
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvTracks, size) {
            return false;
        }

        let payload_position = writer.get_position();

        for track in &self.track_entries_ {
            if !track.Write(writer) {
                return false;
            }
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != size {
            return false;
        }

        self.wrote_tracks_ = true;
        true
    }
}
