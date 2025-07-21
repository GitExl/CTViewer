// Chrono Trigger Decompression Routine
// Reverse engineered by Michael Springer (evilpeer@hotmail.com)
pub fn lz_decompress(data: &Vec<u8>, data_out: &mut Vec<u8>, start: usize) -> usize {
    let mut carry_flag: bool;
    let compressed_size: usize = ((data[start + 1] as u16 * 256) | data[start] as u16) as usize;
    let mut byte_pos: usize = start + 2;
    let mut byte_after: usize = byte_pos + compressed_size;
    let mut bits_count: u8;
    let mut cur_byte: u8;
    let mut work_pos: usize = 0;
    let small_bit_width: bool = data[byte_after] & 0xC0 != 0;

    bits_count = 8;
    loop {
        if byte_pos == byte_after {
            cur_byte = data[byte_pos];
            cur_byte &= 0x3F;
            if cur_byte == 0 {
                return work_pos;
            }
            bits_count = cur_byte;
            byte_after = start + ((data[byte_pos + 2] as u16 * 256) | data[byte_pos + 1] as u16) as usize;
            byte_pos += 3;
        } else {
            cur_byte = data[byte_pos];
            if cur_byte == 0 {
                cur_byte = data[byte_pos + 1];
                data_out[work_pos] = cur_byte;
                work_pos += 1;
                cur_byte = data[byte_pos + 2];
                data_out[work_pos] = cur_byte;
                work_pos += 1;
                cur_byte = data[byte_pos + 3];
                data_out[work_pos] = cur_byte;
                work_pos += 1;
                cur_byte = data[byte_pos + 4];
                data_out[work_pos] = cur_byte;
                work_pos += 1;
                cur_byte = data[byte_pos + 5];
                data_out[work_pos] = cur_byte;
                work_pos += 1;
                cur_byte = data[byte_pos + 6];
                data_out[work_pos] = cur_byte;
                work_pos += 1;
                cur_byte = data[byte_pos + 7];
                data_out[work_pos] = cur_byte;
                work_pos += 1;
                cur_byte = data[byte_pos + 8];
                data_out[work_pos] = cur_byte;
                work_pos += 1;
                byte_pos += 9;
            } else {
                byte_pos += 1;
                if cur_byte & 0x01 == 1 {
                    carry_flag = true;
                } else {
                    carry_flag = false;
                }
                cur_byte >>= 1;
                let mut mem_0d: u8 = cur_byte;
                if carry_flag {
                    copy_bytes(data, data_out, &mut byte_pos, &mut work_pos, small_bit_width);
                } else {
                    cur_byte = data[byte_pos];
                    data_out[work_pos] = cur_byte;
                    work_pos += 1;
                    byte_pos += 1;
                }
                loop {
                    bits_count -= 1;
                    if bits_count == 0 {
                        bits_count = 8;
                        break;
                    } else {
                        if mem_0d & 0x01 == 1 {
                            carry_flag = true;
                        } else {
                            carry_flag = false;
                        }
                        mem_0d >>= 1;
                        if carry_flag {
                            copy_bytes(data, data_out, &mut byte_pos, &mut work_pos, small_bit_width);
                        } else {
                            cur_byte = data[byte_pos];
                            data_out[work_pos] = cur_byte;
                            work_pos += 1;
                            byte_pos += 1;
                        }
                    }
                }
            }
        }
    }
}

fn copy_bytes(data: &Vec<u8>, data_out: &mut Vec<u8>, byte_pos: &mut usize, work_pos: &mut usize, small_bit_width: bool) {
    let mut byte_copy_count: u16;
    let mut byte_copy_off: u16;

    byte_copy_count = data[*byte_pos + 1] as u16;
    if small_bit_width {
        byte_copy_count >>= 3;
    } else {
        byte_copy_count >>= 4;
    }
    byte_copy_count += 2;

    byte_copy_off = (data[*byte_pos + 1] as u16 * 256) | data[*byte_pos] as u16;
    if small_bit_width {
        byte_copy_off &= 0x07FF;
    } else {
        byte_copy_off &= 0x0FFF;
    }

    if (*work_pos as isize - byte_copy_off as isize) < 0 {
        panic!("Copy bytes invalid.");
    }

    for i in 0..(byte_copy_count as usize) + 1 {
        data_out[*work_pos + i] = data_out[*work_pos - byte_copy_off as usize + i];
    }
    *work_pos += byte_copy_count as usize + 1;
    *byte_pos += 2;
}
