use super::cue_point::CuePoint;
use super::util;
use super::writer::Writer;
use crate::MkvId;

///////////////////////////////////////////////////////////////
// Cues element.
#[derive(Debug, Clone)]
struct Cues {
    // CuePoint list.
    cue_entries_: Vec<CuePoint>,

    // If true the muxer will write out the block number for the cue if the
    // block number is different than the default of 1. Default is set to true.
    output_block_number_: bool,
}

impl Cues {
    pub fn cue_entries_size(&self) -> usize {
        self.cue_entries_.len()
    }
    pub fn set_output_block_number(&mut self, output_block_number: bool) {
        self.output_block_number_ = output_block_number;
    }
    pub fn output_block_number(&self) -> bool {
        self.output_block_number_
    }

    pub fn new() -> Cues {
        Cues {
            cue_entries_: Vec::new(),
            output_block_number_: true,
        }
    }

    pub fn AddCue(&mut self, cue: CuePoint) -> bool {
        let mut cue = cue;
        cue.set_output_block_number(self.output_block_number_);
        self.cue_entries_.push(cue);
        true
    }

    pub fn GetCueByIndex(&self, index: usize) -> Option<&CuePoint> {
        if index >= self.cue_entries_.len() {
            return None;
        }

        Some(&self.cue_entries_[index])
    }

    pub fn Size(&self) -> u64 {
        let mut size: u64 = 0;
        for i in 0..self.cue_entries_.len() {
            size += self.cue_entries_[i].Size();
        }
        size += util::EbmlMasterElementSize(MkvId::MkvCues, size);
        size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let mut size: u64 = 0;
        for i in 0..self.cue_entries_.len() {
            size += self.cue_entries_[i].Size();
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvCues, size) {
            return false;
        }

        let payload_position = writer.get_position();
        for i in 0..self.cue_entries_.len() {
            if !self.cue_entries_[i].Write(writer) {
                return false;
            }
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != size {
            return false;
        }

        true
    }
}
