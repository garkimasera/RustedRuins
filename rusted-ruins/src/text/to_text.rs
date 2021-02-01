use crate::game::Command;
use crate::text::{self, misc_txt, obj_txt, ui_txt, ToText, ToTextId};
use common::gamedata::*;
use common::gobj;
use common::objholder::*;
use std::borrow::Cow;

impl<T: ToTextId> ToText for T {
    fn to_text(&self) -> Cow<str> {
        text::to_txt(self).into()
    }
}

impl ToText for Site {
    fn to_text(&self) -> Cow<str> {
        if let Some(ref name) = self.name {
            let name: &str = &*name;
            return name.into();
        }

        match self.content {
            SiteContent::AutoGenDungeon { dungeon_kind } => text::to_txt(&dungeon_kind).into(),
            SiteContent::Town { ref town } => text::obj_txt(town.id()).into(),
            SiteContent::Base { .. } => "base".into(),
            SiteContent::Temp { .. } => "temp".into(),
            SiteContent::Other => {
                warn!("Unnamed other kind site");
                "".into()
            }
        }
    }
}

impl ToText for Item {
    fn to_text(&self) -> Cow<str> {
        use crate::game::item::ItemEx;
        let mut text: String = obj_txt(gobj::idx_to_id(self.idx)).into();

        if let Some(n) = self.charge() {
            text.push_str(&format!(" ({} : {})", ui_txt("item-charges"), n));
        }

        if let Some(title) = self.title() {
            if let Some(title) = super::readable::readable_title_txt(title) {
                text.push_str(&format!(" <{}>", title));
            }
        }

        if let Some((material_name, _)) = self.material() {
            text.push_str(&format!(" ({})", super::prefix::material(material_name)))
        }

        let quality = self.quality.as_int();
        if quality > 0 {
            text.push_str(&format!(" +{}", quality));
        } else if quality < 0 {
            text.push_str(&format!(" -{}", -quality));
        }

        for attr in &self.attributes {
            match attr {
                ItemAttribute::SkillLearning(kind) => {
                    text.push_str(&format!(" <{}>", kind.to_text()));
                }
                _ => (),
            }
        }

        text.into()
    }
}

impl ToText for CharaTemplateIdx {
    fn to_text(&self) -> Cow<str> {
        obj_txt(gobj::idx_to_id(*self)).into()
    }
}

impl ToText for Chara {
    fn to_text(&self) -> Cow<str> {
        use crate::game::chara::CharaEx;
        if self.is_main_character() {
            return misc_txt("you").into();
        }
        if let Some(ref name) = self.name {
            name.into()
        } else {
            obj_txt(gobj::idx_to_id(self.template)).into()
        }
    }
}

impl ToText for Command {
    fn to_text(&self) -> Cow<str> {
        use Command::*;
        let id = match self {
            Move { .. } => "command-move",
            MoveTo { .. } => "command-move_to",
            Shoot { .. } => "command-shoot",
            UseTool { .. } => "command-use-tool",
            Enter => "command-enter",
            Cancel => "command-cancel",
            RotateWindowRight => "command-rotate_window_right",
            RotateWindowLeft => "command-rotate_window_left",
            ItemInfomation => "command-item_information",
            OpenCreationWin => "command-open_creation_win",
            OpenDebugCommandWin => "command-open_debug_command_win",
            OpenEquipWin => "command-open_equip_win",
            OpenExitWin => "command-open_exit_win",
            OpenGameInfoWin => "command-open_game_info_win",
            OpenHelpWin => "command-open_help_win",
            OpenStatusWin => "command-open_status_win",
            OpenItemMenu => "command-open_item_menu",
            PickUpItem => "command-pick_up_item",
            DropItem => "command-drop_item",
            DrinkItem => "command-drink_item",
            EatItem => "command-eat_item",
            ReleaseItem => "command-release_item",
            ActionShortcut(_) => "command-action_shortcut",
            ChangeEquip { .. } => "command-change_equip",
            TextInput { .. } => "command-text_input",
            TextDelete => "command-text_delete",
            MouseButtonDown { .. } => "command-mouse_button_down",
            MouseButtonUp { .. } => "command-mouse_button_up",
            MouseWheel { .. } => "command-mouse_wheel",
            MouseState { .. } => "command-mouse_state",
        };
        ui_txt(id).into()
    }
}

impl ToText for Quest {
    fn to_text(&self) -> Cow<str> {
        match self {
            Quest::SlayMonsters { idx, .. } => {
                let mut table = fluent::FluentArgs::new();
                table.add("monster", fluent::FluentValue::String(idx.to_text()));
                crate::text::misc_txt_with_args("quest-slay_monsters", Some(&table)).into()
            }
        }
    }
}

/// Implement ToText for primitive types
macro_rules! impl_to_text {
    ( $($t:ty),* ) => {
        $(
            impl ToText for $t {
                fn to_text(&self) -> Cow<str> {
                    self.to_string().into()
                }
            }
        )*
    }
}

impl_to_text!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64, String);

impl<'a> ToText for &'a str {
    fn to_text(&self) -> Cow<'static, str> {
        self.to_string().into()
    }
}

impl<'a> ToText for Cow<'a, str> {
    fn to_text(&self) -> Cow<'static, str> {
        self.to_string().into()
    }
}
