use crate::game::extrait::*;
use crate::game::InfoGetter;
use common::gamedata::*;
use script::{set_game_methods, GameMethods};

pub fn init() {
    set_game_methods(GameMethods {
        has_item: |gd, id| gd.has_item_by_id(id),
        gen_dungeons: |gd| {
            let mid = gd.get_current_mapid();
            crate::game::region::gen_dungeon_max(gd, mid.rid());
        },
        receive_quest_rewards: crate::game::quest::receive_rewards,
        receive_item: |gd, id, n| {
            let item = crate::game::item::gen::gen_item_from_id(id, 1);
            let il = gd.get_item_list_mut(ItemListLocation::PLAYER);
            il.append(item.clone(), n);
            let player = gd.chara.get_mut(CharaId::Player);
            game_log_i!("player-receive-item"; chara=player, item=item, n=n);
            player.update();
        },
        receive_money: |gd, amount| {
            gd.player.add_money(amount.into());
            let player = gd.chara.get(CharaId::Player);
            game_log_i!("player-receive-money"; chara=player, amount=amount);
        },
        remove_item: |gd, id, n| {
            let il = gd.player_item_location(id.as_ref()).ok_or(())?;
            gd.remove_item(il, n);
            gd.chara.get_mut(CharaId::Player).update();
            Ok(())
        },
    });
}
