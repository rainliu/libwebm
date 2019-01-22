use super::content_enc_aes_settings::ContentEncAESSettings;
use super::util;
use super::writer::Writer;
use crate::MkvId;

///////////////////////////////////////////////////////////////
// ContentEncoding element
// Elements used to describe if the track data has been encrypted or
// compressed with zlib or header stripping.
// Currently only whole frames can be encrypted with AES. This dictates that
// ContentEncodingOrder will be 0, ContentEncodingScope will be 1,
// ContentEncodingType will be 1, and ContentEncAlgo will be 5.
struct ContentEncoding {
    // Track element names
    enc_algo_: u64,
    enc_key_id_: Vec<u8>,
    encoding_order_: u64,
    encoding_scope_: u64,
    encoding_type_: u64,

    // ContentEncAESSettings element.
    enc_aes_settings_: ContentEncAESSettings,
}

impl ContentEncoding {
    pub fn enc_algo(&self) -> u64 {
        self.enc_algo_
    }
    pub fn encoding_order(&self) -> u64 {
        self.encoding_order_
    }
    pub fn encoding_scope(&self) -> u64 {
        self.encoding_scope_
    }
    pub fn encoding_type(&self) -> u64 {
        self.encoding_type_
    }
    pub fn enc_aes_settings(&self) -> &ContentEncAESSettings {
        &self.enc_aes_settings_
    }

    pub fn new() -> ContentEncoding {
        ContentEncoding {
            enc_algo_: 5,
            enc_key_id_: Vec::new(),
            encoding_order_: 0,
            encoding_scope_: 1,
            encoding_type_: 1,
            enc_aes_settings_: ContentEncAESSettings::new(),
        }
    }

    pub fn SetEncryptionID(&mut self, id: &[u8]) -> bool {
        self.enc_key_id_ = id.to_vec();
        true
    }

    pub fn Size(&self) -> u64 {
        let encryption_size = self.EncryptionSize();
        let encoding_size = self.EncodingSize(0, encryption_size);
        util::EbmlMasterElementSize(MkvId::MkvContentEncoding, encoding_size) + encoding_size
    }

    fn EncodingSize(&self, compresion_size: u64, encryption_size: u64) -> u64 {
        // TODO(fgalligan): Add support for compression settings.
        if compresion_size != 0 {
            return 0;
        }

        let mut encoding_size = 0;

        if encryption_size > 0 {
            encoding_size +=
                util::EbmlMasterElementSize(MkvId::MkvContentEncryption, encryption_size)
                    + encryption_size;
        }
        encoding_size +=
            util::EbmlElementSizeArgU64(MkvId::MkvContentEncodingType, self.encoding_type_);
        encoding_size +=
            util::EbmlElementSizeArgU64(MkvId::MkvContentEncodingScope, self.encoding_scope_);
        encoding_size +=
            util::EbmlElementSizeArgU64(MkvId::MkvContentEncodingOrder, self.encoding_order_);

        encoding_size
    }

    fn EncryptionSize(&self) -> u64 {
        let aes_size = self.enc_aes_settings_.Size();

        let mut encryption_size =
            util::EbmlElementSizeArgSlice(MkvId::MkvContentEncKeyID, &self.enc_key_id_);
        encryption_size += util::EbmlElementSizeArgU64(MkvId::MkvContentEncAlgo, self.enc_algo_);

        encryption_size + aes_size
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let encryption_size = self.EncryptionSize();
        let encoding_size = self.EncodingSize(0, encryption_size);
        let size =
            util::EbmlMasterElementSize(MkvId::MkvContentEncoding, encoding_size) + encoding_size;

        let payload_position = writer.get_position();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvContentEncoding, encoding_size) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(
            writer,
            MkvId::MkvContentEncodingOrder,
            self.encoding_order_,
        ) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(
            writer,
            MkvId::MkvContentEncodingScope,
            self.encoding_scope_,
        ) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvContentEncodingType, self.encoding_type_)
        {
            return false;
        }

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvContentEncryption, encryption_size) {
            return false;
        }
        if !util::WriteEbmlElementArgU64(writer, MkvId::MkvContentEncAlgo, self.enc_algo_) {
            return false;
        }
        if !util::WriteEbmlElementArgSlice(writer, MkvId::MkvContentEncKeyID, &self.enc_key_id_) {
            return false;
        }

        if !self.enc_aes_settings_.Write(writer) {
            return false;
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != size {
            return false;
        }

        return true;
    }
}
