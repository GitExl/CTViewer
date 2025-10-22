use crate::actor::{ActorClass, ActorFlags, ActorTask, DrawMode};
use crate::Context;
use crate::facing::Facing;
use crate::gamestate::gamestate_scene::SceneState;
use crate::scene_script::decoder::ops_char_load::CharacterType;
use crate::scene_script::scene_script::OpResult;
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
        actor.class = ActorClass::NPC;
    }
    if char_type == CharacterType::Enemy {
        actor.class = ActorClass::Enemy;
    }

    // Defaults.
    actor.facing = Facing::Down;
    actor.sprite_priority_top = SpritePriority::BelowL2AboveL1;
    actor.sprite_priority_bottom = SpritePriority::BelowL2AboveL1;
    actor.draw_mode = DrawMode::Draw;
    actor.battle_index = battle_index;
    actor.flags |= ActorFlags::SOLID;
    actor.flags.remove(ActorFlags::PUSHABLE);
    if is_static {
        actor.flags |= ActorFlags::BATTLE_STATIC;
    }

    // Sprite and animation defaults.
    let sprite_state = &mut ctx.sprites_states.get_state_mut(actor_index);
    let sprite_asset = ctx.sprite_assets.load(&ctx.fs, sprite_index);
    sprite_state.sprite_index = sprite_index;
    sprite_state.anim_set_index = sprite_asset.anim_set_index;
    sprite_state.palette_offset = 0;
    sprite_state.palette = sprite_asset.palette.clone();
    sprite_state.anim_index = 0;
    sprite_state.anim_frame = 0;

    OpResult::COMPLETE
}

pub fn exec_load_character_player(ctx: &mut Context, scene_state: &mut SceneState, actor_index: usize, character_index: usize, battle_index: usize, must_be_active: bool) -> OpResult {

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
        actor.flags.insert(ActorFlags::DEAD);
        return OpResult::COMPLETE;
    }

    // Sprite and animation defaults.
    let sprite_state = &mut ctx.sprites_states.get_state_mut(actor_index);
    let sprite_asset = ctx.sprite_assets.load(&ctx.fs, character_index);
    sprite_state.sprite_index = character_index;
    sprite_state.palette = sprite_asset.palette.clone();
    sprite_state.palette_index = sprite_asset.palette_index;
    sprite_state.palette_offset = 0;
    sprite_state.anim_set_index = sprite_asset.anim_set_index;
    sprite_state.anim_index = 0;
    sprite_state.anim_frame = 0;
    sprite_state.anim_delay = 0;

    // Actor defaults.
    actor.battle_index = battle_index;
    actor.player_index = Some(character_index);
    actor.draw_mode = DrawMode::Draw;
    actor.flags.remove(ActorFlags::MOVE_ONTO_OBJECT | ActorFlags::MOVE_ONTO_TILE | ActorFlags::PUSHABLE);
    actor.flags.insert(ActorFlags::SOLID);
    actor.task = ActorTask::None;

    // Script state defaults.
    let script_state = &mut scene_state.script_states.get_mut(actor_index).unwrap();
    script_state.delay |= 1;

    // Inherit coordinates and facing from PC 0, unless this is PC 0 then use scene enter data.
    if character_index != 0 {
        actor.facing = enter_facing;
        actor.move_to(enter_pos, true, &scene_state.scene_map);
    } else {
        actor.facing = scene_state.enter_facing;
        actor.move_to(scene_state.enter_position, true, &scene_state.scene_map);
    }

    if !must_be_active {
        scene_state.player_actors.insert(character_index, actor_index);
        actor.class = ActorClass::PCOutOfParty;
    } else {
        // todo set actual pc index from party active order
        actor.class = match character_index {
            0 => ActorClass::PC1,
            1 => ActorClass::PC2,
            2 => ActorClass::PC3,
            _ => ActorClass::PC1
        };
    }

    OpResult::COMPLETE
}
