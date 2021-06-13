use byteorder::ByteOrder;
use byteorder::LittleEndian;

pub fn to_u32_array(x: &[u8]) -> Vec<u32> {
    let mut out = vec![0; x.len() / 4];
    LittleEndian::read_u32_into(x, &mut out);
    out
}
