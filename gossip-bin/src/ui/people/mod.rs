use super::{GossipUi, Page};
use eframe::egui;
use egui::{Context, Ui};

mod follow;
mod followed;
mod muted;
mod person;

pub(super) fn update(app: &mut GossipUi, ctx: &Context, _frame: &mut eframe::Frame, ui: &mut Ui) {
    if app.page == Page::PeopleList {
        followed::update(app, ctx, _frame, ui);
    } else if app.page == Page::PeopleFollow {
        follow::update(app, ctx, _frame, ui);
    } else if app.page == Page::PeopleMuted {
        muted::update(app, ctx, _frame, ui);
    } else if matches!(app.page, Page::Person(_)) {
        person::update(app, ctx, _frame, ui);
    }
}
