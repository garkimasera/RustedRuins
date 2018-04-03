
use super::Game;
use common::gamedata::chara::*;
use game::extrait::*;

/// This function will be called before the character's turn
/// 
pub fn preturn(game: &mut Game, cid: CharaId) -> bool {
    let chara = game.gd.chara.get_mut(cid);

    // Process character status
    for s in chara.status.iter_mut() {
        s.advance_turn(1);
    }
    chara.status.retain(|s| !s.is_expired()); // Remove expired status
    can_act(chara)
}

/// Judges this character can act or not
fn can_act(chara: &Chara) -> bool {
    for s in chara.status.iter() {
        match *s {
            CharaStatus::Asleep { .. } => {
                game_log_i!("asleep"; chara=chara.get_name());
                return false;
            }
            _ => (),
        }
    }
    true
}