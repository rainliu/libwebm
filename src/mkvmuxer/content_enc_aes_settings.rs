use super::util;
use super::writer::Writer;
use crate::MkvId;

const CTR: u64 = 1;

///////////////////////////////////////////////////////////////
// ContentEncAESSettings element
pub struct ContentEncAESSettings {
    // Sub elements
    cipher_mode_: u64,
}

impl ContentEncAESSettings {
    pub fn new() -> ContentEncAESSettings {
        ContentEncAESSettings { cipher_mode_: CTR }
    }

    pub fn cipher_mode(&self) -> u64 {
        self.cipher_mode_
    }

    pub fn PayloadSize(&self) -> u64 {
        util::EbmlElementSizeArgU64(MkvId::MkvAESSettingsCipherMode as u64, self.cipher_mode_)
    }

    pub fn Size(&self) -> u64 {
        let payload = self.PayloadSize();
        util::EbmlMasterElementSize(MkvId::MkvContentEncAESSettings as u64, payload) + payload
    }

    pub fn Write(&self, writer: &mut dyn Writer) -> bool {
        let payload = self.PayloadSize();

        if !util::WriteEbmlMasterElement(writer, MkvId::MkvContentEncAESSettings as u64, payload) {
            return false;
        }
        let payload_position = writer.get_position();

        if !util::WriteEbmlElementArgU64(
            writer,
            MkvId::MkvAESSettingsCipherMode as u64,
            self.cipher_mode_,
        ) {
            return false;
        }

        let stop_position = writer.get_position();
        if stop_position - payload_position != payload {
            return false;
        }

        return true;
    }
}
