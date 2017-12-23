
use std::rc::Rc;
use std::cell::RefCell;
use gtk;
use gtk::prelude::*;
use pixbuf_holder::PixbufHolder;
use edit_map::EditingMap;

#[derive(Clone)]
pub struct Ui {
    pub window: gtk::ApplicationWindow,
    pub map_drawing_area: gtk::DrawingArea,
    pub new_map_dialog: gtk::Dialog,
    pub adjustment_new_map_width: gtk::Adjustment,
    pub adjustment_new_map_height: gtk::Adjustment,
    pub adjustment_map_pos_x: gtk::Adjustment,
    pub adjustment_map_pos_y: gtk::Adjustment,
    pub pbh: Rc<PixbufHolder>,
    pub map: Rc<RefCell<EditingMap>>,
}

macro_rules! get_object {
    ($builder:expr, $id:expr) => {
        if let Some(object) = $builder.get_object($id) {
            object
        } else {
            panic!("Builder Error: \"{}\" is not found", $id)
        }
    }
}

pub fn build_ui(application: &gtk::Application) {
    // Get widgets from glade file
    let builder = gtk::Builder::new_from_string(include_str!("ui.glade"));

    let ui = Ui {
        window:           get_object!(builder, "window1"),
        map_drawing_area: get_object!(builder, "map-drawing-area"),
        new_map_dialog:   get_object!(builder, "new-map-dialog"),
        adjustment_new_map_width:  get_object!(builder, "adjustment-new-map-width"),
        adjustment_new_map_height: get_object!(builder, "adjustment-new-map-height"),
        adjustment_map_pos_x:      get_object!(builder, "adjustment-map-pos-x"),
        adjustment_map_pos_y:      get_object!(builder, "adjustment-map-pos-y"),
        pbh: Rc::new(PixbufHolder::new()),
        map: Rc::new(RefCell::new(EditingMap::new(16, 16))),
    };

    let menu_new:  gtk::MenuItem = get_object!(builder, "menu-new");
    let menu_quit: gtk::MenuItem = get_object!(builder, "menu-quit");

    ui.window.set_application(application);
    // Connect signals
    {
        let uic = ui.clone();
        ui.window.connect_delete_event(move |_, _| {
            uic.window.destroy();
            Inhibit(false)
        });
    }
    {
        let uic = ui.clone();
        ui.map_drawing_area.connect_draw(move |widget, context| {
            let width = widget.get_allocated_width();
            let height = widget.get_allocated_height();
            let map = uic.map.borrow();
            let pos_x = uic.adjustment_map_pos_x.get_value() as i32;
            let pos_y = uic.adjustment_map_pos_y.get_value() as i32;
            ::draw_map::draw_map(context, &*map, &*uic.pbh, width, height, pos_x, pos_y);
            Inhibit(false)
        });
    }
    {
        let uic = ui.clone();
        menu_new.connect_activate(move |_| {
            uic.new_map_dialog.show();
            let responce_id = uic.new_map_dialog.run();
            uic.new_map_dialog.hide();
            if responce_id == 1 {
                let width  = uic.adjustment_new_map_width.get_value() as u32;
                let height = uic.adjustment_new_map_height.get_value() as u32;
                uic.adjustment_map_pos_x.set_value(0.0);
                uic.adjustment_map_pos_y.set_value(0.0);
                uic.adjustment_map_pos_x.set_upper(width as f64);
                uic.adjustment_map_pos_y.set_upper(height as f64);
                let new_map = EditingMap::new(width, height);
                *uic.map.borrow_mut() = new_map;
                uic.map_drawing_area.queue_draw();
            }
        });
    }
    {
        let uic = ui.clone();
        menu_quit.connect_activate(move |_| {
            uic.window.destroy();
        });
    }
    {
        let uic = ui.clone();
        menu_quit.connect_activate(move |_| {
            uic.window.destroy();
        });
    }
    {
        let uic = ui.clone();
        ui.adjustment_map_pos_x.connect_value_changed(move |_| {
            uic.map_drawing_area.queue_draw();
        });
    }
        {
        let uic = ui.clone();
        ui.adjustment_map_pos_y.connect_value_changed(move |_| {
            uic.map_drawing_area.queue_draw();
        });
    }

    ui.window.show_all();
}

