use crate::filesystem::backend::FileSystemBackendTrait;
use crate::GameMode;

pub struct FileSystem {
    pub backend: Box<dyn FileSystemBackendTrait>,
    pub mode: GameMode,
}

impl FileSystem {
    pub fn new(backend: Box<dyn FileSystemBackendTrait>, mode: GameMode) -> Self {
        FileSystem {
            backend,
            mode,
        }
    }
}
