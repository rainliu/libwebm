const DOC_TYPE_WEBM: &'static str = "webm";
const DOC_TYPE_MATROSKA: &'static str = "matroska";

trait Writer {
    fn write(buf: &[u8], len: u32) -> i32;
    fn position() -> i64;
    fn seekable() -> bool;
    fn element_start_notify(element_id: u64, position: i64);

    fn write_ebml_header(&self, doc_type_version: u64, doc_type: &str) -> bool {
        false
    }
}
