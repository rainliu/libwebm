use super::chapter::Chapter;
use super::util;
use super::writer::Writer;
use crate::MkvId;

struct Chapters {
    // Array for storage of chapter objects.
    chapters_: Vec<Chapter>,
}

impl Chapters {
    pub fn new() -> Chapters {
        Chapters {
            chapters_: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.chapters_.len()
    }

    pub fn AddChapter(&mut self, chapter: Chapter) {
        self.chapters_.push(chapter);
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut payload_size = 0;
        for chapter in &self.chapters_ {
            let chapter_payload_size = chapter.PayloadSize();
            payload_size +=
                util::EbmlMasterElementSize(MkvId::MkvChapterAtom, chapter_payload_size)
                    + chapter_payload_size;
        }
        payload_size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let payload_size = self.PayloadSize();
        let edition_size =
            util::EbmlMasterElementSize(MkvId::MkvEditionEntry, payload_size) + payload_size;

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvChapters, payload_size) {
            return false;
        }

        let start = writer.get_position();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvEditionEntry, payload_size) {
            return false;
        }

        for chapter in &self.chapters_ {
            if !chapter.Write(writer) {
                return false;
            }
        }

        let stop = writer.get_position();
        if stop - start != edition_size {
            return false;
        }

        true
    }
}
