use super::commonuse::*;
use super::group_window::*;
use super::item_menu::ItemMenu;
use super::widget::*;
use crate::config::UI_CFG;
use crate::draw::border::draw_window_border;
use crate::eventhandler::InputMode;
use crate::game::extrait::*;
use crate::game::item::filter::*;
use crate::game::{DialogOpenRequest, Game, InfoGetter};
use crate::text::ToText;
use common::gamedata::*;
use common::gobj;
use sdl2::rect::Rect;

pub type ActionCallback = dyn FnMut(&mut DoPlayerAction, ItemLocation) -> DialogResult;
pub enum ItemWindowMode {
    List,
    PickUp,
    Drop,
    Throw,
    Drink,
    Eat,
    Use,
    Release,
    Read,
    ShopSell,
    ShopBuy {
        cid: CharaId,
    },
    Select {
        ill: ItemListLocation,
        filter: ItemFilter,
        action: Box<ActionCallback>,
    },
}

impl ItemWindowMode {
    pub fn is_main_mode(&self) -> bool {
        use ItemWindowMode::*;
        matches!(
            self,
            List | Drop | Throw | Drink | Eat | Use | Release | Read
        )
    }
}

pub struct ItemWindow {
    rect: Rect,
    list: ListWidget<(IconIdx, TextCache, LabelWidget)>,
    mode: ItemWindowMode,
    item_locations: Vec<ItemLocation>,
    escape_click: bool,
    info_label0: LabelWidget,
    info_label1: LabelWidget,
    menu: Option<super::item_menu::ItemMenu>,
}

const ITEM_WINDOW_GROUP_SIZE: u32 = 8;

pub fn create_item_window_group(game: &Game, mode: Option<ItemWindowMode>) -> GroupWindow {
    let mem_info = vec![
        MemberInfo {
            idx: gobj::id_to_idx("!tab-icon-item-list"),
            text_id: "tab_text-item_list",
            creator: |game| Box::new(ItemWindow::new(ItemWindowMode::List, game)),
        },
        MemberInfo {
            idx: gobj::id_to_idx("!tab-icon-item-drop"),
            text_id: "tab_text-item_drop",
            creator: |game| Box::new(ItemWindow::new(ItemWindowMode::Drop, game)),
        },
        MemberInfo {
            idx: gobj::id_to_idx("!tab-icon-item-throw"),
            text_id: "tab_text-item_throw",
            creator: |game| Box::new(ItemWindow::new(ItemWindowMode::Throw, game)),
        },
        MemberInfo {
            idx: gobj::id_to_idx("!tab-icon-item-drink"),
            text_id: "tab_text-item_drink",
            creator: |game| Box::new(ItemWindow::new(ItemWindowMode::Drink, game)),
        },
        MemberInfo {
            idx: gobj::id_to_idx("!tab-icon-item-eat"),
            text_id: "tab_text-item_eat",
            creator: |game| Box::new(ItemWindow::new(ItemWindowMode::Eat, game)),
        },
        MemberInfo {
            idx: gobj::id_to_idx("!tab-icon-item-use"),
            text_id: "tab_text-item_use",
            creator: |game| Box::new(ItemWindow::new(ItemWindowMode::Use, game)),
        },
        MemberInfo {
            idx: gobj::id_to_idx("!tab-icon-item-release"),
            text_id: "tab_text-item_release",
            creator: |game| Box::new(ItemWindow::new(ItemWindowMode::Release, game)),
        },
        MemberInfo {
            idx: gobj::id_to_idx("!tab-icon-item-read"),
            text_id: "tab_text-item_read",
            creator: |game| Box::new(ItemWindow::new(ItemWindowMode::Read, game)),
        },
    ];
    let rect: Rect = UI_CFG.item_window.rect.into();
    let i = mode.map(|mode| match mode {
        ItemWindowMode::List => 0,
        ItemWindowMode::Drop => 1,
        ItemWindowMode::Throw => 2,
        ItemWindowMode::Drink => 3,
        ItemWindowMode::Eat => 4,
        ItemWindowMode::Use => 5,
        ItemWindowMode::Release => 6,
        ItemWindowMode::Read => 7,
        _ => unreachable!(),
    });

    GroupWindow::new(
        "item",
        ITEM_WINDOW_GROUP_SIZE,
        i,
        game,
        mem_info,
        (rect.x, rect.y),
    )
}

impl ItemWindow {
    pub fn new(mode: ItemWindowMode, game: &Game) -> ItemWindow {
        let rect = UI_CFG.item_window.rect.into();
        let n_row = UI_CFG.item_window.n_row;
        let list_h = UI_CFG.list_widget.h_row_default;

        let mut item_window = ItemWindow {
            rect,
            list: ListWidget::with_scroll_bar(
                (0i32, 0i32, rect.w as u32, n_row * list_h),
                UI_CFG.item_window.column_pos.clone(),
                n_row,
                true,
            ),
            mode,
            item_locations: Vec::new(),
            escape_click: false,
            info_label0: LabelWidget::new(UI_CFG.item_window.info_label_rect0, "", FontKind::M),
            info_label1: LabelWidget::new(UI_CFG.item_window.info_label_rect1, "", FontKind::M)
                .right(),
            menu: None,
        };
        item_window.update_by_mode(&game.gd);
        item_window
    }

    pub fn new_select(
        ill: ItemListLocation,
        filter: ItemFilter,
        action: Box<ActionCallback>,
        pa: &mut DoPlayerAction,
    ) -> ItemWindow {
        let mode = ItemWindowMode::Select {
            ill,
            filter,
            action,
        };
        ItemWindow::new(mode, pa.game())
    }

    pub fn new_select_and_equip(
        cid: CharaId,
        slot: (EquipSlotKind, u8),
        pa: &mut DoPlayerAction,
    ) -> ItemWindow {
        let equip_selected_item = move |pa: &mut DoPlayerAction, il: ItemLocation| {
            pa.change_equipment(cid, slot, il);
            DialogResult::Close
        };

        ItemWindow::new_select(
            ItemListLocation::Chara { cid },
            ItemFilter::new().equip_slot_kind(slot.0),
            Box::new(equip_selected_item),
            pa,
        )
    }

    fn update_by_mode(&mut self, gd: &GameData) {
        let ill_player = ItemListLocation::Chara {
            cid: CharaId::Player,
        };
        let ill_ground = ItemListLocation::OnMap {
            mid: gd.get_current_mapid(),
            pos: gd.player_pos(),
        };

        match &self.mode {
            ItemWindowMode::List => {
                let filtered_list = gd.get_filtered_item_list(ill_player, ItemFilter::all());
                self.update_list(filtered_list);
            }
            ItemWindowMode::PickUp => {
                let filtered_list = gd.get_filtered_item_list(ill_ground, ItemFilter::all());
                self.update_list(filtered_list);
            }
            ItemWindowMode::Drop => {
                let filtered_list = gd.get_filtered_item_list(ill_player, ItemFilter::all());
                self.update_list(filtered_list);
            }
            ItemWindowMode::Throw => {
                let player_str = gd.chara.get(CharaId::Player).attr.str;
                let filter = ItemFilter::new().throwable(Some(player_str));
                let filtered_list = gd.get_filtered_item_list(ill_player, filter);
                self.update_list(filtered_list);
            }
            ItemWindowMode::Drink => {
                let filtered_list = gd.get_merged_filtered_item_list(
                    ill_ground,
                    ill_player,
                    ItemFilter::new().drinkable(true),
                );
                self.update_list(filtered_list);
            }
            ItemWindowMode::Eat => {
                let filtered_list = gd.get_merged_filtered_item_list(
                    ill_ground,
                    ill_player,
                    ItemFilter::new().eatable(true),
                );
                self.update_list(filtered_list);
            }
            ItemWindowMode::Use => {
                let filtered_list = gd.get_merged_filtered_item_list(
                    ill_ground,
                    ill_player,
                    ItemFilter::new().usable(true),
                );
                self.update_list(filtered_list);
            }
            ItemWindowMode::Release => {
                let filtered_list = gd.get_merged_filtered_item_list(
                    ill_ground,
                    ill_player,
                    ItemFilter::new().kind_rough(ItemKindRough::MagicDevice),
                );
                self.update_list(filtered_list);
            }
            ItemWindowMode::Read => {
                let filtered_list = gd.get_merged_filtered_item_list(
                    ill_ground,
                    ill_player,
                    ItemFilter::new().readable(true),
                );
                self.update_list(filtered_list);
            }
            ItemWindowMode::ShopBuy { cid } => {
                let ill = ItemListLocation::Shop { cid: *cid };
                let filtered_list = gd.get_filtered_item_list(ill, ItemFilter::new());
                self.update_list(filtered_list);
            }
            ItemWindowMode::ShopSell => {
                let ill = ItemListLocation::Chara {
                    cid: CharaId::Player,
                };
                let filtered_list = gd.get_filtered_item_list(ill, ItemFilter::new());
                self.update_list(filtered_list);
            }
            ItemWindowMode::Select { ill, filter, .. } => {
                let filtered_list = gd.get_filtered_item_list(*ill, *filter);
                self.update_list(filtered_list);
            }
        }
        self.update_label(gd);
    }

    fn update_list(&mut self, list: FilteredItemList) {
        self.list.set_n_item(list.clone().count() as u32);

        let mode = &self.mode;

        self.item_locations.clear();
        for (il, _, _) in list.clone() {
            self.item_locations.push(il);
        }

        let window_width = self.rect.width();

        self.list.update_rows_by_func(move |i| {
            let (_, item, n_item) = list.clone().nth(i as usize).unwrap();

            let item_text = format!("{} x {}", item.to_text(), n_item);

            // Infomation displayed in the right column
            let additional_info = match mode {
                ItemWindowMode::ShopBuy { .. } => format!("{}G", item.price()),
                ItemWindowMode::ShopSell => format!("{}G", item.selling_price()),
                _ => format!("{:.2}kg", item.w() as f32 / 1000.0),
            };

            let t1 = TextCache::one(item_text, FontKind::M, UI_CFG.color.normal_font.into());
            let w = window_width
                - UI_CFG.item_window.column_pos.clone()[2] as u32
                - UI_CFG.vscroll_widget.width;
            let t2 = LabelWidget::new(
                Rect::new(0, 0, w, UI_CFG.list_widget.h_row_default),
                &additional_info,
                FontKind::M,
            )
            .right();

            (item.icon(), t1, t2)
        });
    }

    fn update_label(&mut self, gd: &GameData) {
        let chara = gd.chara.get(CharaId::Player);
        let (weight, capacity) = chara.item_weight();

        self.info_label0.set_text(&format!(
            "{:0.1}/{:0.1} kg",
            weight / 1000.0,
            capacity / 1000.0
        ));

        match self.mode {
            ItemWindowMode::ShopBuy { .. } | ItemWindowMode::ShopSell { .. } => {
                self.info_label1
                    .set_text(&format!("{} G", gd.player.money()));
            }
            _ => (),
        }
    }

    fn do_action_for_item(&mut self, pa: &mut DoPlayerAction, il: ItemLocation) -> DialogResult {
        match self.mode {
            ItemWindowMode::List => {
                pa.request_dialog_open(DialogOpenRequest::ItemInfo { il });
                DialogResult::Continue
            }
            ItemWindowMode::PickUp => {
                pa.pick_up_item(il, ItemMoveNum::All);
                if pa.gd().is_item_on_player_tile() {
                    self.update_by_mode(pa.gd());
                    DialogResult::Continue
                } else {
                    DialogResult::Close
                }
            }
            ItemWindowMode::Drop => {
                pa.drop_item(il, 1);
                self.update_by_mode(pa.gd());
                DialogResult::Continue
            }
            ItemWindowMode::Throw => {
                pa.throw_item(il);
                DialogResult::CloseAll
            }
            ItemWindowMode::Drink => {
                pa.drink_item(il);
                DialogResult::CloseAll
            }
            ItemWindowMode::Eat => {
                pa.eat_item(il);
                DialogResult::CloseAll
            }
            ItemWindowMode::Use => {
                pa.use_item(il);
                DialogResult::CloseAll
            }
            ItemWindowMode::Release => {
                pa.release_item(il);
                DialogResult::CloseAll
            }
            ItemWindowMode::Read => {
                if pa.read_item(il) {
                    DialogResult::Continue
                } else {
                    DialogResult::CloseAll
                }
            }
            ItemWindowMode::ShopBuy { .. } => {
                pa.buy_item(il);
                self.update_by_mode(pa.gd());
                DialogResult::Continue
            }
            ItemWindowMode::ShopSell => {
                pa.sell_item(il);
                self.update_by_mode(pa.gd());
                DialogResult::Continue
            }
            ItemWindowMode::Select { ref mut action, .. } => action(pa, il),
        }
    }
}

impl Window for ItemWindow {
    fn draw(&mut self, context: &mut Context, game: &Game, anim: Option<(&Animation, u32)>) {
        draw_window_border(context, self.rect);
        self.list.draw(context);
        self.info_label0.draw(context);
        self.info_label1.draw(context);
        if let Some(menu) = self.menu.as_mut() {
            menu.draw(context, game, anim);
        }
    }
}

impl DialogWindow for ItemWindow {
    fn process_command(&mut self, command: &Command, pa: &mut DoPlayerAction) -> DialogResult {
        if let Some(menu) = self.menu.as_mut() {
            match menu.process_command(command, pa) {
                DialogResult::Special(SpecialDialogResult::ItemListUpdate) => {
                    self.menu = None;
                    self.update_by_mode(pa.gd());
                    return DialogResult::Continue;
                }
                DialogResult::Close => {
                    self.menu = None;
                    return DialogResult::Continue;
                }
                DialogResult::CloseAll => {
                    self.menu = None;
                    return DialogResult::CloseAll;
                }
                _ => {
                    return DialogResult::Continue;
                }
            }
        }

        let cursor_pos = if let Command::MouseButtonUp { x, y, .. } = command {
            Some((*x, *y))
        } else {
            None
        };

        check_escape_click!(self, command, self.mode.is_main_mode());

        if command == &Command::ItemInfomation {
            let il = self.item_locations[self.list.get_current_choice() as usize];
            pa.request_dialog_open(DialogOpenRequest::ItemInfo { il })
        }

        let command = command.relative_to(self.rect);

        if let Some(response) = self.list.process_command(&command) {
            match response {
                ListWidgetResponse::Select(i) => {
                    // Any item is selected
                    let il = self.item_locations[i as usize];
                    return self.do_action_for_item(pa, il);
                }
                ListWidgetResponse::SelectForMenu(i) => {
                    // Item selected to open menu
                    let il = self.item_locations[i as usize];
                    self.menu = Some(ItemMenu::new(pa.gd(), &self.mode, il, cursor_pos));
                }
                ListWidgetResponse::Scrolled => {
                    self.update_by_mode(pa.gd());
                }
                _ => (),
            }
            return DialogResult::Continue;
        }

        match command {
            Command::Cancel => DialogResult::Close,
            _ => DialogResult::Continue,
        }
    }

    fn mode(&self) -> InputMode {
        InputMode::Dialog
    }

    fn draw_mode(&self) -> WindowDrawMode {
        WindowDrawMode::SkipUnderWindows
    }

    fn update(&mut self, gd: &GameData) {
        self.update_by_mode(gd);
    }

    fn tab_switched(&mut self) {
        self.menu = None;
    }
}
