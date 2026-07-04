use std::collections::{HashMap, HashSet};
use sdl3::keyboard::Keycode;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum InputAction {
    Exit,

    // Pause
    TogglePause,

    // Menus and screens
    OpenMap,
    OpenSettingsMenu,
    OpenPartyMenu,

    // Menu
    MenuPrevious,
    MenuNext,
    MenuUp,
    MenuDown,
    MenuLeft,
    MenuRight,

    // World interaction
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Activate,
    Run,

    // Dialogue
    DialogueChoicePrevious,
    DialogueChoiceNext,
    DialogueChoiceConfirm,

    // Debug mode
    ToggleDebug,

    DebugCameraUp,
    DebugCameraDown,
    DebugCameraLeft,
    DebugCameraRight,

    DebugToggleLayer1,
    DebugToggleLayer2,
    DebugToggleLayer3,
    DebugToggleSprites,
    DebugTogglePalette,

    DebugOverlaysDisable,
    DebugOverlays1,
    DebugOverlays2,
    DebugOverlays3,
    DebugOverlays4,
    DebugOverlays5,
    DebugOverlays6,
    DebugOverlays7,
    DebugOverlays8,
    DebugOverlays9,

    DebugActorStep,
    DebugDump,
}

pub struct InputManager {
    bindings: HashMap<InputAction, Keycode>,
    keycode_to_bindings: HashMap<Keycode, HashSet<InputAction>>,

    inputs_down: HashSet<InputAction>,
    inputs_pressed: HashSet<InputAction>,
    inputs_released: HashSet<InputAction>,
}

impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            bindings: HashMap::new(),
            keycode_to_bindings: HashMap::new(),

            inputs_down: HashSet::new(),
            inputs_pressed: HashSet::new(),
            inputs_released: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.inputs_pressed.clear();
        self.inputs_released.clear();
    }

    pub fn is_down(&self, action: InputAction) -> bool {
        self.inputs_down.contains(&action)
    }

    pub fn was_pressed(&self, action: InputAction) -> bool {
        self.inputs_pressed.contains(&action)
    }

    pub fn was_released(&self, action: InputAction) -> bool {
        self.inputs_released.contains(&action)
    }

    pub fn bind(&mut self, action: InputAction, keycode: Keycode) {
        self.bindings.insert(action, keycode);
        if let Some(actions) = self.keycode_to_bindings.get_mut(&keycode) {
            actions.insert(action);
        } else {
            self.keycode_to_bindings.insert(keycode, HashSet::from_iter(vec![action]));
        }
    }

    pub fn unbind(&mut self, action: InputAction, keycode: Keycode) {
        self.bindings.remove(&action);
        if let Some(actions) = self.keycode_to_bindings.get_mut(&keycode) {
            actions.remove(&action);
        }
    }

    pub fn key_down(&mut self, key: Keycode) {
        if let Some(actions) = self.keycode_to_bindings.get(&key) {
            for action in actions {
                self.inputs_down.insert(*action);
                self.inputs_pressed.insert(*action);
            }
        }
    }

    pub fn key_up(&mut self, key: Keycode) {
        if let Some(actions) = self.keycode_to_bindings.get(&key) {
            for action in actions {
                self.inputs_down.remove(&action);
                self.inputs_released.insert(*action);
            }
        }
    }
}
