use super::content_encoding::ContentEncoding;
use super::util;
use super::writer::Writer;
use crate::MkvId;

struct Track {
    // Track element names.
    codec_id_: String,
    codec_private_: Vec<u8>,
    language_: String,
    max_block_additional_id_: u64,
    name_: String,
    number_: u64,
    track_type_: u64,
    uid_: u64,
    codec_delay_: u64,
    seek_pre_roll_: u64,
    default_duration_: u64,

    // ContentEncoding element list.
    content_encoding_entries_: Vec<ContentEncoding>,
}

impl Track {
    pub fn new() -> Track {
        Track {
            codec_id_: String::new(),
            codec_private_: Vec::new(),
            language_: String::new(),
            max_block_additional_id_: 0,
            name_: String::new(),
            number_: 0,
            track_type_: 0,
            uid_: util::MakeUID(),
            codec_delay_: 0,
            seek_pre_roll_: 0,
            default_duration_: 0,
            content_encoding_entries_: Vec::new(),
        }
    }

    pub fn set_codec_id(&mut self, codec_id: &str) {
        self.codec_id_ = codec_id.to_string();
    }
    pub fn codec_id(&self) -> &str {
        &self.codec_id_
    }
    pub fn set_codec_private(&mut self, codec_private: &[u8]) {
        self.codec_private_ = codec_private.to_vec();
    }
    pub fn codec_private(&self) -> &[u8] {
        &self.codec_private_
    }
    pub fn set_language(&mut self, language: &str) {
        self.language_ = language.to_string();
    }
    pub fn language(&self) -> &str {
        &self.language_
    }
    pub fn set_max_block_additional_id(&mut self, max_block_additional_id: u64) {
        self.max_block_additional_id_ = max_block_additional_id;
    }
    pub fn max_block_additional_id(&self) -> u64 {
        self.max_block_additional_id_
    }
    pub fn set_name(&mut self, name: &str) {
        self.name_ = name.to_string();
    }
    pub fn name(&self) -> &str {
        &self.name_
    }
    pub fn set_number(&mut self, number: u64) {
        self.number_ = number;
    }
    pub fn number(&self) -> u64 {
        self.number_
    }
    pub fn set_track_type(&mut self, track_type: u64) {
        self.track_type_ = track_type;
    }
    pub fn track_type(&self) -> u64 {
        self.track_type_
    }
    pub fn set_uid(&mut self, uid: u64) {
        self.uid_ = uid;
    }
    pub fn uid(&self) -> u64 {
        self.uid_
    }
    pub fn set_codec_delay(&mut self, codec_delay: u64) {
        self.codec_delay_ = codec_delay;
    }
    pub fn codec_delay(&self) -> u64 {
        self.codec_delay_
    }
    pub fn set_seek_pre_roll(&mut self, seek_pre_roll: u64) {
        self.seek_pre_roll_ = seek_pre_roll;
    }
    pub fn seek_pre_roll(&self) -> u64 {
        self.seek_pre_roll_
    }
    pub fn set_default_duration(&mut self, default_duration: u64) {
        self.default_duration_ = default_duration;
    }
    pub fn default_duration(&self) -> u64 {
        self.default_duration_
    }

    pub fn PayloadSize(&self) -> u64 {
        let mut size = util::EbmlElementSizeArgU64(MkvId::MkvTrackNumber, self.number_);
        size += util::EbmlElementSizeArgU64(MkvId::MkvTrackUID, self.uid_);
        size += util::EbmlElementSizeArgU64(MkvId::MkvTrackType, self.track_type_);
        if !self.codec_id_.is_empty() {
            size += util::EbmlElementSizeArgStr(MkvId::MkvCodecID, &self.codec_id_);
        }
        if !self.codec_private_.is_empty() {
            size += util::EbmlElementSizeArgSlice(MkvId::MkvCodecPrivate, &self.codec_private_);
        }
        if !self.language_.is_empty() {
            size += util::EbmlElementSizeArgStr(MkvId::MkvLanguage, &self.language_);
        }
        if !self.name_.is_empty() {
            size += util::EbmlElementSizeArgStr(MkvId::MkvName, &self.name_);
        }
        if self.max_block_additional_id_ > 0 {
            size += util::EbmlElementSizeArgU64(
                MkvId::MkvMaxBlockAdditionID,
                self.max_block_additional_id_,
            );
        }
        if self.codec_delay_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvCodecDelay, self.codec_delay_);
        }
        if self.seek_pre_roll_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvSeekPreRoll, self.seek_pre_roll_);
        }
        if self.default_duration_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvDefaultDuration, self.default_duration_);
        }

        if !self.content_encoding_entries_.is_empty() {
            let mut content_encodings_size = 0;
            for encoding in &self.content_encoding_entries_ {
                content_encodings_size += encoding.Size();
            }

            size += util::EbmlMasterElementSize(MkvId::MkvContentEncodings, content_encodings_size)
                + content_encodings_size;
        }

        size
    }

    pub fn Size(&self) -> u64 {
        let mut size = self.PayloadSize();
        size += util::EbmlMasterElementSize(MkvId::MkvTrackEntry, size);
        size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        // mandatory elements without a default value.
        if self.track_type_ == 0 || self.codec_id_.is_empty() {
            return false;
        }

        // AV1 tracks require a CodecPrivate. See
        // https://github.com/Matroska-Org/matroska-specification/blob/av1-mappin/codec/av1.md
        // TODO(tomfinegan): Update the above link to the AV1 Matroska mappings to
        // point to a stable version once it is finalized, or our own WebM mappings
        // page on webmproject.org should we decide to release them.
        //if (!strcmp(codec_id_, Tracks::kAv1CodecId) && !codec_private_)
        //return false;

        // |size| may be bigger than what is written out in this function because
        // derived classes may write out more data in the Track element.
        let payload_size = self.PayloadSize();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvTrackEntry, payload_size) {
            return false;
        }

        let mut size = util::EbmlElementSizeArgU64(MkvId::MkvTrackNumber, self.number_);
        size += util::EbmlElementSizeArgU64(MkvId::MkvTrackUID, self.uid_);
        size += util::EbmlElementSizeArgU64(MkvId::MkvTrackType, self.track_type_);
        if !self.codec_id_.is_empty() {
            size += util::EbmlElementSizeArgStr(MkvId::MkvCodecID, &self.codec_id_);
        }
        if !self.codec_private_.is_empty() {
            size += util::EbmlElementSizeArgSlice(MkvId::MkvCodecPrivate, &self.codec_private_);
        }
        if !self.language_.is_empty() {
            size += util::EbmlElementSizeArgStr(MkvId::MkvLanguage, &self.language_);
        }
        if !self.name_.is_empty() {
            size += util::EbmlElementSizeArgStr(MkvId::MkvName, &self.name_);
        }
        if self.max_block_additional_id_ > 0 {
            size += util::EbmlElementSizeArgU64(
                MkvId::MkvMaxBlockAdditionID,
                self.max_block_additional_id_,
            );
        }
        if self.codec_delay_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvCodecDelay, self.codec_delay_);
        }
        if self.seek_pre_roll_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvSeekPreRoll, self.seek_pre_roll_);
        }
        if self.default_duration_ > 0 {
            size += util::EbmlElementSizeArgU64(MkvId::MkvDefaultDuration, self.default_duration_);
        }

        let payload_position = writer.get_position();

        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvTrackNumber, self.number_) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvTrackUID, self.uid_) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvTrackType, self.track_type_) {
            return false;
        }
        if self.max_block_additional_id_ > 0 {
            if !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvMaxBlockAdditionID,
                self.max_block_additional_id_,
            ) {
                return false;
            }
        }
        if self.codec_delay_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvCodecDelay, self.codec_delay_) {
                return false;
            }
        }
        if self.seek_pre_roll_ > 0 {
            if !util::WriteEbmlElementArgU64(writer, MkvId::MkvSeekPreRoll, self.seek_pre_roll_) {
                return false;
            }
        }
        if self.default_duration_ > 0 {
            if !util::WriteEbmlElementArgU64(
                writer,
                MkvId::MkvDefaultDuration,
                self.default_duration_,
            ) {
                return false;
            }
        }
        if !self.codec_id_.is_empty() {
            if !util::WriteEbmlElementArgStr(writer, MkvId::MkvCodecID, &self.codec_id_) {
                return false;
            }
        }
        if !self.codec_private_.is_empty() {
            if !util::WriteEbmlElementArgSlice(writer, MkvId::MkvCodecPrivate, &self.codec_private_)
            {
                return false;
            }
        }
        if !self.language_.is_empty() {
            if !util::WriteEbmlElementArgStr(writer, MkvId::MkvLanguage, &self.language_) {
                return false;
            }
        }
        if !self.name_.is_empty() {
            if !util::WriteEbmlElementArgStr(writer, MkvId::MkvName, &self.name_) {
                return false;
            }
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != size {
            return false;
        }

        if !self.content_encoding_entries_.is_empty() {
            let mut content_encodings_size = 0;
            for encoding in &self.content_encoding_entries_ {
                content_encodings_size += encoding.Size();
            }

            if !util::WriteEbmlMasterElement(
                writer,
                MkvId::MkvContentEncodings,
                content_encodings_size,
            ) {
                return false;
            }

            for encoding in &self.content_encoding_entries_ {
                if !encoding.Write(writer) {
                    return false;
                }
            }
        }

        //stop_position = writer->Position();
        true
    }
}
