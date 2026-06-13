use crate::scene::actor::{SceneActorClass, SceneActorFlags, SceneActorTask, DrawMode};
use crate::Context;
use crate::facing::Facing;
use crate::gamestate::gamestate_scene::SceneState;
use crate::scene_script::decoder::ops_char_load::CharacterType;
use crate::scene_script::scene_script::{ActorScriptState, OpResult};
use crate::sprites::sprite_renderer::SpritePriority;

pub fn exec_load_character(ctx: &mut Context, scene_state: &mut SceneState, actor_index: usize, char_type: CharacterType, index: usize, is_static: bool, battle_index: usize) -> OpResult {
    let sprite_index = match char_type {
        CharacterType::PCAsNPC => index,
        CharacterType::NPC => index + 7,
        CharacterType::Enemy => index + 256,
        _ => panic!("Wrong character type, must still refactor the rest of this..."),
    };

    let actor = scene_state.actors.get_mut(actor_index).unwrap();

    // todo: set remaining actor classes
    if char_type == CharacterType::PCAsNPC || char_type == CharacterType::NPC {
        actor.class = SceneActorClass::NPC;
    }
    if char_type == CharacterType::Enemy {
        actor.class = SceneActorClass::Enemy;
    }

    // Defaults.
    actor.facing = Facing::Down;
    actor.sprite_priority_top = SpritePriority::BelowL2AboveL1;
    actor.sprite_priority_bottom = SpritePriority::BelowL2AboveL1;
    actor.draw_mode = DrawMode::Draw;
    actor.battle_index = battle_index;
    actor.flags |= SceneActorFlags::SOLID;
    actor.flags.remove(SceneActorFlags::PUSHABLE);
    if is_static {
        actor.flags |= SceneActorFlags::BATTLE_STATIC;
    }

    // Sprite and animation defaults.
    let sprite_info_key = ctx.assets.load_sprite_info(&ctx.fs, sprite_index);
    let sprite_info = ctx.assets.get_sprite_info(sprite_info_key);
    actor.sprite_info_key = Some(sprite_info_key);

    actor.palette_key = Some(sprite_info.palette_key);
    actor.palette_offset = 0;

    actor.anim_set_index = sprite_info.anim_set_index;
    actor.anim_index = 0;
    actor.anim_frame = 0;

    let palette_key = sprite_info.palette_key;
    let palette = ctx.assets.get_palette(palette_key);
    actor.local_palette = palette.clone();

    OpResult::COMPLETE
}

pub fn exec_load_character_player(ctx: &mut Context, scene_state: &mut SceneState, script_state: &mut ActorScriptState, actor_index: usize, character_index: usize, battle_index: usize, must_be_active: bool) -> OpResult {

    // We need to borrow the actors list mutably, so determine the enter position and facing based
    // off of either PC0 or the scene entrance here.
    let pc0_index = scene_state.player_actors.get(&0);
    let mut enter_pos= scene_state.enter_position;
    let mut enter_facing = scene_state.enter_facing;
    if character_index != 0 {
      if let Some(pc0_index) = pc0_index {
         let pc0 = scene_state.actors.get(*pc0_index).unwrap();
         enter_pos = pc0.pos;
         enter_facing = pc0.facing;
      }
    }

    let actor = scene_state.actors.get_mut(actor_index).unwrap();

    // Player characters that are not active are considered dead.
    let is_active = ctx.party.is_character_active(character_index);
    let is_recruited = ctx.party.is_character_recruited(character_index);
    if !is_recruited || (must_be_active && !is_active) {
        actor.flags.insert(SceneActorFlags::DEAD);
        return OpResult::COMPLETE;
    }

    // Sprite and animation defaults.
    let sprite_info_key = ctx.assets.load_sprite_info(&ctx.fs, character_index);
    let sprite_info = ctx.assets.get_sprite_info(sprite_info_key);
    actor.sprite_info_key = Some(sprite_info_key);

    actor.palette_key = Some(sprite_info.palette_key);
    actor.palette_offset = 0;

    actor.anim_set_index = sprite_info.anim_set_index;
    actor.anim_index = 0;
    actor.anim_frame = 0;
    actor.anim_delay = 0;

    let palette_key = sprite_info.palette_key;
    let palette = ctx.assets.get_palette(palette_key);
    actor.local_palette = palette.clone();

    // Actor defaults.
    actor.battle_index = battle_index;
    actor.player_index = Some(character_index);
    actor.draw_mode = DrawMode::Draw;
    actor.flags.remove(SceneActorFlags::MOVE_ONTO_OBJECT | SceneActorFlags::MOVE_ONTO_TILE | SceneActorFlags::PUSHABLE);
    actor.flags.insert(SceneActorFlags::SOLID);
    actor.task = SceneActorTask::None;

    // Player characters get faster script execution by default.
    script_state.delay = 1;

    // Inherit coordinates and facing from PC 0, unless this is PC 0 then use scene enter data.
    if character_index != 0 {
        actor.facing = enter_facing;
        actor.move_to(enter_pos, true, &scene_state.scene_map);
    } else {
        actor.facing = scene_state.enter_facing;
        actor.move_to(scene_state.enter_position, true, &scene_state.scene_map);
    }

    if !must_be_active {
        actor.class = SceneActorClass::PCOutOfParty;
    } else {
        // todo set actual pc index from party active order
        actor.class = match character_index {
            0 => SceneActorClass::PC1,
            1 => SceneActorClass::PC2,
            2 => SceneActorClass::PC3,
            _ => SceneActorClass::PC1
        };
        scene_state.player_actors.insert(character_index, actor_index);
    }

    OpResult::COMPLETE
}
