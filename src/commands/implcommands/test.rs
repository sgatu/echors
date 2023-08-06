pub struct TestCmd {}
impl TestCmd {
    pub fn execute() -> Result<Option<Vec<u8>>, String> {
        return Ok(Some("ALL OK".as_bytes().to_vec()));
    }
}
