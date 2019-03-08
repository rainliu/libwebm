use super::util;
use super::writer::Writer;
use crate::MkvId;

#[derive(Debug, Clone)]
pub struct SimpleTag {
    tag_name_: String,
    tag_string_: String,
}

impl SimpleTag {
    pub fn new() -> SimpleTag {
        SimpleTag {
            tag_name_: String::new(),
            tag_string_: String::new(),
        }
    }

    pub fn set_tag_name(&mut self, tag_name: &str) {
        self.tag_name_ = String::from(tag_name);
    }

    pub fn set_tag_string(&mut self, tag_string: &str) {
        self.tag_string_ = String::from(tag_string);
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut payload_size = util::EbmlElementSizeArgStr(MkvId::MkvTagName, &self.tag_name_);

        payload_size += util::EbmlElementSizeArgStr(MkvId::MkvTagString, &self.tag_string_);

        payload_size
    }
    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let payload_size = self.PayloadSize();
        let simple_tag_size =
            util::EbmlMasterElementSize(MkvId::MkvSimpleTag, payload_size) + payload_size;

        let start = writer.get_position();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvSimpleTag, payload_size) {
            return false;
        }

        if !util::WriteEbmlElementArgStr(writer, MkvId::MkvTagName, &self.tag_name_) {
            return false;
        }

        if !util::WriteEbmlElementArgStr(writer, MkvId::MkvTagString, &self.tag_string_) {
            return false;
        }

        let stop = writer.get_position();

        if stop - start != simple_tag_size {
            return false;
        }

        true
    }
}

pub struct Tag {
    simple_tags_: Vec<SimpleTag>,
}

impl Tag {
    pub fn new() -> Tag {
        Tag {
            simple_tags_: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.simple_tags_.clear();
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut payload_size = 0;

        for st in &self.simple_tags_ {
            payload_size += st.PayloadSize();
            payload_size += util::EbmlMasterElementSize(MkvId::MkvSimpleTag, payload_size);
        }

        payload_size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let payload_size = self.PayloadSize();

        let tag_size = util::EbmlMasterElementSize(MkvId::MkvTag, payload_size) + payload_size;

        let start = writer.get_position();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvTag, payload_size) {
            return false;
        }

        for st in &self.simple_tags_ {
            if !st.Write(writer) {
                return false;
            }
        }

        let stop = writer.get_position();
        if stop - start != tag_size {
            return false;
        }

        true
    }
}

impl Clone for Tag {
    fn clone(&self) -> Tag {
        Tag {
            simple_tags_: self.simple_tags_.to_vec(),
        }
    }
}

pub struct Tags {
    tags_: Vec<Tag>,
}

impl Tags {
    pub fn new() -> Tags {
        Tags { tags_: Vec::new() }
    }

    pub fn add_tag(&mut self, tag: Tag) {
        self.tags_.push(tag);
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut payload_size = 0;

        for t in &self.tags_ {
            payload_size += t.PayloadSize();
            payload_size += util::EbmlMasterElementSize(MkvId::MkvTag, payload_size);
        }

        payload_size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let payload_size = self.PayloadSize();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvTags, payload_size) {
            return false;
        }

        let start = writer.get_position();

        for t in &self.tags_ {
            if !t.Write(writer) {
                return false;
            }
        }

        let stop = writer.get_position();
        if stop - start != payload_size {
            return false;
        }

        true
    }
}
