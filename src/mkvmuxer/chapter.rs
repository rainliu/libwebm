use super::util;
use super::writer::Writer;
use crate::MkvId;

#[derive(Debug, Clone)]
struct Display {
    title_: String,
    language_: String,
    country_: String,
}

impl Display {
    pub fn new() -> Display {
        Display {
            title_: String::new(),
            language_: String::new(),
            country_: String::new(),
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title_ = title.to_string();
    }

    pub fn title(&self) -> &str {
        &self.title_
    }

    pub fn set_language(&mut self, language: &str) {
        self.language_ = language.to_string();
    }

    pub fn language(&self) -> &str {
        &self.language_
    }

    pub fn set_country(&mut self, country: &str) {
        self.country_ = country.to_string();
    }

    pub fn country(&self) -> &str {
        &self.country_
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut payload_size = util::EbmlElementSizeArgStr(MkvId::MkvChapString, &self.title_);

        if !self.language_.is_empty() {
            payload_size += util::EbmlElementSizeArgStr(MkvId::MkvChapLanguage, &self.language_);
        }
        if !self.country_.is_empty() {
            payload_size += util::EbmlElementSizeArgStr(MkvId::MkvChapCountry, &self.country_);
        }

        payload_size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let payload_size = self.PayloadSize();
        let display_size =
            util::EbmlMasterElementSize(MkvId::MkvChapterDisplay, payload_size) + payload_size;

        let start = writer.get_position();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvChapterDisplay, payload_size) {
            return false;
        }

        if !util::WriteEbmlElementArgStr(writer, MkvId::MkvChapString, &self.title_) {
            return false;
        }

        if !self.language_.is_empty() {
            if !util::WriteEbmlElementArgStr(writer, MkvId::MkvChapLanguage, &self.language_) {
                return false;
            }
        }

        if !self.country_.is_empty() {
            if !util::WriteEbmlElementArgStr(writer, MkvId::MkvChapCountry, &self.country_) {
                return false;
            }
        }

        let stop = writer.get_position();

        if stop - start != display_size {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct Chapter {
    // The string identifier for this chapter (corresponds to WebVTT cue
    // identifier).
    id_: String,

    // Start timecode of the chapter.
    start_timecode_: u64,

    // Stop timecode of the chapter.
    end_timecode_: u64,

    // The binary identifier for this chapter.
    uid_: u64,

    // The Atom element can contain multiple Display sub-elements, as
    // the same logical title can be rendered in different languages.
    displays_: Vec<Display>,
}

impl Chapter {
    pub fn new() -> Chapter {
        Chapter {
            id_: String::new(),
            start_timecode_: 0,
            end_timecode_: 0,
            uid_: util::MakeUID(),
            displays_: Vec::new(),
        }
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut payload_size = util::EbmlElementSizeArgStr(MkvId::MkvChapterStringUID, &self.id_)
            + util::EbmlElementSizeArgU64(MkvId::MkvChapterUID, self.uid_)
            + util::EbmlElementSizeArgU64(MkvId::MkvChapterTimeStart, self.start_timecode_)
            + util::EbmlElementSizeArgU64(MkvId::MkvChapterTimeEnd, self.end_timecode_);

        for d in &self.displays_ {
            let display_payload_size = d.PayloadSize();
            payload_size +=
                util::EbmlMasterElementSize(MkvId::MkvChapterDisplay, display_payload_size)
                    + display_payload_size;
        }

        payload_size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let payload_size = self.PayloadSize();

        let atom_size =
            util::EbmlMasterElementSize(MkvId::MkvChapterAtom, payload_size) + payload_size;

        let start = writer.get_position();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvChapterAtom, payload_size) {
            return false;
        }

        if !util::WriteEbmlElementArgStr(writer, MkvId::MkvChapterStringUID, &self.id_) {
            return false;
        }

        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvChapterUID, self.uid_) {
            return false;
        }

        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvChapterTimeStart, self.start_timecode_) {
            return false;
        }

        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvChapterTimeEnd, self.end_timecode_) {
            return false;
        }

        for d in &self.displays_ {
            if !d.Write(writer) {
                return false;
            }
        }

        let stop = writer.get_position();
        if stop - start != atom_size {
            return false;
        }

        true
    }
}
