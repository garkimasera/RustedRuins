//! Process characters action

pub mod get_item;
pub mod harvest;
pub mod use_item;

use super::extrait::*;
use super::target::Target;
use super::{Game, InfoGetter};
use crate::game::damage::*;
use crate::game::effect::{do_effect, weapon_to_effect};
use common::gamedata::*;
use common::gobj;
use geom::*;
use rules::RULES;

pub fn try_move(game: &mut Game, chara_id: CharaId, dir: Direction) -> bool {
    if dir.as_vec() == (0, 0) {
        return true;
    } // Move to current tile always success
    let dest_tile = game.gd.get_current_map().chara_pos(chara_id).unwrap() + dir.as_vec();

    if !game
        .gd
        .get_current_map()
        .is_passable(game.gd.chara.get(chara_id), dest_tile)
    {
        return false;
    }

    let other_chara = game.gd.get_current_map().get_chara(dest_tile);
    if other_chara.is_none() {
        game.gd.get_current_map_mut().move_chara(chara_id, dir);
        if chara_id == CharaId::Player {
            game.anim_queue.push_player_move(dir);
        }
    } else {
        let relation = game.gd.chara_relation(chara_id, other_chara.unwrap());

        match relation {
            Relationship::ALLY | Relationship::FRIENDLY | Relationship::NEUTRAL => {
                let current_map = game.gd.get_current_map_mut();
                if current_map.tile[dest_tile].chara != Some(CharaId::Player) {
                    current_map.move_chara(chara_id, dir);
                }
                if chara_id == CharaId::Player {
                    game.anim_queue.push_player_move(dir);
                }
            }
            Relationship::HOSTILE => {
                melee_attack(game, chara_id, other_chara.unwrap());
            }
        }
    }
    true
}

/// Melee attack
pub fn melee_attack(game: &mut Game, cid: CharaId, target: CharaId) {
    use crate::game::chara::power::*;

    let attacker = game.gd.chara.get(cid);
    let (effect, skill_kind, eff) =
        if let Some(weapon) = attacker.equip.item(EquipSlotKind::MeleeWeapon, 0) {
            let skill_kind = get_skill_kind_from_weapon(&weapon);
            let eff = weapon.calc_eff();
            (weapon_to_effect(weapon), skill_kind, eff)
        } else {
            // Attack by bare hands
            let effect = Effect {
                kind: vec![EffectKind::Melee {
                    element: Element::Physical,
                }],
                target_mode: TargetMode::Enemy,
                power_adjust: vec![],
                range: 1,
                shape: ShapeKind::OneTile,
                size: 0,
                anim_kind: EffectAnimKind::Chara,
                anim_img: "!damage-blunt".into(),
                anim_img_shot: String::new(),
                sound: "punch".into(),
            };
            (effect, SkillKind::BareHands, 1)
        };
    let (power, hit_power) = calc_power(
        attacker,
        CharaPowerKind::MeleeAttack,
        Element::Physical,
        skill_kind,
    );
    let power = power * eff as f32;

    do_effect(game, &effect, Some(cid), target, power, hit_power);

    // Exp processing
    let target_level = game.gd.chara.get(target).level;
    let attacker = game.gd.chara.get_mut(cid);
    attacker.add_attack_exp(skill_kind, target_level);
}

/// Shoot target
pub fn shoot_target(game: &mut Game, cid: CharaId, target: CharaId) -> bool {
    use crate::game::chara::power::*;

    let attacker = game.gd.chara.get(cid);
    let (effect, skill_kind, eff) =
        if let Some(weapon) = attacker.equip.item(EquipSlotKind::RangedWeapon, 0) {
            let skill_kind = get_skill_kind_from_weapon(&weapon);
            let eff = weapon.calc_eff();
            (weapon_to_effect(weapon), skill_kind, eff)
        } else {
            return false;
        };
    let (power, hit_power) = calc_power(
        attacker,
        CharaPowerKind::RangedAttack,
        Element::Physical,
        skill_kind,
    );
    let power = power * eff as f32;
    do_effect(game, &effect, Some(cid), target, power, hit_power);

    // Exp processing
    let target_level = game.gd.chara.get(target).level;
    let attacker = game.gd.chara.get_mut(cid);
    attacker.add_attack_exp(skill_kind, target_level);

    true
}

fn get_skill_kind_from_weapon(item: &Item) -> SkillKind {
    let weapon_obj = gobj::get_obj(item.idx);
    match weapon_obj.kind {
        ItemKind::Weapon(kind) => kind.into(),
        _ => SkillKind::BareHands,
    }
}

/// Throw one item
pub fn throw_item(game: &mut Game, il: ItemLocation, cid: CharaId, target: Target) {
    let gd = &mut game.gd;
    let effect = crate::game::item::throw::item_to_throw_effect(gd, il, cid);
    let item = gd.remove_item_and_get(il, 1);
    let chara = gd.chara.get(cid);
    let power = if item.obj().throw_effect.is_some() {
        item.calc_eff() as f32
            * chara.attr.str as f32
            * chara.attr.dex as f32
            * (chara.skills.get(SkillKind::Throwing) as f32 + RULES.combat.skill_base)
    } else {
        item.w() as f32 * RULES.effect.throw_weight_to_eff_factor * chara.attr.str as f32
    };
    game_log!("throw-item"; chara=chara, item=item);
    super::effect::do_effect(game, &effect, Some(cid), target, power, 1.0);

    // Exp processing
    let target_level = match target {
        Target::Chara(cid) => game.gd.chara.get(cid).level,
        _ => 1,
    };
    let attacker = game.gd.chara.get_mut(cid);
    attacker.add_attack_exp(SkillKind::Throwing, target_level);
}

/// Drink one item
pub fn drink_item(game: &mut Game, il: ItemLocation, cid: CharaId) {
    let gd = &mut game.gd;
    let item = gd.remove_item_and_get(il, 1); // Decrease the number of item by 1

    let chara = gd.chara.get_mut(cid);
    game_log!("drink-item"; chara=chara, item=item);

    let power = item.calc_eff() as f32 * RULES.effect.item_drink_power_factor;
    apply_medical_effect(game, cid, &item.obj().medical_effect, power);
}

/// Eat one item
pub fn eat_item(game: &mut Game, il: ItemLocation, cid: CharaId) {
    let gd = &mut game.gd;
    let item = gd.remove_item_and_get(il, 1); // Decrease the number of item by 1
    let item_obj = item.obj();

    let chara = gd.chara.get_mut(cid);
    game_log!("eat-item"; chara=chara, item=item);
    let nutrition: f32 = item_obj.nutrition.into();
    if let Some(damage) = chara.add_sp(nutrition * RULES.chara.sp_nutrition_factor, cid) {
        do_damage(game, cid, damage, CharaDamageKind::Starve);
    }

    let power = item.calc_eff() as f32 * RULES.effect.item_eat_power_factor;
    apply_medical_effect(game, cid, &item.obj().medical_effect, power);
}

pub fn release_item(game: &mut Game, il: ItemLocation, cid: CharaId, target: Target) {
    let mut item = game.gd.remove_item_and_get(il, 1);
    let item_obj = item.obj();
    let item_eff = item.calc_eff() as f32;

    match item.charge() {
        Some(n) if n >= 1 => {
            let skill_level = game.gd.chara.get(cid).skills.get(SkillKind::MagicDevice) as f32;
            let power = (skill_level / 10.0 + 1.0) * item_eff * RULES.magic.magic_device_base_power;
            if let Some(effect) = item_obj.magical_effect.as_ref() {
                super::effect::do_effect(game, effect, Some(cid), target, power, 1.0);
            } else {
                return;
            }
            *item.charge_mut().unwrap() = n - 1;
        }
        _ => (),
    }

    game.gd.get_item_list_mut(il.0).append(item, 1);
}

fn apply_medical_effect(game: &mut Game, cid: CharaId, effect: &Option<Effect>, eff: f32) {
    if effect.is_none() {
        return;
    }
    let effect = effect.as_ref().unwrap();
    super::effect::do_effect(game, effect, None, cid, eff as f32, 1.0);
}
