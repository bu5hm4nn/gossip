mod copy_button;
pub(crate) mod list_entry;
pub use copy_button::{CopyButton, COPY_SYMBOL_SIZE};

mod nav_item;
use egui_winit::egui::{
    self, vec2, FontSelection, Rect, Response, Sense, TextEdit, Ui, WidgetText,
};
pub use nav_item::NavItem;

mod relay_entry;
pub use relay_entry::{RelayEntry, RelayEntryView};

mod modal_popup;
pub use modal_popup::modal_popup;

use super::GossipUi;
pub const DROPDOWN_DISTANCE: f32 = 10.0;

// pub fn break_anywhere_label(ui: &mut Ui, text: impl Into<WidgetText>) {
//     let mut job = text.into().into_text_job(
//         ui.style(),
//         FontSelection::Default,
//         ui.layout().vertical_align(),
//     );
//     job.job.sections.first_mut().unwrap().format.color =
//         ui.visuals().widgets.noninteractive.fg_stroke.color;
//     job.job.wrap.break_anywhere = true;
//     ui.label(job.job);
// }

pub fn page_header<R>(
    ui: &mut Ui,
    title: impl Into<egui::RichText>,
    right_aligned_content: impl FnOnce(&mut Ui) -> R,
) {
    ui.vertical(|ui| {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add_space(2.0);
                ui.heading(title);
            });
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                right_aligned_content,
            );
        });
        ui.add_space(10.0);
    });
}

/// Create a label which truncates after max_width
pub fn truncated_label(ui: &mut Ui, text: impl Into<WidgetText>, max_width: f32) {
    let mut job = text.into().into_text_job(
        ui.style(),
        FontSelection::Default,
        ui.layout().vertical_align(),
    );
    job.job.sections.first_mut().unwrap().format.color =
        ui.visuals().widgets.noninteractive.fg_stroke.color;
    job.job.wrap.break_anywhere = true;
    job.job.wrap.max_width = max_width;
    job.job.wrap.max_rows = 1;
    let wgalley = ui.fonts(|fonts| job.into_galley(fonts));
    // the only way to force egui to respect all our above settings
    // is to pass in the galley directly
    ui.label(wgalley.galley);
}

pub fn break_anywhere_hyperlink_to(ui: &mut Ui, text: impl Into<WidgetText>, url: impl ToString) {
    let mut job = text.into().into_text_job(
        ui.style(),
        FontSelection::Default,
        ui.layout().vertical_align(),
    );
    job.job.wrap.break_anywhere = true;
    ui.hyperlink_to(job.job, url);
}

pub fn search_filter_field(ui: &mut Ui, field: &mut String, width: f32) -> Response {
    // search field
    let response = ui.add(
        TextEdit::singleline(field)
            .text_color(ui.visuals().widgets.inactive.fg_stroke.color)
            .desired_width(width),
    );
    let rect = Rect::from_min_size(
        response.rect.right_top() - vec2(response.rect.height(), 0.0),
        vec2(response.rect.height(), response.rect.height()),
    );

    // search clear button
    if ui
        .put(
            rect,
            NavItem::new("\u{2715}", field.is_empty())
                .color(ui.visuals().widgets.inactive.fg_stroke.color)
                .active_color(ui.visuals().widgets.active.fg_stroke.color)
                .hover_color(ui.visuals().hyperlink_color)
                .sense(Sense::click()),
        )
        .clicked()
    {
        field.clear();
    }

    response
}

pub(super) fn set_important_button_visuals(ui: &mut Ui, app: &GossipUi) {
    let visuals = ui.visuals_mut();
    visuals.widgets.inactive.weak_bg_fill = app.theme.accent_color();
    visuals.widgets.inactive.fg_stroke.width = 1.0;
    visuals.widgets.inactive.fg_stroke.color = app.theme.get_style().visuals.extreme_bg_color;
    visuals.widgets.hovered.weak_bg_fill = app.theme.navigation_text_color();
    visuals.widgets.hovered.fg_stroke.color = app.theme.accent_color();
    visuals.widgets.inactive.fg_stroke.color = app.theme.get_style().visuals.extreme_bg_color;
}

// /// UTF-8 safe truncate (String::truncate() can panic)
// #[inline]
// pub fn safe_truncate(s: &str, max_chars: usize) -> &str {
//     let v: Vec<&str> = s.split('\n').collect();
//     let s = v.first().unwrap_or(&s);
//     match s.char_indices().nth(max_chars) {
//         None => s,
//         Some((idx, _)) => &s[..idx],
//     }
// }

// #[test]
// fn safe_truncate_single_line() {
//     let input = "0123456789";
//     let output = safe_truncate(input, 5);
//     assert_eq!(&input[0..5], output);
// }

// #[test]
// fn safe_truncate_multi_line() {
//     let input = "1234567890\nabcdefg\nhijklmn";
//     let output = safe_truncate(input, 20);
//     assert_eq!(&input[0..10], output);
// }
