use crate::ui::feed::NoteRenderData;
use crate::ui::widgets;
use crate::ui::GossipUi;
use crate::ui::HighlightType;
use eframe::egui;
use egui::text::LayoutJob;
use egui::widget_text::WidgetText;
use egui::{Color32, Context, Frame, Margin, RichText, Ui};
use egui_winit::egui::Vec2;
use egui_winit::egui::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Background {
    None,
    Input,
    Note,
    HighlightedNote,
}

#[derive(Default)]
pub struct ThemeTest {
    button_small: bool,
    button_wide: bool,
}

pub(in crate::ui) fn update(
    app: &mut GossipUi,
    _ctx: &Context,
    _frame: &mut eframe::Frame,
    ui: &mut Ui,
) {
    widgets::page_header(ui, "Theme Test", |_ui| {});

    app.vert_scroll_area()
        .id_source(ui.auto_id_with("theme_test"))
        .show(ui, |ui| {
            button_test(app, ui);

            ui.add_space(20.0);

            // On No Background
            Frame::none()
                .inner_margin(Margin::symmetric(20.0, 20.0))
                .show(ui, |ui| {
                    ui.heading("No Background");
                    inner(app, ui, Background::None);
                });

            // On Note Background
            let render_data = NoteRenderData {
                height: 200.0,
                is_new: false,
                is_main_event: false,
                has_repost: false,
                is_comment_mention: false,
                is_thread: false,
                is_first: true,
                is_last: true,
                thread_position: 0,
            };
            Frame::none()
                .inner_margin(app.theme.feed_frame_inner_margin(&render_data))
                .outer_margin(app.theme.feed_frame_outer_margin(&render_data))
                .rounding(app.theme.feed_frame_rounding(&render_data))
                .shadow(app.theme.feed_frame_shadow(&render_data))
                .fill(app.theme.feed_frame_fill(&render_data))
                .stroke(app.theme.feed_frame_stroke(&render_data))
                .show(ui, |ui| {
                    ui.heading("Note Background");
                    ui.label("(with note margins)");
                    inner(app, ui, Background::Note);
                });

            // On Highlighted Note Background
            let render_data = NoteRenderData {
                height: 200.0,
                is_new: true,
                is_main_event: false,
                has_repost: false,
                is_comment_mention: false,
                is_thread: false,
                is_first: true,
                is_last: true,
                thread_position: 0,
            };
            Frame::none()
                .inner_margin(app.theme.feed_frame_inner_margin(&render_data))
                .outer_margin(app.theme.feed_frame_outer_margin(&render_data))
                .rounding(app.theme.feed_frame_rounding(&render_data))
                .shadow(app.theme.feed_frame_shadow(&render_data))
                .fill(app.theme.feed_frame_fill(&render_data))
                .stroke(app.theme.feed_frame_stroke(&render_data))
                .show(ui, |ui| {
                    ui.heading("Unread Note Background");
                    ui.label("(with note margins)");
                    inner(app, ui, Background::HighlightedNote);
                });

            // On Input Background
            Frame::none()
                .fill(app.theme.get_style().visuals.extreme_bg_color)
                .inner_margin(Margin::symmetric(20.0, 20.0))
                .show(ui, |ui| {
                    ui.heading("Input Background");
                    inner(app, ui, Background::Input);
                });
        });
}

fn inner(app: &mut GossipUi, ui: &mut Ui, background: Background) {
    let theme = app.theme;
    let accent = RichText::new("accent").color(theme.accent_color());
    let accent_complementary = RichText::new("accent complimentary (indirectly used)")
        .color(theme.accent_complementary_color());

    line(ui, accent);
    line(ui, accent_complementary);

    if background == Background::Input {
        for (ht, txt) in [
            (HighlightType::Nothing, "nothing"),
            (HighlightType::PublicKey, "public key"),
            (HighlightType::Event, "event"),
            (HighlightType::Relay, "relay"),
            (HighlightType::Hyperlink, "hyperlink"),
        ] {
            let mut highlight_job = LayoutJob::default();
            highlight_job.append(
                &format!("highlight text format for {}", txt),
                0.0,
                theme.highlight_text_format(ht),
            );
            line(ui, WidgetText::LayoutJob(highlight_job));
        }
    }

    if background == Background::Note || background == Background::HighlightedNote {
        let warning_marker =
            RichText::new("warning marker").color(theme.warning_marker_text_color());
        line(ui, warning_marker);

        let notice_marker = RichText::new("notice marker").color(theme.notice_marker_text_color());
        line(ui, notice_marker);
    }

    if background != Background::Input {
        ui.horizontal(|ui| {
            ui.label(RichText::new("•").color(Color32::from_gray(128)));
            crate::ui::widgets::break_anywhere_hyperlink_to(
                ui,
                "https://hyperlink.example.com",
                "https://hyperlink.example.com",
            );
        });
    }
}

fn line(ui: &mut Ui, label: impl Into<WidgetText>) {
    let bullet = RichText::new("•").color(Color32::from_gray(128));
    ui.horizontal(|ui| {
        ui.label(bullet);
        ui.label(label);
    });
}

fn button_test(app: &mut GossipUi, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.heading("Button Test:");
        ui.add_space(30.0);
        if ui
            .add(egui::RadioButton::new(
                !app.theme_test.button_small && !app.theme_test.button_wide,
                "normal",
            ))
            .clicked()
        {
            app.theme_test.button_small = false;
            app.theme_test.button_wide = false;
        }
        if ui
            .add(egui::RadioButton::new(app.theme_test.button_small, "small"))
            .clicked()
        {
            app.theme_test.button_small ^= true;
            if app.theme_test.button_small {
                app.theme_test.button_wide = false;
            }
        }
        if ui
            .add(egui::RadioButton::new(app.theme_test.button_wide, "wide"))
            .clicked()
        {
            app.theme_test.button_wide ^= true;
            if app.theme_test.button_wide {
                app.theme_test.button_small = false;
            }
        }
    });
    ui.add_space(30.0);
    const TEXT: &str = "Continue";
    let theme = &app.theme;
    const CSIZE: Vec2 = Vec2 { x: 100.0, y: 20.0 };
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.add_sized(CSIZE, egui::Label::new("Default"));
            ui.add_space(20.0);
            widgets::Button::primary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_default(ui);
            ui.add_space(20.0);
            widgets::Button::secondary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_default(ui);
            ui.add_space(20.0);
            widgets::Button::bordered(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_default(ui);
        });
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_sized(CSIZE, egui::Label::new("Hovered"));
            ui.add_space(20.0);
            widgets::Button::primary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_hovered(ui);
            ui.add_space(20.0);
            widgets::Button::secondary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_hovered(ui);
            ui.add_space(20.0);
            widgets::Button::bordered(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_hovered(ui);
        });
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_sized(CSIZE, egui::Label::new("Active"));
            ui.add_space(20.0);
            widgets::Button::primary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_active(ui);
            ui.add_space(20.0);
            widgets::Button::secondary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_active(ui);
            ui.add_space(20.0);
            widgets::Button::bordered(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_active(ui);
        });
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_sized(CSIZE, egui::Label::new("Disabled"));
            ui.add_space(20.0);
            widgets::Button::primary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_disabled(ui);
            ui.add_space(20.0);
            widgets::Button::secondary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_disabled(ui);
            ui.add_space(20.0);
            widgets::Button::bordered(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_disabled(ui);
        });
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_sized(CSIZE, egui::Label::new("Focused"));
            ui.add_space(20.0);
            widgets::Button::primary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_focused(ui);
            ui.add_space(20.0);
            widgets::Button::secondary(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_focused(ui);
            ui.add_space(20.0);
            widgets::Button::bordered(theme, TEXT)
                .small(app.theme_test.button_small)
                .wide(app.theme_test.button_wide)
                .draw_focused(ui);
        });
        ui.add_space(30.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.add_sized(CSIZE, egui::Label::new("try it->"));
            });
            ui.add_space(20.0);
            ui.vertical(|ui| {
                let response = widgets::Button::primary(theme, TEXT)
                    .small(app.theme_test.button_small)
                    .wide(app.theme_test.button_wide)
                    .ui(ui);
                if ui.link("focus").clicked() {
                    response.request_focus();
                }
            });
            ui.add_space(20.0);
            ui.vertical(|ui| {
                let response = widgets::Button::secondary(theme, TEXT)
                    .small(app.theme_test.button_small)
                    .wide(app.theme_test.button_wide)
                    .ui(ui);
                if ui.link("focus").clicked() {
                    response.request_focus();
                }
            });
            ui.add_space(20.0);
            ui.vertical(|ui| {
                let response = widgets::Button::bordered(theme, TEXT)
                    .small(app.theme_test.button_small)
                    .wide(app.theme_test.button_wide)
                    .ui(ui);
                if ui.link("focus").clicked() {
                    response.request_focus();
                }
            });
        });
    });
}
