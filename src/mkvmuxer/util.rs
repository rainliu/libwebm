use super::writer::Writer;
use rand::Rng;
use std::io;
use std::io::{Error, ErrorKind};

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

fn EbmlMasterElementSize(t: u64, _value: u64) -> u64 {
    // Size of EBML ID
    let mut ebml_size: i32 = GetUIntSize(t);
    // Datasize
    ebml_size += GetCodedUIntSize(t);
    ebml_size as u64
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

fn EbmlElementSizeArgU64(t: u64, value: u64) -> u64 {
    EbmlElementSizeArgsU64(t, value, 0)
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

fn SerializeInt(writer: &mut dyn Writer, value: i64, size: i32) -> io::Result<()> {
    if size < 1 || size > 8 {
        Err(Error::new(ErrorKind::Other, "size should be in [1,8]"))
    } else {
        let mut buffer = vec![0; size as usize];
        for i in 1..=size {
            let byte_count = size - i;
            let bit_count = byte_count * 8;

            let bb: i64 = value >> bit_count;
            buffer[i as usize - 1] = bb as u8;
        }

        writer.write(&buffer)
    }
}

#[repr(C)]
union U32 {
    u: u32,
    f: f32,
}

fn SerializeFloat(writer: &mut dyn Writer, f: f32) -> io::Result<()> {
    assert!(std::mem::size_of::<u32>() == std::mem::size_of::<f32>());
    // This union is merely used to avoid a reinterpret_cast from float& to
    // uint32& which will result in violation of strict aliasing.
    let value: U32 = U32 { f: f };

    let mut buffer = vec![0; 4];
    for i in 1..=4 {
        let byte_count = 4 - i;
        let bit_count = byte_count * 8;

        let bb = unsafe {value.u >> bit_count};
        buffer[i as usize - 1] = bb as u8;
    }

    writer.write(&buffer)
}
