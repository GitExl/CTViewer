use crate::gamestate::gamestate_scene::SceneState;
use crate::scene::scene_map::SceneTileFlags;
use crate::scene_script::scene_script::OpResult;
use crate::scene_script::scene_script_decoder::CopyTilesFlags;

pub fn exec_tile_copy(scene_state: &mut SceneState, left: u32, top: u32, right: u32, bottom: u32, dest_x: u32, dest_y: u32, flags: CopyTilesFlags, _delayed: bool) -> OpResult {
    // todo the delay option is unclear. op E5 will delay the actual tile copy until some later
    //  point, possibly when there is time for it, or when the player is not moving?

    // Copy tile indexes.
    if flags.contains(CopyTilesFlags::LAYER1) || flags.contains(CopyTilesFlags::LAYER2) || flags.contains(CopyTilesFlags::LAYER3) {
        for (layer_index, layer) in scene_state.map.layers.iter_mut().enumerate() {
            if layer_index == 0 && !flags.contains(CopyTilesFlags::LAYER1) {
                continue;
            }
            if layer_index == 1 && !flags.contains(CopyTilesFlags::LAYER2) {
                continue;
            }
            if layer_index == 2 && !flags.contains(CopyTilesFlags::LAYER3) {
                continue;
            }

            // Convert tile coordinates to chip coordinates.
            let chip_left = left * 2;
            let chip_top = top * 2;
            let chip_bottom = bottom * 2;
            let chip_right = right * 2;
            let chip_dest_x = dest_x * 2;
            let chip_dest_y = dest_y * 2;

            for chip_y in 0..chip_bottom - chip_top {
                for chip_x in 0..chip_right - chip_left {
                    let src_chip_x = chip_x + chip_left;
                    let src_chip_y = chip_y + chip_top;
                    let src_chip_index = (src_chip_x + src_chip_y * layer.chip_width) as usize;
                    if src_chip_index >= layer.chips.len() {
                        continue;
                    }

                    let dest_chip_x = chip_x + chip_dest_x;
                    let dest_chip_y = chip_y + chip_dest_y;
                    let dest_chip_index = (dest_chip_x + dest_chip_y * layer.chip_width) as usize;
                    if dest_chip_index >= layer.chips.len() {
                        continue;
                    }

                    layer.chips[dest_chip_index] = layer.chips[src_chip_index];
                }
            }
        }
    }

    // Copy property bytes.
    // The tile indices are already changed during map loading so handling L1_TILE_ADD and
    // L2_TILE_ADD is not actually necessary.
    if flags.contains(CopyTilesFlags::PROPS1) || flags.contains(CopyTilesFlags::PROPS2) || flags.contains(CopyTilesFlags::PROPS3) {

        // Since we do not store properties as bytes, we have to do some extra work to copy the
        // specific flags and struct members.
        let mut flag_mask = SceneTileFlags::empty();
        if flags.contains(CopyTilesFlags::PROPS2) {
            flag_mask |= SceneTileFlags::L1_TILE_ADD | SceneTileFlags::L2_TILE_ADD | SceneTileFlags::RLE_COMPRESSED;
        }
        if flags.contains(CopyTilesFlags::PROPS2) {
            flag_mask |= SceneTileFlags::DOOR_TRIGGER | SceneTileFlags::UNKNOWN_1 | SceneTileFlags::NPC_COLLISION_BATTLE;
        }
        if flags.contains(CopyTilesFlags::PROPS2) {
            flag_mask |= SceneTileFlags::COLLISION_IGNORE_Z | SceneTileFlags::COLLISION_INVERTED | SceneTileFlags::UNKNOWN_2 | SceneTileFlags::Z_NEUTRAL | SceneTileFlags::NPC_COLLISION;
        }

        let scene_map = &mut scene_state.scene_map;
        for y in 0..bottom - top {
            for x in 0..right - left {
                let src_index = (x + y * scene_map.props.width) as usize;
                if src_index >= scene_map.props.props.len() {
                    continue;
                }

                let dest_index = (dest_x + x + (dest_y + y) * scene_map.props.width) as usize;
                if dest_index >= scene_map.props.props.len() {
                    continue;
                }

                // Copy only masked flags.
                scene_map.props.props[dest_index].flags.remove(flag_mask);
                let new_flags = flag_mask & scene_map.props.props[src_index].flags;
                scene_map.props.props[dest_index].flags.insert(new_flags);

                // Copy individual properties.
                if flags.contains(CopyTilesFlags::PROPS1) {
                    scene_map.props.props[dest_index].collision = scene_map.props.props[src_index].collision;
                }
                if flags.contains(CopyTilesFlags::PROPS2) {
                    scene_map.props.props[dest_index].sprite_priority_top = scene_map.props.props[src_index].sprite_priority_top;
                    scene_map.props.props[dest_index].move_speed = scene_map.props.props[src_index].move_speed;
                    scene_map.props.props[dest_index].move_direction = scene_map.props.props[src_index].move_direction;
                }
                if flags.contains(CopyTilesFlags::PROPS3) {
                    scene_map.props.props[dest_index].sprite_priority_bottom = scene_map.props.props[src_index].sprite_priority_bottom;
                    scene_map.props.props[dest_index].z_plane = scene_map.props.props[src_index].z_plane;
                }
            }
        }
    }

    OpResult::COMPLETE | OpResult::YIELD
}
