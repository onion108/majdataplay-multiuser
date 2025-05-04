use egui::{Context, Id, Ui, vec2};

pub fn vert_centered(id: &str, ui: &mut Ui, ctx: &Context, body: impl FnOnce(&mut Ui) -> ()) {
    let layout_dat = Id::new("general_layout_id");
    ui.vertical(|ui| {
        let id = Id::new(id);
        let last_size = ctx.data(|map| map.get_temp::<f32>(id));
        if let Some(last_size) = last_size {
            ui.advance_cursor_after_rect(egui::Rect::from_two_pos(
                ui.cursor().min,
                ui.cursor().min + vec2(0., (ui.available_height() - last_size) / 2.),
            ));
        }
        let begin = ui.cursor().min.y;
        body(ui);
        let end = ui.cursor().min.y;
        let size = end - begin;
        if let Some(last_size) = last_size {
            if last_size != size {
                ctx.data_mut(|map| {
                    map.insert_temp(id, size);
                });
            }
        } else {
            ctx.data_mut(|map| {
                map.insert_temp(id, size);
                map.insert_temp(layout_dat, true);
            });
        }
    });
}

pub fn hori_centered(id: &str, ui: &mut Ui, ctx: &Context, body: impl FnOnce(&mut Ui) -> ()) {
    let layout_dat = Id::new("general_layout_id");
    ui.horizontal(|ui| {
        let id = Id::new(id);
        let last_size = ctx.data(|map| map.get_temp::<f32>(id));
        if let Some(last_size) = last_size {
            ui.advance_cursor_after_rect(egui::Rect::from_two_pos(
                ui.cursor().min,
                ui.cursor().min + vec2((ui.available_width() - last_size) / 2., 0.),
            ));
        }
        let begin = ui.cursor().min.x;
        body(ui);
        let end = ui.cursor().min.x;
        let size = end - begin;
        if let Some(last_size) = last_size {
            if last_size != size {
                ctx.data_mut(|map| {
                    map.insert_temp(id, size);
                });
            }
        } else {
            ctx.data_mut(|map| {
                map.insert_temp(id, size);
                map.insert_temp(layout_dat, true);
            });
        }
    });
}

pub fn discard_layout_on_need(ctx: &Context) {
    let id = Id::new("general_layout_id");
    let disc = ctx.data(|map| map.get_temp::<bool>(id));
    if disc.unwrap_or(false) {
        ctx.request_discard("Recalculate layout");
    }
    ctx.data_mut(|map| map.insert_temp(id, false));
}
