mod content;
mod notedata;

pub use notedata::Notes;
use std::cell::RefCell;
use std::rc::Rc;

use notedata::{NoteData, RepostType};

use super::FeedNoteParams;
use crate::comms::ToOverlordMessage;
use crate::feed::FeedKind;
use crate::globals::{Globals, GLOBALS};
use crate::ui::widgets::CopyButton;
use crate::ui::{GossipUi, Page};
use crate::AVATAR_SIZE_F32;
pub const AVATAR_SIZE_REPOST_F32: f32 = 27.0; // points, not pixels
use eframe::egui::{self, Margin};
use egui::{
    Align, Context, Frame, Image, Label, Layout, RichText, Sense, Separator, Stroke, TextStyle, Ui,
    Vec2,
};
use nostr_types::{Event, EventDelegation, EventKind, EventPointer, IdHex};

pub struct NoteRenderData {
    /// Available height for post
    pub height: f32,
    /// Has this post been seen yet?
    pub is_new: bool,
    /// This message is the focus of the view (formerly called is_focused)
    pub is_main_event: bool,
    /// This message is a repost of another message
    pub has_repost: bool,
    /// Is this post being mentioned within a comment
    pub is_comment_mention: bool,
    /// This message is part of a thread
    pub is_thread: bool,
    /// Is this the first post in the display?
    pub is_first: bool,
    /// Is this the last post in the display
    pub is_last: bool,
    /// Position in the thread, focused message = 0
    pub thread_position: i32,
    /// User can post
    pub can_post: bool,
}

pub(super) fn render_note(
    app: &mut GossipUi,
    ctx: &Context,
    _frame: &mut eframe::Frame,
    ui: &mut Ui,
    feed_note_params: FeedNoteParams,
) {
    let FeedNoteParams {
        id,
        indent,
        as_reply_to,
        threaded,
        is_first,
        is_last,
    } = feed_note_params;

    if let Some(note_ref) = app.notes.try_update_and_get(&id) {
        // FIXME respect app.settings.show_long_form on reposts
        // FIXME drop the cached notes on recompute

        if let Ok(note_data) = note_ref.try_borrow() {
            // Completely eliminate muted posts if not viewing a thread
            if note_data.author.muted > 0 && !threaded {
                return;
            }

            let is_new = app.settings.highlight_unread_events
                && !GLOBALS.viewed_events.contains(&note_data.event.id);

            let is_main_event: bool = {
                let feed_kind = GLOBALS.feed.get_feed_kind();
                match feed_kind {
                    FeedKind::Thread { id, .. } => id == note_data.event.id,
                    _ => false,
                }
            };

            let height = if let Some(height) = app.height.get(&id) {
                height
            } else {
                &0.0
            };

            let render_data = NoteRenderData {
                height: *height,
                has_repost: note_data.repost.is_some(),
                is_comment_mention: false,
                is_new,
                is_thread: threaded,
                is_first,
                is_last,
                is_main_event,
                thread_position: indent as i32,
                can_post: GLOBALS.signer.is_ready(),
            };

            let top = ui.next_widget_position();

            ui.horizontal(|ui| {
                // Outer indents first
                app.settings.theme.feed_post_outer_indent(ui, &render_data);

                let inner_response = Frame::none()
                    .inner_margin(app.settings.theme.feed_frame_inner_margin(&render_data))
                    .outer_margin(app.settings.theme.feed_frame_outer_margin(&render_data))
                    .rounding(app.settings.theme.feed_frame_rounding(&render_data))
                    .shadow(app.settings.theme.feed_frame_shadow(&render_data))
                    .fill(app.settings.theme.feed_frame_fill(&render_data))
                    .stroke(app.settings.theme.feed_frame_stroke(&render_data))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            // Inner indents first
                            app.settings.theme.feed_post_inner_indent(ui, &render_data);

                            render_note_inner(
                                app,
                                ctx,
                                ui,
                                note_ref.clone(),
                                &render_data,
                                as_reply_to,
                                &None,
                            );
                        });
                    });

                // Mark post as viewed if hovered AND we are not scrolling
                if inner_response.response.hovered() && app.current_scroll_offset == 0.0 {
                    GLOBALS.viewed_events.insert(id);
                    GLOBALS.new_viewed_events.blocking_write().insert(id);
                }
            });

            // Store actual rendered height for future reference
            let bottom = ui.next_widget_position();
            app.height.insert(id, bottom.y - top.y);

            thin_separator(
                ui,
                app.settings.theme.feed_post_separator_stroke(&render_data),
            );

            if threaded && !as_reply_to {
                let replies = Globals::get_replies_sync(id);
                let iter = replies.iter();
                let first = replies.first();
                let last = replies.last();
                for reply_id in iter {
                    super::render_note_maybe_fake(
                        app,
                        ctx,
                        _frame,
                        ui,
                        FeedNoteParams {
                            id: *reply_id,
                            indent: indent + 1,
                            as_reply_to,
                            threaded,
                            is_first: Some(reply_id) == first,
                            is_last: Some(reply_id) == last,
                        },
                    );
                }
            }
        }
    }
}

// FIXME, create some way to limit the arguments here.
fn render_note_inner(
    app: &mut GossipUi,
    ctx: &Context,
    ui: &mut Ui,
    note_ref: Rc<RefCell<NoteData>>,
    render_data: &NoteRenderData,
    hide_footer: bool,
    parent_repost: &Option<RepostType>,
) {
    if let Ok(note) = note_ref.try_borrow() {
        let collapsed = app.collapsed.contains(&note.event.id);

        // Load avatar texture
        let avatar = if note.author.muted > 0 {
            app.placeholder_avatar.clone()
        } else {
            if let Some(avatar) = app.try_get_avatar(ctx, &note.author.pubkey) {
                avatar
            } else {
                app.placeholder_avatar.clone()
            }
        };

        // Determine avatar size
        let avatar_size = if parent_repost.is_none() {
            match note.repost {
                None | Some(RepostType::CommentMention) => AVATAR_SIZE_F32,
                Some(_) => AVATAR_SIZE_REPOST_F32,
            }
        } else {
            match parent_repost {
                None | Some(RepostType::CommentMention) => AVATAR_SIZE_REPOST_F32,
                Some(_) => AVATAR_SIZE_F32,
            }
        };

        let inner_margin = app.settings.theme.feed_frame_inner_margin(render_data);

        let avatar_margin_left = if parent_repost.is_none() {
            match note.repost {
                None | Some(RepostType::CommentMention) => 0.0,
                Some(_) => (AVATAR_SIZE_F32 - AVATAR_SIZE_REPOST_F32) / 2.0,
            }
        } else {
            match parent_repost {
                None | Some(RepostType::CommentMention) => {
                    (AVATAR_SIZE_F32 - AVATAR_SIZE_REPOST_F32) / 2.0
                }
                Some(_) => 0.0,
            }
        };

        let hide_footer = if hide_footer || note.author.muted > 0 {
            true
        } else if parent_repost.is_none() {
            match note.repost {
                None | Some(RepostType::CommentMention) => false,
                Some(_) => true,
            }
        } else {
            match parent_repost {
                None | Some(RepostType::CommentMention) => true,
                Some(_) => false,
            }
        };

        let content_pull_top =
            inner_margin.top + ui.style().spacing.item_spacing.y * 4.0 - avatar_size;

        let content_margin_left = AVATAR_SIZE_F32 + inner_margin.left;
        let footer_margin_left = content_margin_left;

        ui.vertical(|ui| {
            // First row

            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                ui.add_space(avatar_margin_left);

                // render avatar
                if ui
                    .add(
                        Image::new(
                            &avatar,
                            Vec2 {
                                x: avatar_size,
                                y: avatar_size,
                            },
                        )
                        .sense(Sense::click()),
                    )
                    .clicked()
                {
                    app.set_page(Page::Person(note.author.pubkey.clone()));
                };

                ui.add_space(avatar_margin_left);

                ui.add_space(3.0);

                GossipUi::render_person_name_line(app, ui, &note.author);

                ui.horizontal_wrapped(|ui| {
                    if let Some((irt, _)) = note.event.replies_to() {
                        ui.add_space(8.0);

                        ui.style_mut().override_text_style = Some(TextStyle::Small);
                        let idhex: IdHex = irt.into();
                        let nam = format!("▲ #{}", GossipUi::hex_id_short(&idhex));
                        if ui.link(&nam).clicked() {
                            app.set_page(Page::Feed(FeedKind::Thread {
                                id: irt,
                                referenced_by: note.event.id,
                            }));
                        };
                        ui.reset_style();
                    }

                    ui.add_space(8.0);

                    if note.event.pow() > 0 {
                        ui.label(format!("POW={}", note.event.pow()));
                    }

                    match &note.delegation {
                        EventDelegation::InvalidDelegation(why) => {
                            let color = app.settings.theme.warning_marker_text_color();
                            ui.add(Label::new(RichText::new("INVALID DELEGATION").color(color)))
                                .on_hover_text(why);
                        }
                        EventDelegation::DelegatedBy(_) => {
                            let color = app.settings.theme.notice_marker_text_color();
                            ui.label(RichText::new("DELEGATED").color(color));
                        }
                        _ => {}
                    }

                    if note.deletion.is_some() {
                        let color = app.settings.theme.warning_marker_text_color();
                        ui.label(RichText::new("DELETED").color(color));
                    }

                    if note.event.kind == EventKind::Repost {
                        let color = app.settings.theme.notice_marker_text_color();
                        ui.label(RichText::new("REPOSTED").color(color));
                    }

                    if note.event.kind == EventKind::EncryptedDirectMessage {
                        let color = app.settings.theme.notice_marker_text_color();
                        ui.label(RichText::new("ENCRYPTED DM").color(color));
                    }
                });

                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    ui.menu_button(RichText::new("=").size(13.0), |ui| {
                        if !render_data.is_main_event
                            && note.event.kind != EventKind::EncryptedDirectMessage
                        {
                            if ui.button("View Thread").clicked() {
                                app.set_page(Page::Feed(FeedKind::Thread {
                                    id: note.event.id,
                                    referenced_by: note.event.id,
                                }));
                            }
                        }
                        if ui.button("Copy nevent").clicked() {
                            let event_pointer = EventPointer {
                                id: note.event.id,
                                relays: match GLOBALS.events.get_seen_on(&note.event.id) {
                                    None => vec![],
                                    Some(vec) => {
                                        vec.iter().map(|url| url.to_unchecked_url()).collect()
                                    }
                                },
                            };
                            ui.output_mut(|o| o.copied_text = event_pointer.as_bech32_string());
                        }
                        if ui.button("Copy note1 Id").clicked() {
                            ui.output_mut(|o| o.copied_text = note.event.id.as_bech32_string());
                        }
                        if ui.button("Copy hex Id").clicked() {
                            ui.output_mut(|o| o.copied_text = note.event.id.as_hex_string());
                        }
                        if ui.button("Copy Raw data").clicked() {
                            ui.output_mut(|o| {
                                o.copied_text = serde_json::to_string_pretty(&note.event).unwrap()
                            });
                        }
                        if ui.button("Dismiss").clicked() {
                            GLOBALS.dismissed.blocking_write().push(note.event.id);
                        }
                        if Some(note.event.pubkey) == GLOBALS.signer.public_key()
                            && note.deletion.is_none()
                        {
                            if ui.button("Delete").clicked() {
                                let _ = GLOBALS
                                    .to_overlord
                                    .send(ToOverlordMessage::DeletePost(note.event.id));
                            }
                        }
                        if ui.button("Rerender").clicked() {
                            app.notes.cache_invalidate_note(&note.event.id);
                        }
                    });
                    ui.add_space(4.0);

                    let is_thread_view: bool = {
                        let feed_kind = GLOBALS.feed.get_feed_kind();
                        matches!(feed_kind, FeedKind::Thread { .. })
                    };

                    if is_thread_view && note.event.replies_to().is_some() {
                        if collapsed {
                            let color = app.settings.theme.warning_marker_text_color();
                            if ui
                                .button(RichText::new("▼").size(13.0).color(color))
                                .on_hover_text("Expand thread")
                                .clicked()
                            {
                                app.collapsed.retain(|&id| id != note.event.id);
                            }
                        } else {
                            if ui
                                .button(RichText::new("△").size(13.0))
                                .on_hover_text("Collapse thread")
                                .clicked()
                            {
                                app.collapsed.push(note.event.id);
                            }
                        }
                        ui.add_space(4.0);
                    }

                    if !render_data.is_main_event
                        && note.event.kind != EventKind::EncryptedDirectMessage
                    {
                        if ui
                            .button(RichText::new("◉").size(13.0))
                            .on_hover_text("View Thread")
                            .clicked()
                        {
                            app.set_page(Page::Feed(FeedKind::Thread {
                                id: note.event.id,
                                referenced_by: note.event.id,
                            }));
                        }
                    }

                    ui.add_space(4.0);

                    let mut seen_on_popup_position = ui.next_widget_position();
                    seen_on_popup_position.y += 18.0; // drop below the icon itself

                    if ui
                        .add(Label::new(RichText::new("👁").size(12.0)).sense(Sense::hover()))
                        .hovered()
                    {
                        egui::Area::new(ui.next_auto_id())
                            .movable(false)
                            .interactable(false)
                            // .pivot(Align2::RIGHT_TOP) // Fails to work as advertised
                            .fixed_pos(seen_on_popup_position)
                            // FIXME IN EGUI: constrain is moving the box left for all of these boxes
                            // even if they have different IDs and don't need it.
                            .constrain(true)
                            .show(ctx, |ui| {
                                ui.set_min_width(200.0);
                                egui::Frame::popup(&app.settings.theme.get_style()).show(
                                    ui,
                                    |ui| {
                                        if let Some(urls) =
                                            GLOBALS.events.get_seen_on(&note.event.id)
                                        {
                                            for url in urls.iter() {
                                                ui.label(url.as_str());
                                            }
                                        } else {
                                            ui.label("unknown");
                                        }
                                    },
                                );
                            });
                    }

                    ui.label(
                        RichText::new(crate::date_ago::date_ago(note.event.created_at))
                            .italics()
                            .weak(),
                    )
                    .on_hover_ui(|ui| {
                        if let Ok(stamp) =
                            time::OffsetDateTime::from_unix_timestamp(note.event.created_at.0)
                        {
                            if let Ok(formatted) =
                                stamp.format(&time::format_description::well_known::Rfc2822)
                            {
                                ui.label(formatted);
                            }
                        }
                    });
                });
            });

            ui.add_space(2.0);

            // MAIN CONTENT
            if !collapsed {
                if note.author.muted > 0 {
                    ui.label(RichText::new("MUTED POST").monospace().italics());
                } else {
                    render_content(
                        app,
                        ui,
                        ctx,
                        note_ref.clone(),
                        note.deletion.is_some(),
                        content_margin_left,
                        content_pull_top,
                    );
                }

                // deleted?
                if let Some(delete_reason) = &note.deletion {
                    Frame::none()
                        .inner_margin(Margin {
                            left: footer_margin_left,
                            bottom: 0.0,
                            right: 0.0,
                            top: 8.0,
                        })
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("Deletion Reason: {}", delete_reason))
                                    .italics(),
                            );
                        });
                }

                // Footer
                if !hide_footer {
                    Frame::none()
                        .inner_margin(Margin {
                            left: footer_margin_left,
                            bottom: 0.0,
                            right: 0.0,
                            top: 8.0,
                        })
                        .outer_margin(Margin {
                            left: 0.0,
                            bottom: 0.0,
                            right: 0.0,
                            top: 0.0,
                        })
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                if ui
                                    .add(CopyButton {})
                                    .on_hover_text("Copy Contents")
                                    .clicked()
                                {
                                    if app.render_raw == Some(note.event.id) {
                                        ui.output_mut(|o| {
                                            o.copied_text =
                                                serde_json::to_string(&note.event).unwrap()
                                        });
                                    } else {
                                        ui.output_mut(|o| {
                                            o.copied_text = note.event.content.clone()
                                        });
                                    }
                                }

                                ui.add_space(24.0);

                                if render_data.can_post
                                    && note.event.kind != EventKind::EncryptedDirectMessage
                                {
                                    // Button to Repost
                                    if ui
                                        .add(
                                            Label::new(RichText::new("↻").size(18.0))
                                                .sense(Sense::click()),
                                        )
                                        .on_hover_text("Repost")
                                        .clicked()
                                    {
                                        app.draft_repost = Some(note.event.id);
                                        app.replying_to = None;
                                        app.show_post_area = true;
                                    }

                                    ui.add_space(24.0);

                                    // Button to quote note
                                    if ui
                                        .add(
                                            Label::new(RichText::new("“…”").size(18.0))
                                                .sense(Sense::click()),
                                        )
                                        .on_hover_text("Quote")
                                        .clicked()
                                    {
                                        if !app.draft.ends_with(' ') && !app.draft.is_empty() {
                                            app.draft.push(' ');
                                        }
                                        let event_pointer = EventPointer {
                                            id: note.event.id,
                                            relays: match GLOBALS.events.get_seen_on(&note.event.id)
                                            {
                                                None => vec![],
                                                Some(vec) => vec
                                                    .iter()
                                                    .map(|url| url.to_unchecked_url())
                                                    .collect(),
                                            },
                                        };
                                        app.draft.push_str(&event_pointer.as_bech32_string());
                                        app.draft_repost = None;
                                        app.replying_to = None;
                                        app.show_post_area = true;
                                        app.draft_needs_focus = true;
                                    }

                                    ui.add_space(24.0);

                                    // Button to reply
                                    if ui
                                        .add(
                                            Label::new(RichText::new("💬").size(18.0))
                                                .sense(Sense::click()),
                                        )
                                        .on_hover_text("Reply")
                                        .clicked()
                                    {
                                        app.replying_to = Some(note.event.id);
                                        app.draft_repost = None;
                                        app.show_post_area = true;
                                        app.draft_needs_focus = true;
                                    }

                                    ui.add_space(24.0);
                                }

                                // Button to render raw
                                if ui
                                    .add(
                                        Label::new(RichText::new("🥩").size(13.0))
                                            .sense(Sense::click()),
                                    )
                                    .on_hover_text("Raw")
                                    .clicked()
                                {
                                    if app.render_raw != Some(note.event.id) {
                                        app.render_raw = Some(note.event.id);
                                    } else {
                                        app.render_raw = None;
                                    }
                                }

                                ui.add_space(24.0);

                                // Button to render QR code
                                if ui
                                    .add(
                                        Label::new(RichText::new("⚃").size(16.0))
                                            .sense(Sense::click()),
                                    )
                                    .on_hover_text("QR Code")
                                    .clicked()
                                {
                                    if app.render_qr != Some(note.event.id) {
                                        app.render_qr = Some(note.event.id);
                                        app.qr_codes.remove("feedqr");
                                    } else {
                                        app.render_qr = None;
                                        app.qr_codes.remove("feedqr");
                                    }
                                }

                                ui.add_space(24.0);

                                // Buttons to react and reaction counts
                                if app.settings.reactions {
                                    let default_reaction_icon = match note.self_already_reacted {
                                        true => "♥",
                                        false => "♡",
                                    };
                                    if ui
                                        .add(
                                            Label::new(
                                                RichText::new(default_reaction_icon).size(20.0),
                                            )
                                            .sense(Sense::click()),
                                        )
                                        .clicked()
                                    {
                                        if !render_data.can_post {
                                            *GLOBALS.status_message.blocking_write() =
                                                "Your key is not setup.".to_string();
                                        } else {
                                            let _ =
                                                GLOBALS.to_overlord.send(ToOverlordMessage::Like(
                                                    note.event.id,
                                                    note.event.pubkey,
                                                ));
                                        }
                                    }
                                    for (ch, count) in note.reactions.iter() {
                                        if *ch == '+' {
                                            ui.label(format!("{}", count));
                                        }
                                    }
                                    ui.add_space(12.0);
                                    for (ch, count) in note.reactions.iter() {
                                        if *ch != '+' {
                                            ui.label(
                                                RichText::new(format!("{} {}", ch, count)).strong(),
                                            );
                                        }
                                    }
                                }
                            });
                        });
                }
            }
        });
    }
}

fn thin_separator(ui: &mut Ui, stroke: Stroke) {
    let mut style = ui.style_mut();
    style.visuals.widgets.noninteractive.bg_stroke = stroke;
    ui.add(Separator::default().spacing(0.0));
    ui.reset_style();
}

fn render_subject(ui: &mut Ui, event: &Event) {
    if let Some(subject) = event.subject() {
        ui.style_mut().spacing.item_spacing.x = 0.0;
        ui.style_mut().spacing.item_spacing.y = 10.0;
        ui.label(RichText::new(subject).text_style(TextStyle::Name("subject".into())));
        ui.end_row();
        ui.reset_style();
    }
}

fn render_content(
    app: &mut GossipUi,
    ui: &mut Ui,
    ctx: &Context,
    note_ref: Rc<RefCell<NoteData>>,
    as_deleted: bool,
    content_margin_left: f32,
    content_pull_top: f32,
) {
    if let Ok(note) = note_ref.try_borrow() {
        let event = &note.event;

        let bottom_of_avatar = ui.cursor().top();

        Frame::none()
            .inner_margin(Margin {
                left: content_margin_left,
                bottom: 0.0,
                right: 0.0,
                top: 0.0,
            })
            .outer_margin(Margin {
                left: 0.0,
                bottom: 0.0,
                right: 0.0,
                top: content_pull_top,
            })
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if app.render_raw == Some(event.id) {
                        ui.label(serde_json::to_string_pretty(&event).unwrap());
                    } else if app.render_qr == Some(event.id) {
                        app.render_qr(ui, ctx, "feedqr", event.content.trim());
                    // FIXME should this be the unmodified content (event.content)?
                    } else if event.content_warning().is_some() && !app.approved.contains(&event.id)
                    {
                        ui.label(
                            RichText::new(format!(
                                "Content-Warning: {}",
                                event.content_warning().unwrap()
                            ))
                            .monospace()
                            .italics(),
                        );
                        if ui.button("Show Post").clicked() {
                            app.approved.insert(event.id);
                            app.height.remove(&event.id); // will need to be recalculated.
                        }
                    } else if note.repost == Some(RepostType::Kind6Embedded) {
                        if note.embedded_event.is_some() {
                            let inner_note_data =
                                NoteData::new(note.embedded_event.clone().unwrap());
                            let inner_ref = Rc::new(RefCell::new(inner_note_data));
                            render_repost(
                                app,
                                ui,
                                ctx,
                                &note.repost,
                                inner_ref,
                                content_margin_left,
                                bottom_of_avatar,
                            );
                        }
                    } else {
                        // Possible subject line
                        render_subject(ui, event);

                        content::render_content(
                            app,
                            ui,
                            ctx,
                            note_ref.clone(),
                            as_deleted,
                            content_margin_left,
                            bottom_of_avatar,
                        );
                    }
                });
            });
    }
}

fn render_repost(
    app: &mut GossipUi,
    ui: &mut Ui,
    ctx: &Context,
    parent_repost: &Option<RepostType>,
    repost_ref: Rc<RefCell<NoteData>>,
    content_margin_left: f32,
    bottom_of_avatar: f32,
) {
    if let Ok(repost_data) = repost_ref.try_borrow() {
        let render_data = NoteRenderData {
            height: 0.0,
            has_repost: repost_data.repost.is_some(),
            is_comment_mention: *parent_repost == Some(RepostType::CommentMention),
            is_new: false,
            is_main_event: false,
            is_thread: false,
            is_first: false,
            is_last: false,
            thread_position: 0,
            can_post: GLOBALS.signer.is_ready(),
        };

        let push_top = {
            let diff = bottom_of_avatar - ui.cursor().top();
            if diff > 0.0 {
                diff
            } else {
                0.0
            }
        };

        // insert a newline if the current line has text
        if ui.cursor().min.x > ui.max_rect().min.x {
            ui.end_row();
        }

        ui.vertical(|ui| {
            Frame::none()
                .inner_margin(app.settings.theme.repost_inner_margin(&render_data))
                .outer_margin({
                    let mut margin = app.settings.theme.repost_outer_margin(&render_data);
                    margin.left -= content_margin_left;
                    margin.top += push_top;
                    margin
                })
                .rounding(app.settings.theme.repost_rounding(&render_data))
                .shadow(app.settings.theme.repost_shadow(&render_data))
                .fill(app.settings.theme.repost_fill(&render_data))
                .stroke(app.settings.theme.repost_stroke(&render_data))
                .show(ui, |ui| {
                    ui.add_space(
                        app.settings
                            .theme
                            .repost_space_above_separator_before(&render_data),
                    );
                    thin_separator(
                        ui,
                        app.settings
                            .theme
                            .repost_separator_before_stroke(&render_data),
                    );
                    ui.add_space(
                        app.settings
                            .theme
                            .repost_space_below_separator_before(&render_data),
                    );
                    ui.horizontal_wrapped(|ui| {
                        // FIXME: don't recurse forever
                        render_note_inner(
                            app,
                            ctx,
                            ui,
                            repost_ref.clone(),
                            &render_data,
                            false,
                            parent_repost,
                        );
                    });
                    ui.add_space(
                        app.settings
                            .theme
                            .repost_space_above_separator_after(&render_data),
                    );
                    thin_separator(
                        ui,
                        app.settings
                            .theme
                            .repost_separator_after_stroke(&render_data),
                    );
                    ui.add_space(
                        app.settings
                            .theme
                            .repost_space_below_separator_after(&render_data),
                    );
                });
        });
    }
}
