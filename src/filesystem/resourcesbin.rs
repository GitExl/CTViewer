use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use flate2::bufread::GzDecoder;

const BASE_SEED: i32 = 0x19000000;
const VAL1: i32 = 0x41C64E6D;
const VAL2: i32 = 12345;

pub struct ResourceFile {
    pub path: String,
    pub offset: u64,
    pub size: u64,
}

struct DirectoryEntry {
    pub path_offset: u64,
    pub offset: u64,
    pub size: u64,
}

pub struct ResourcesBin {
    pub files: HashMap<String,ResourceFile>,
    bin_path: Box<Path>,
}

impl ResourcesBin {
    pub fn new(path: &Path) -> ResourcesBin {
        let file = File::open(&path).expect(&format!("Could not open resources file {}.", path.to_str().unwrap()));
        let mut reader = BufReader::new(file);

        // Decode header data.
        let mut header_data = vec![0u8; 16];
        reader.read_exact(&mut header_data).unwrap();
        let mut header = decode_as_cursor(&mut Cursor::new(header_data), 0, 16, 0);

        // Read header fields.
        let mut id_raw = vec![0u8; 4];
        header.read_exact(&mut id_raw).unwrap();
        let signature = core::str::from_utf8( &id_raw).unwrap();
        let file_length = header.read_u32::<LittleEndian>().unwrap() as u64;
        let directory_offset = header.read_u32::<LittleEndian>().unwrap() as u64;
        let directory_length = header.read_u32::<LittleEndian>().unwrap() as u64;

        // Validate header.
        if signature != "ARC1" {
            panic!("Not a valid resources file: unknown signature.")
        }
        let file_len = reader.get_ref().metadata().unwrap().len();
        if file_len != file_length {
            panic!("Not a valid resources file: file length mismatch.");
        }
        if directory_offset > file_len {
            panic!("Not a valid resources file: directory is beyond file end.");
        }
        if directory_offset + directory_length > file_len {
            panic!("Not a valid resources file: directory continues beyond file end.");
        }

        // Decode and decompress directory data. Use the directory offset as block seed.
        let mut directory_data = vec![0u8; directory_length as usize];
        reader.seek(SeekFrom::Start(directory_offset)).unwrap();
        reader.read_exact(&mut directory_data).unwrap();
        let mut directory = decode_gz(&mut Cursor::new(directory_data), 0, directory_length as usize, directory_offset as i32);

        // Read raw entry data.
        let mut entries: Vec<DirectoryEntry> = Vec::new();
        let file_count = directory.read_u32::<LittleEndian>().unwrap() as usize;
        for _ in 0..file_count {
            entries.push(DirectoryEntry {
                path_offset: directory.read_u32::<LittleEndian>().unwrap() as u64,
                offset: directory.read_u32::<LittleEndian>().unwrap() as u64,
                size: directory.read_u32::<LittleEndian>().unwrap() as u64,
            });
        }

        // Read file paths and add as files to a map.
        let mut files = HashMap::new();
        for entry in entries {
            directory.seek(SeekFrom::Start(entry.path_offset)).unwrap();
            let mut entry_path = Vec::new();
            directory.read_until(0, &mut entry_path).unwrap();
            let entry_path_str = core::str::from_utf8(&entry_path[0..entry_path.len() - 1]).unwrap().to_string();

            files.insert(entry_path_str.clone(), ResourceFile {
                path: entry_path_str,
                offset: entry.offset,
                size: entry.size,
            });
        }

        ResourcesBin {
            files,
            bin_path: path.into(),
        }
    }

    pub fn file_exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    pub fn file_get(&self, path: &str) -> Cursor<Vec<u8>> {
        let entry = self.files.get(path).expect(&format!("File not found: {}", path));

        // Reopen the resources file (because we don't want to make this mutable).
        let mut file = File::open(&self.bin_path).unwrap();

        // Read data from the entry offset.
        file.seek(SeekFrom::Start(entry.offset)).unwrap();
        let mut data = vec![0; entry.size as usize];
        file.read_exact(&mut data).unwrap();
        let mut data_cursor = Cursor::new(data);

        // Decode and decompress. Use the entry offset as the block seed.
        decode_gz(&mut data_cursor, 0, entry.size as usize, entry.offset as i32)
    }
}

fn decode_gz(data: &mut Cursor<Vec<u8>>, offset: u64, length: usize, block_seed: i32) -> Cursor<Vec<u8>> {
    let mut decoded_data = decode_as_cursor(data, offset, length, block_seed);
    let len = decoded_data.read_u32::<BigEndian>().unwrap() as usize;

    // Feed the decoded data into the GZIP decompressor.
    let mut gz_decoder = GzDecoder::new(decoded_data);

    // Read back just the earlier specified data length.
    let mut buf = vec![0u8; len];
    gz_decoder.read(&mut buf).unwrap();

    Cursor::new(buf)
}

fn decode_as_cursor(data: &mut Cursor<Vec<u8>>, offset: u64, length: usize, block_seed: i32) -> Cursor<Vec<u8>> {
    Cursor::new(decode(data, offset, length, block_seed))
}

fn decode(data: &mut Cursor<Vec<u8>>, offset: u64, length: usize, block_seed: i32) -> Vec<u8> {
    let mut output = vec![0; length];
    data.seek(SeekFrom::Start(offset)).unwrap();

    // "Decrypt" the data. Thanks to CTExplore.
    let mut num1 = BASE_SEED + block_seed;
    for i in 0..length {
        num1 = num1.overflowing_mul(VAL1).0.overflowing_add(VAL2).0;
        let num2 = data.read_u8().unwrap() as i32;
        output[i] = (num2 ^ num1 >> 24) as u8;
    }

    output
}
