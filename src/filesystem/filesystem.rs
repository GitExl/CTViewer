use crate::filesystem::backend::FileSystemBackendTrait;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ParseMode {
    Snes,
    Pc,
}

pub struct FileSystem {
    pub backend: Box<dyn FileSystemBackendTrait>,
    pub parse_mode: ParseMode,
}

impl FileSystem {
    pub fn new(backend: Box<dyn FileSystemBackendTrait>, parse_mode: ParseMode) -> Self {
        FileSystem {
            backend,
            parse_mode,
        }
    }
}
