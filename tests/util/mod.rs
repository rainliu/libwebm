use libwebm::mkvmuxer::writer::MkvWriter;
use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read};
use libwebm::mkvmuxer::util;

// constants for muxer and parser tests
const kAppString: &'static str = "mkvmuxer_unit_tests";
const kOpusCodecId: &'static str = "A_OPUS";
const kVorbisCodecId: &'static str = "A_VORBIS";
const kAudioTrackNumber: i32 = 2;
const kBitDepth: i32 = 2;
const kChannels: i32 = 2;
const kDuration: f64 = 2.345;
const kFrameLength: i32 = 10;
const kHeight: i32 = 180;
const kInvalidTrackNumber: i32 = 100;
const kOpusCodecDelay: u64 = 6500000;
const kOpusPrivateDataSizeMinimum: usize = 19;
const kOpusSeekPreroll: u64 = 80000000;
const kMetadataCodecId: &'static str = "D_WEBVTT/METADATA";
const kMetadataTrackNumber: i32 = 3;
const kMetadataTrackType: i32 = 0x21;
const kSampleRate: i32 = 30;
const kTimeCodeScale: i32 = 1000;
const kTrackName: &'static str = "unit_test";
const kVP8CodecId: &'static str = "V_VP8";
const kVP9CodecId: &'static str = "V_VP9";
const kVideoFrameRate: f64 = 0.5;
const kVideoTrackNumber: i32 = 1;
const kWidth: i32 = 320;

fn GetTempFileName() -> String {
    let temp_dir = std::env::temp_dir().to_str().unwrap().to_string();
    temp_dir + "/libwebm_temp." + &util::MakeUID().to_string()
}

fn GetTestDataDir() -> String {
    let test_data_path = std::env::var("LIBWEBM_TEST_DATA_PATH");
    match test_data_path {
        Ok(path) => path,
        Err(err) => ".".to_string(),
    }
}

fn GetTestFilePath(name: &str) -> String {
    let libwebm_testdata_dir = GetTestDataDir();
    libwebm_testdata_dir + "/" + name
}

fn CompareFiles(file1: &str, file2: &str) -> io::Result<()> {
    let mut f1 = File::open(file1)?;
    let mut f2 = File::open(file2)?;

    let block_size = 4096;
    let mut buf1 = vec![0u8; block_size];
    let mut buf2 = vec![0u8; block_size];

    loop {
        match (f1.read(&mut buf1), f2.read(&mut buf2)) {
            (Ok(n1), Ok(n2)) => {
                if n1 != n2 {
                    return Err(Error::new(ErrorKind::Other, "f1 and f2 not same size"));
                } else if buf1 != buf2 {
                    return Err(Error::new(ErrorKind::Other, "f1 and f2 not same content"));
                } else if n1 < block_size {
                    return Ok(());
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "f1 or f2 eof but another not eof",
                ));
            }
        }
    }
}

struct MuxerTest {
    writer_: MkvWriter,
    filename_: String,
    //segment_: Segment,
    dummy_data_: Vec<u8>,//[u8; ],
}

impl MuxerTest{
    fn new() -> MuxerTest {
        let temp_file = GetTempFileName();
        let file = File::open(temp_file.clone()).unwrap();

        MuxerTest{
            writer_: MkvWriter::new(file),
            filename_: temp_file,
            dummy_data_: vec![0; kFrameLength as usize],
        }
    }

    fn get_filename(&self) ->&str {
        &self.filename_
    }
}