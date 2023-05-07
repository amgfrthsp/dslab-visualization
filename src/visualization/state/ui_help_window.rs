use egui::Context;
use macroquad::window::{screen_height, screen_width};

pub fn draw_ui_help_window(egui_ctx: &Context) {
    egui::Window::new("Help")
        .fixed_size((screen_width() * 0.3, screen_height() * 0.3))
        .show(egui_ctx, |ui| {
            ui.heading("Управление клавишами: ");
            ui.label("");
            ui.horizontal(|ui| {
                ui.strong("Down / Up");
                ui.label(" - замедлить / ускорить анимацию");
            });
            ui.horizontal(|ui| {
                ui.strong("Right");
                ui.label(" - перейти к следующему событию");
            });
            ui.horizontal(|ui| {
                ui.strong("- / +");
                ui.label(" - уменьшить / увеличить масштаб");
            });
            ui.horizontal(|ui| {
                ui.strong("Space");
                ui.label(" - остановить / продолжить анимацию");
            });
            ui.label("");
            ui.heading("Config window: ");
            ui.label("");
            ui.horizontal(|ui| {
                ui.set_max_width(ui.available_width() * 0.2);
                ui.strong("Next event at {}");
                ui.label(
                    "показывает, когда произойдет следующее событие - \
                нода упадет или восстановится, сообщение отправится или \
                дойдет до получателя, сработает таймер и другие. Ориентируясь \
                на глобальное время и эту информацию, с помощью клавиши Right можно \
                перейти к следующему событию и не ждать, пока анимация отработает сама.\n\n\
                Здесь можно включить/отключить визуализацию таймеров, настроить скорость \
                анимации и то, для каких нод будут показываться события (сообщения и таймеры).",
                );
                ui.set_max_width(f32::INFINITY);
            });
        });
}
