use super::writer::Writer;
use rand::Rng;
use std::io;
use std::io::{Error, ErrorKind};
use crate::common::MkvId;

const EBML_UNKNOWN_VALUE: u64 = 0x01FFFFFFFFFFFFFF;
const MAX_BLOCK_TIMECODE: i64 = 0x07FFF;

// Date elements are always 8 octets in size.
const DATE_ELEMENT_SIZE: i32 = 8;

fn GetCodedUIntSize(value: u64) -> i32 {
    if value < 0x000000000000007F {
        return 1;
    } else if value < 0x0000000000003FFF {
        return 2;
    } else if value < 0x00000000001FFFFF {
        return 3;
    } else if value < 0x000000000FFFFFFF {
        return 4;
    } else if value < 0x00000007FFFFFFFF {
        return 5;
    } else if value < 0x000003FFFFFFFFFF {
        return 6;
    } else if value < 0x0001FFFFFFFFFFFF {
        return 7;
    }
    8
}

fn GetUIntSize(value: u64) -> i32 {
    if value < 0x0000000000000100 {
        return 1;
    } else if value < 0x0000000000010000 {
        return 2;
    } else if value < 0x0000000001000000 {
        return 3;
    } else if value < 0x0000000100000000 {
        return 4;
    } else if value < 0x0000010000000000 {
        return 5;
    } else if value < 0x0001000000000000 {
        return 6;
    } else if value < 0x0100000000000000 {
        return 7;
    }
    8
}

fn GetIntSize(value: i64) -> i32 {
    // Doubling the requested value ensures positive values with their high bit
    // set are written with 0-padding to avoid flipping the signedness.
    let v: u64 = if value < 0 {
        (value as u64) ^ 0xFFFFFFFFFFFFFFFF
    } else {
        value as u64
    };
    GetUIntSize(2 * v)
}

fn GetVersion(major: &mut i32, minor: &mut i32, build: &mut i32, revision: &mut i32) {
    *major = 0;
    *minor = 2;
    *build = 1;
    *revision = 0;
}

fn MakeUID() -> u64 {
    let mut rng = rand::thread_rng();
    let uid: u64 = rng.gen();
    return uid;
}

fn SerializeInt(writer: &mut dyn Writer, value: u64, size: i32) -> io::Result<()> {
    if size < 1 || size > 8 {
        Err(Error::new(ErrorKind::Other, "size should be in [1,8]"))
    } else {
        let mut buffer = vec![0; size as usize];
        for i in 1..=size {
            let byte_count = size - i;
            let bit_count = byte_count * 8;
            let bb = value >> bit_count;
            buffer[i as usize - 1] = bb as u8;
        }

        writer.write(&buffer)
    }
}

fn SerializeFloat(writer: &mut dyn Writer, f: f32) -> io::Result<()> {
    assert!(std::mem::size_of::<u32>() == std::mem::size_of::<f32>());
    // This union is merely used to avoid a reinterpret_cast from float& to
    // uint32& which will result in violation of strict aliasing.

    #[repr(C)]
    union U32 {
        u: u32,
        f: f32,
    }

    let value: U32 = U32 { f: f };

    let mut buffer = vec![0; 4];
    for i in 1..=4 {
        let byte_count = 4 - i;
        let bit_count = byte_count * 8;
        let bb = unsafe { value.u >> bit_count };
        buffer[i as usize - 1] = bb as u8;
    }

    writer.write(&buffer)
}

fn WriteUInt(writer: &mut dyn Writer, value: u64) -> io::Result<()> {
    let size = GetCodedUIntSize(value);

    WriteUIntSize(writer, value, size)
}

fn WriteUIntSize(writer: &mut dyn Writer, value: u64, size: i32) -> io::Result<()> {
    if size < 0 || size > 8 {
        return Err(Error::new(ErrorKind::Other, "size should be in [0,8]"));
    }

    let mut value = value;
    let mut size = size;
    if size > 0 {
        let bit = (1 as u64) << (size * 7);

        if value > (bit - 2) {
            return Err(Error::new(ErrorKind::Other, "value should > bit-2"));
        }

        value |= bit;
    } else {
        size = 1;
        let mut bit = 0;

        loop {
            bit = (1 as u64) << (size * 7);
            let m = bit - 2;

            if value <= m {
                break;
            }

            size += 1;
        }

        if size > 8 {
            return Err(Error::new(ErrorKind::Other, "size cannot > 8"));
        }

        value |= bit;
    }

    SerializeInt(writer, value, size)
}

fn WriteID(writer: &mut dyn Writer, t: u64) -> io::Result<()> {
    writer.element_start_notify(t, writer.get_position());

    let size = GetUIntSize(t);

    SerializeInt(writer, t, size)
}

fn EbmlMasterElementSize(t: u64, _value: u64) -> u64 {
    // Size of EBML ID
    let mut ebml_size: i32 = GetUIntSize(t);
    // Datasize
    ebml_size += GetCodedUIntSize(t);
    ebml_size as u64
}

fn WriteEbmlMasterElement(writer: &mut dyn Writer, t: u64, size: u64) -> bool {
    if WriteID(writer, t).is_err() {
        return false;
    }
    if WriteUInt(writer, size).is_err() {
        return false;
    }
    true
}

fn EbmlDateElementSize(t: u64) -> u64 {
    // Size of EBML ID
    let mut ebml_size: u64 = GetUIntSize(t) as u64;
    // Datasize
    ebml_size += DATE_ELEMENT_SIZE as u64;
    // Size of Datasize
    ebml_size += 1;
    ebml_size
}

fn WriteEbmlDateElement(writer: &mut dyn Writer, t: u64, value: i64) -> bool {
    if WriteID(writer, t).is_err() {
        return false;
    }

    if WriteUInt(writer, DATE_ELEMENT_SIZE as u64).is_err() {
        return false;
    }

    if SerializeInt(writer, value as u64, DATE_ELEMENT_SIZE).is_err() {
        return false;
    }

    true
}

/*pub enum EbmlElementSizeArguments {
    ArgI64(i64),
    ArgU64(u64),
    ArgF32(f32),
    ArgsU64(u64, u64),
    ArgStr(Option<&str>),
    ArgSlice(Option<&[u8]>, u64),
}*/

fn EbmlElementSizeArgI64(t: u64, value: i64) -> u64 {
    // Size of EBML ID
    let mut ebml_size: u64 = GetUIntSize(t) as u64;
    // Datasize
    ebml_size += GetIntSize(value) as u64;
    // Size of Datasize
    ebml_size += 1;
    ebml_size
}

fn WriteEbmlElementArgI64(writer: &mut dyn Writer, t: u64, value: i64) -> bool {
    if WriteID(writer, t).is_err() {
        return false;
    }

    let size = GetIntSize(value);
    if WriteUInt(writer, size as u64).is_err() {
        return false;
    }

    if SerializeInt(writer, value as u64, size).is_err() {
        return false;
    }

    true
}

fn EbmlElementSizeArgU64(t: u64, value: u64) -> u64 {
    EbmlElementSizeArgsU64(t, value, 0)
}

fn WriteEbmlElementArgU64(writer: &mut dyn Writer, t: u64, value: u64) -> bool {
    WriteEbmlElementArgsU64(writer, t, value, 0)
}

fn EbmlElementSizeArgF32(t: u64, _value: f32) -> u64 {
    // Size of EBML ID
    let mut ebml_size: u64 = GetUIntSize(t) as u64;
    // Datasize
    ebml_size += std::mem::size_of::<f32>() as u64;
    // Size of Datasize
    ebml_size += 1;
    ebml_size
}

fn WriteEbmlElement(writer: &mut dyn Writer, t: u64, value: f32) -> bool {
    if WriteID(writer, t).is_err() {
        return false;
    }

    if WriteUInt(writer, 4).is_err() {
        return false;
    }

    if SerializeFloat(writer, value).is_err() {
        return false;
    }

    true
}

fn EbmlElementSizeArgsU64(t: u64, value: u64, fixed_size: u64) -> u64 {
    // Size of EBML ID
    let mut ebml_size: u64 = GetUIntSize(t) as u64;
    // Datasize
    ebml_size += if fixed_size > 0 {
        fixed_size
    } else {
        GetUIntSize(value) as u64
    };
    // Size of Datasize
    ebml_size += 1;
    ebml_size
}

fn WriteEbmlElementArgsU64(writer: &mut dyn Writer, t: u64, value: u64, fixed_size: u64) -> bool {
    if WriteID(writer, t).is_err() {
        return false;
    }

    let mut size: u64 = GetUIntSize(value) as u64;
    if fixed_size > 0 {
        if size > fixed_size {
            return false;
        }
        size = fixed_size;
    }
    if WriteUInt(writer, size).is_err() {
        return false;
    }

    if SerializeInt(writer, value, size as i32).is_err() {
        return false;
    }

    true
}

fn EbmlElementSizeArgStr(t: u64, value: Option<&str>) -> u64 {
    if let Some(value) = value {
        // Size of EBML ID
        let mut ebml_size: u64 = GetUIntSize(t) as u64;
        // Datasize
        ebml_size += value.len() as u64;
        // Size of Datasize
        ebml_size += GetCodedUIntSize(value.len() as u64) as u64;
        ebml_size
    } else {
        0
    }
}

fn WriteEbmlElementArgStr(writer: &mut dyn Writer, t: u64, value: &str) -> bool {
    if WriteID(writer, t).is_err() {
        return false;
    }

    let length = value.len() as u64;
    if WriteUInt(writer, length).is_err() {
        return false;
    }

    if writer.write(value.as_bytes()).is_err() {
        return false;
    }

    return true;
}

fn EbmlElementSizeArgSlice(t: u64, value: Option<&[u8]>, size: u64) -> u64 {
    if let Some(_value) = value {
        // Size of EBML ID
        let mut ebml_size: u64 = GetUIntSize(t) as u64;
        // Datasize
        ebml_size += size;
        // Size of Datasize
        ebml_size += GetCodedUIntSize(size) as u64;
        ebml_size
    } else {
        0
    }
}

fn WriteEbmlElementArgSlice(writer: &mut dyn Writer, t: u64, value: &[u8], size: u64) -> bool {
    if size < 1 || value.len() != size as usize {
        return false;
    }

    if WriteID(writer, t).is_err() {
        return false;
    }

    if WriteUInt(writer, size).is_err() {
        return false;
    }

    if writer.write(value).is_err() {
        return false;
    }

    true
}

fn WriteVoidElement(writer: &mut dyn Writer, size: u64) -> u64 {
    // Subtract one for the void ID and the coded size.
    let void_entry_size: u64 = size - 1 - GetCodedUIntSize(size - 1) as u64;
    let void_size: u64 =
        EbmlMasterElementSize(MkvId::MkvVoid as u64, void_entry_size) + void_entry_size;

    if void_size != size {
        return 0;
    }

    let payload_position = writer.get_position();

    if WriteID(writer, MkvId::MkvVoid as u64).is_err() {
        return 0;
    }

    if WriteUInt(writer, void_entry_size).is_err() {
        return 0;
    }

    let value = vec![0; void_entry_size as usize];
    if writer.write(&value).is_err() {
        return 0;
    }

    let stop_position = writer.get_position();
    if stop_position - payload_position != void_size {
        return 0;
    }

    return void_size;
}
