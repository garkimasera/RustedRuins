pub mod builder;
pub mod from_template;
pub mod search;
pub mod tile_info;
pub mod wall_damage;
pub mod wilderness;

use super::chara::gen::create_npc_chara;
use super::item::gen::gen_dungeon_item;
use super::Game;
use crate::text::ToText;
use common::basic::MAX_ITEM_FOR_DRAW;
use common::gamedata::*;
use common::gobj;
use common::obj::TileKind;
use geom::*;
use rules::RULES;

pub trait MapEx {
    /// The tile is passable for given character or not.
    fn is_passable(&self, chara: &Chara, pos: Vec2d) -> bool;
    fn move_chara(&mut self, cid: CharaId, dir: Direction) -> bool;
}

impl MapEx for Map {
    fn is_passable(&self, _chara: &Chara, pos: Vec2d) -> bool {
        if !self.is_inside(pos) {
            return false;
        }

        if self.tile[pos].wall.is_empty() {
            let tile = gobj::get_obj(self.tile[pos].main_tile());
            match tile.kind {
                TileKind::Ground => true,
                TileKind::Water => false,
            }
        } else {
            false
        }
    }

    fn move_chara(&mut self, cid: CharaId, dir: Direction) -> bool {
        if let Some(p) = self.chara_pos(cid) {
            let new_p = p + dir.as_vec();
            self.swap_chara(p, new_p)
        } else {
            false
        }
    }
}

pub fn switch_map_with_pos(game: &mut Game, mid: MapId, pos: Option<Vec2d>) {
    game.ui_request.push_back(super::UiRequest::StopCentering);
    game.clear_target();

    let save_dir = game.save_dir.as_ref().unwrap();
    let gd = &mut game.gd;

    trace!("Switch map to {:?}", mid);
    // If next_mid floor doesn't exist, create new floor
    if !mid.is_region_map() && !gd.region.map_exist(mid) {
        info!("{:?} is not exist, so try to create new floor", mid);
        super::dungeon_gen::extend_site_floor(gd, mid.sid());
    }
    let prev_mid = gd.get_current_mapid();
    gd.region.preload_map(mid, save_dir.join("maps"));
    gd.set_current_mapid(mid);

    let new_player_pos = if let Some(pos) = pos {
        pos
    } else {
        if mid.is_region_map() && !prev_mid.is_region_map() && mid.rid() == prev_mid.rid() {
            // Exit from a site to region map
            gd.region.get_site_pos(prev_mid.sid())
        } else {
            // Move to another floor of the same site
            let current_map = gd.get_current_map();
            if let Some(p) = current_map.search_stairs(prev_mid.floor()) {
                p
            } else {
                current_map
                    .entrance
                    .get(0)
                    .map(|pos| *pos)
                    .unwrap_or(Vec2d(0, 0))
            }
        }
    };

    gd.get_current_map_mut()
        .locate_chara(CharaId::Player, new_player_pos);

    crate::audio::play_sound("floor-change");
    crate::audio::play_music(&gd.get_current_map().music);
    super::view::update_view_map(game);
}

/// Switch current map to the specified map
pub fn switch_map(game: &mut Game, mid: MapId) {
    switch_map_with_pos(game, mid, None);
}

pub fn gen_npcs(gd: &mut GameData, mid: MapId, n: u32, floor_level: u32) {
    let dungeon_kind = match gd.region.get_site(mid.sid()).content {
        SiteContent::AutoGenDungeon { dungeon_kind } => dungeon_kind,
        _ => DungeonKind::Ruin,
    };

    for _ in 0..n {
        if let Some(p) = choose_empty_tile(gd.region.get_map(mid)) {
            let chara = create_npc_chara(dungeon_kind, floor_level);
            trace!("Generate new npc {}", chara.to_text());
            let cid = gd.add_chara_to_map(chara, mid);
            let map = gd.region.get_map_mut(mid);
            map.locate_chara(cid, p);
        } else {
            warn!("Failed npc generating because empty tile not found");
            return;
        }
    }
}

/// Choose one empty tile in random
pub fn choose_empty_tile(map: &Map) -> Option<Vec2d> {
    use rng::gen_range;
    const MAX_TRY: usize = 10;

    // Function to determine the tile is empty or not
    let is_tile_empty = |tile: &TileInfo| {
        if tile.wall.is_empty() && tile.chara.is_none() && tile.special.is_none() {
            let tile_idx = tile.main_tile();
            let tile_obj = gobj::get_obj(tile_idx);
            tile_obj.kind == TileKind::Ground
        } else {
            false
        }
    };

    for _ in 0..MAX_TRY {
        let p = Vec2d(gen_range(0, map.w) as i32, gen_range(0, map.h) as i32);
        let tile = &map.tile[p];

        // Empty tile don't has wall, chara, and isn't special tile.
        if is_tile_empty(tile) {
            return Some(p);
        }
    }

    // If random tile choosing is failed many times, count empty tiles and choose
    let n_empty_tile = map.tile.iter().filter(|t| is_tile_empty(t)).count();
    if n_empty_tile == 0 {
        None
    } else {
        let r = gen_range(0, n_empty_tile);
        let p = map
            .tile
            .iter_with_idx()
            .filter(|&(_, t)| is_tile_empty(t))
            .nth(r)
            .unwrap()
            .0;
        Some(p)
    }
}

/// Locate some items for a new map
pub fn gen_items(gd: &mut GameData, mid: MapId) {
    use rng::*;
    let dungeon_kind = {
        let site = gd.region.get_site(mid.sid());
        match site.content {
            SiteContent::AutoGenDungeon { dungeon_kind } => dungeon_kind,
            _ => {
                return;
            } // No item generation
        }
    };
    let item_gen_probability = RULES.dungeon_gen[&dungeon_kind].item_gen_probability;
    let item_gen_probability = if 0.0 <= item_gen_probability && item_gen_probability <= 1.0 {
        item_gen_probability
    } else {
        warn!(
            "invalid value {} for item_gen_probablility",
            item_gen_probability
        );
        return;
    };
    let map = gd.region.get_map_mut(mid);

    for p in map.tile.iter_idx() {
        let tile = &mut map.tile[p];
        if !tile.wall.is_empty() {
            continue;
        }

        if get_rng().gen_bool(item_gen_probability) {
            if let Some(item) = gen_dungeon_item(mid.floor(), dungeon_kind) {
                map.locate_item(item, p, 1);
            }
        }
    }
}

pub fn update_observed_map(game: &mut Game) {
    let view_map = &game.view_map;
    let map = game.gd.get_current_map_mut();

    for p in map.tile.iter_idx() {
        if !view_map.get_tile_visible(p) {
            continue;
        }

        let tile = &map.tile[p];
        let observed_tile = &mut map.observed_tile[p];

        observed_tile.tile = true;
        observed_tile.wall = tile.wall;
        observed_tile.deco = tile.deco;
        observed_tile.special = tile.special;
        observed_tile.items.clear();

        for &(ref item, _) in tile.item_list.iter().take(MAX_ITEM_FOR_DRAW) {
            let mut variation = 0;
            for attr in &item.attributes {
                match attr {
                    ItemAttribute::ImageVariation(n) => {
                        variation = *n;
                    }
                    _ => (),
                }
            }
            observed_tile.items.push((item.idx, variation));
        }
    }
}
