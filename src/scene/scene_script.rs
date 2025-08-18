use std::io::Cursor;

pub struct SceneActorScript {
    ptrs: [usize; 16],
}

impl SceneActorScript {
    pub fn new(ptrs: [usize; 16]) -> SceneActorScript {
        SceneActorScript {
            ptrs,
        }
    }
}

pub struct SceneScript {
    index: usize,
    data: Cursor<Vec<u8>>,
    actors: Vec<SceneActorScript>,
}

impl SceneScript {
    pub fn new(index: usize, data: Vec<u8>, actors: Vec<SceneActorScript>) -> SceneScript {
        SceneScript {
            index,
            data: Cursor::new(data),
            actors,
        }
    }

    pub fn dump(&self) {
        println!("Scene script {}", self.index);
        for (index, _) in self.actors.iter().enumerate() {
            println!("  Actor script {}", index);
        }
        println!();
    }
}

pub struct SceneActorScriptState {
    pub ops_per_tick: u32,
    pub address: usize,
    pub stored_address: usize,
    pub priority_address: [usize; 8],
}

impl SceneActorScriptState {
    pub fn new() -> SceneActorScriptState {
        SceneActorScriptState {
            ops_per_tick: 0,
            address: 0,
            stored_address: 0,
            priority_address: [0; 8],
        }
    }
}
