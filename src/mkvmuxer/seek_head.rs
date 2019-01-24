use super::util;
use super::writer::Writer;
use crate::MkvId;

const kSeekEntryCount: usize = 5;

struct SeekHead {
    seek_entry_id_: Vec<u32>,

    // Seek entry pos element list.
    seek_entry_pos_: Vec<u64>,

    // The file position of SeekHead element.
    start_pos_: u64,
}

impl SeekHead {
    pub fn new() -> SeekHead {
        SeekHead {
            seek_entry_id_: vec![0; kSeekEntryCount],
            seek_entry_pos_: vec![0; kSeekEntryCount],
            start_pos_: 0,
        }
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let entry_size = kSeekEntryCount as u64 * self.MaxEntrySize();
        let size = util::EbmlMasterElementSize(MkvId::MkvSeekHead, entry_size);

        let start_pos_ = writer.get_position();
        let bytes_written = util::WriteVoidElement(writer, size + entry_size);
        if bytes_written == 0 {
            return false;
        }

        true
    }

    pub fn AddSeekEntry(&mut self, id: u32, pos: u64) -> bool {
        for i in 0..kSeekEntryCount {
            if self.seek_entry_id_[i] == 0 {
                self.seek_entry_id_[i] = id;
                self.seek_entry_pos_[i] = pos;
                return true;
            }
        }
        false
    }

    pub fn GetId(&self, index: usize) -> u32 {
        if index >= kSeekEntryCount {
            std::u32::MAX
        } else {
            self.seek_entry_id_[index]
        }
    }

    pub fn GetPosition(&self, index: usize) -> u64 {
        if index >= kSeekEntryCount {
            std::u64::MAX
        } else {
            self.seek_entry_pos_[index]
        }
    }

    pub fn SetSeekEntry(&mut self, index: usize, id: u32, position: u64) -> bool {
        if index >= kSeekEntryCount {
            return false;
        }
        self.seek_entry_id_[index] = id;
        self.seek_entry_pos_[index] = position;
        true
    }

    pub fn MaxEntrySize(&self) -> u64 {
        let max_entry_payload_size = util::EbmlElementSizeArgU64(MkvId::MkvSeekID, 0xffffffff)
            + util::EbmlElementSizeArgU64(MkvId::MkvSeekPosition, 0xffffffffffffffff);
        let max_entry_size = util::EbmlMasterElementSize(MkvId::MkvSeek, max_entry_payload_size)
            + max_entry_payload_size;
        max_entry_size
    }

    pub fn Finalize(&self, writer: &mut dyn Writer) -> bool {
        if writer.seekable() {
            //if self.start_pos_ == -1 {
            //    return false;
            //}

            let mut payload_size = 0;
            let mut entry_size = vec![0u64; kSeekEntryCount];

            for i in 0..kSeekEntryCount {
                if self.seek_entry_id_[i] != 0 {
                    entry_size[i] = util::EbmlElementSizeArgU64(
                        MkvId::MkvSeekID,
                        self.seek_entry_id_[i] as u64,
                    );
                    entry_size[i] += util::EbmlElementSizeArgU64(
                        MkvId::MkvSeekPosition,
                        self.seek_entry_pos_[i],
                    );

                    payload_size +=
                        util::EbmlMasterElementSize(MkvId::MkvSeek, entry_size[i]) + entry_size[i];
                }
            }

            // No SeekHead elements
            if payload_size == 0 {
                return true;
            }

            let pos = writer.get_position();
            if writer.set_position(self.start_pos_).is_err() {
                return false;
            }

            if !util::WriteEbmlMasterElement(writer, MkvId::MkvSeekHead, payload_size) {
                return false;
            }

            for i in 0..kSeekEntryCount {
                if self.seek_entry_id_[i] != 0 {
                    if !util::WriteEbmlMasterElement(writer, MkvId::MkvSeek, entry_size[i]) {
                        return false;
                    }

                    if !util::WriteEbmlElementArgU64(
                        writer,
                        MkvId::MkvSeekID,
                        self.seek_entry_id_[i] as u64,
                    ) {
                        return false;
                    }

                    if !util::WriteEbmlElementArgU64(
                        writer,
                        MkvId::MkvSeekPosition,
                        self.seek_entry_pos_[i],
                    ) {
                        return false;
                    }
                }
            }

            let total_entry_size = kSeekEntryCount as u64 * self.MaxEntrySize();
            let total_size = util::EbmlMasterElementSize(MkvId::MkvSeekHead, total_entry_size)
                + total_entry_size;
            let size_left = total_size - (writer.get_position() - self.start_pos_);

            let bytes_written = util::WriteVoidElement(writer, size_left);
            if bytes_written == 0 {
                return false;
            }

            if writer.set_position(pos).is_err() {
                return false;
            }
        }

        true
    }
}
