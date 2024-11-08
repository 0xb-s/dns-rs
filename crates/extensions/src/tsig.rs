pub struct TSIG {
    pub algorithm: String,
    pub name: String,
    pub time_signed: u32,
    pub fudge: u16,
    pub mac: Vec<u8>,
    pub original_id: u16,
    pub error: u16,
    pub other_data: Vec<u8>,
}
