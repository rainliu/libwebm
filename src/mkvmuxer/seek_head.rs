use super::util;
use super::writer::Writer;
use crate::MkvId;

const kSeekEntryCount: usize = 5;

struct SeekHead {
    seek_entry_id_: Vec<u32>,

    // Seek entry pos element list.
    seek_entry_pos_: Vec<u64>,

    // The file position of SeekHead element.
    start_pos_: i64,
}

impl SeekHead{
    pub fn new() ->SeekHead{
        SeekHead{
            seek_entry_id_:vec![0;kSeekEntryCount],
            seek_entry_pos_:vec![0;kSeekEntryCount],
            start_pos_:0,
        }
    }
}