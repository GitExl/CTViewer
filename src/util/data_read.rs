use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

pub fn read_script_blob(data: &mut Cursor<Vec<u8>>) -> ([u8; 64], usize) {
    let data_len = data.read_u16::<LittleEndian>().unwrap() as usize - 2;
    if data_len > 64 {
        panic!("Blob data of {} bytes is larger than the supported 64 bytes.", data_len);
    }

    let mut blob = vec![0u8; data_len];
    data.read_exact(&mut blob).unwrap();

    let mut blob_out = [0u8; 64];
    for i in 0..data_len {
        blob_out[i] = blob[i];
    }
    (blob_out, data_len)
}

pub fn read_24_bit_address(data: &mut Cursor<Vec<u8>>) -> usize {
    data.read_u8().unwrap() as usize |
        (data.read_u8().unwrap() as usize) << 8 |
        (data.read_u8().unwrap() as usize) << 16
}

pub fn read_segmented_address(data: &mut Cursor<Vec<u8>>) -> usize {
    data.read_u8().unwrap() as usize |
        (data.read_u8().unwrap() as usize) << 8
}
