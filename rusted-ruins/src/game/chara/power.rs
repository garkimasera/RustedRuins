use crate::game::extrait::*;
use crate::rules::RULES;
use common::gamedata::*;

pub enum CharaPowerKind {
    MeleeAttack,
    RangedAttack,
}

/// Calculate character's (power, hit_power)
pub fn calc_power(
    chara: &Chara,
    power_kind: CharaPowerKind,
    _element: Element,
    skill_kind: SkillKind,
) -> (f32, f32) {
    let skill_lv = chara.skill_level(skill_kind) as f32;
    let (attr, hit_attr) = match power_kind {
        CharaPowerKind::MeleeAttack => (chara.attr.str, chara.attr.dex),
        CharaPowerKind::RangedAttack => (chara.attr.dex, chara.attr.dex),
    };
    let (attr, hit_attr) = (attr as f32, hit_attr as f32);

    let skill_base = RULES.combat.skill_base;

    let power = attr * attr * (skill_lv + skill_base).powf(1.5);
    let hit_power = hit_attr * (skill_lv + skill_base);

    (power, hit_power)
}
