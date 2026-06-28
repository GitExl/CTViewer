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
    inputs_down: HashSet<InputAction>,
    inputs_pressed: HashSet<InputAction>,
    inputs_released: HashSet<InputAction>,
}

impl InputManager {
    pub fn new() -> InputManager {
        let mut bindings = HashMap::new();

        // Default bindings.
        bindings.insert(InputAction::Exit, Keycode::Escape);

        bindings.insert(InputAction::TogglePause, Keycode::P);
        bindings.insert(InputAction::OpenMap, Keycode::Tab);
        bindings.insert(InputAction::OpenSettingsMenu, Keycode::R);
        bindings.insert(InputAction::OpenPartyMenu, Keycode::C);

        bindings.insert(InputAction::MenuPrevious, Keycode::Q);
        bindings.insert(InputAction::MenuNext, Keycode::E);
        bindings.insert(InputAction::MenuDown, Keycode::S);
        bindings.insert(InputAction::MenuLeft, Keycode::A);
        bindings.insert(InputAction::MenuRight, Keycode::D);

        bindings.insert(InputAction::MoveUp, Keycode::W);
        bindings.insert(InputAction::MoveDown, Keycode::S);
        bindings.insert(InputAction::MoveLeft, Keycode::A);
        bindings.insert(InputAction::MoveRight, Keycode::D);
        bindings.insert(InputAction::Activate, Keycode::F);
        bindings.insert(InputAction::Run, Keycode::LShift);

        bindings.insert(InputAction::DialogueChoicePrevious, Keycode::W);
        bindings.insert(InputAction::DialogueChoiceNext, Keycode::S);
        bindings.insert(InputAction::DialogueChoiceConfirm, Keycode::F);

        bindings.insert(InputAction::ToggleDebug, Keycode::Backspace);
        bindings.insert(InputAction::DebugCameraUp, Keycode::W);
        bindings.insert(InputAction::DebugCameraDown, Keycode::S);
        bindings.insert(InputAction::DebugCameraLeft, Keycode::A);
        bindings.insert(InputAction::DebugCameraRight, Keycode::D);
        bindings.insert(InputAction::DebugToggleLayer1, Keycode::_1);
        bindings.insert(InputAction::DebugToggleLayer2, Keycode::_2);
        bindings.insert(InputAction::DebugToggleLayer3, Keycode::_3);
        bindings.insert(InputAction::DebugToggleSprites, Keycode::_4);
        bindings.insert(InputAction::DebugTogglePalette, Keycode::_5);
        bindings.insert(InputAction::DebugOverlaysDisable, Keycode::Z);
        bindings.insert(InputAction::DebugOverlays1, Keycode::X);
        bindings.insert(InputAction::DebugOverlays2, Keycode::C);
        bindings.insert(InputAction::DebugOverlays3, Keycode::V);
        bindings.insert(InputAction::DebugOverlays4, Keycode::B);
        bindings.insert(InputAction::DebugOverlays5, Keycode::N);
        bindings.insert(InputAction::DebugOverlays6, Keycode::M);
        bindings.insert(InputAction::DebugOverlays7, Keycode::Comma);
        bindings.insert(InputAction::DebugOverlays8, Keycode::Period);
        bindings.insert(InputAction::DebugOverlays9, Keycode::Backslash);
        bindings.insert(InputAction::DebugDump, Keycode::Slash);
        bindings.insert(InputAction::DebugActorStep, Keycode::Space);

        InputManager {
            bindings,
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

    pub fn key_down(&mut self, key: Keycode) {
        for (action, action_keycode) in self.bindings.iter() {
            if *action_keycode == key {
                self.inputs_down.insert(*action);
                self.inputs_pressed.insert(*action);
            }
        }
    }

    pub fn key_up(&mut self, key: Keycode) {
        for (action, action_keycode) in self.bindings.iter() {
            if *action_keycode == key {
                self.inputs_down.remove(&action);
                self.inputs_released.insert(*action);
            }
        }
    }
}
