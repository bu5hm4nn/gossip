use super::{GossipUi, Page};
use crate::comms::ToOverlordMessage;
#[cfg(not(feature = "side-menu"))]
use crate::feed::FeedKind;
use crate::globals::{Globals, GLOBALS};
use crate::ui::widgets::CopyButton;
use eframe::egui;
use egui::style::Margin;
use egui::{Color32, Context, Frame, ScrollArea, Stroke, Ui, Vec2};
use nostr_types::{KeySecurity, PublicKeyHex};
use zeroize::Zeroize;

mod delegation;
mod metadata;

pub(super) fn update(app: &mut GossipUi, ctx: &Context, _frame: &mut eframe::Frame, ui: &mut Ui) {
    #[cfg(not(feature = "side-menu"))]
    {
        ui.horizontal(|ui| {
            if ui
                .add(egui::SelectableLabel::new(
                    app.page == Page::YourKeys,
                    "Keys",
                ))
                .clicked()
            {
                app.set_page(Page::YourKeys);
            }
            ui.separator();
            if let Some(pubkeyhex) = GLOBALS.signer.public_key() {
                if ui
                    .add(egui::SelectableLabel::new(
                        app.page == Page::Feed(FeedKind::Person(pubkeyhex.into())),
                        "Notes »",
                    ))
                    .clicked()
                {
                    app.set_page(Page::Feed(FeedKind::Person(pubkeyhex.into())));
                }
                ui.separator();
            }
            if ui
                .add(egui::SelectableLabel::new(
                    app.page == Page::YourMetadata,
                    "Metadata",
                ))
                .clicked()
            {
                app.set_page(Page::YourMetadata);
            }
            ui.separator();
            if ui
                .add(egui::SelectableLabel::new(
                    app.page == Page::YourDelegation,
                    "Delegation",
                ))
                .clicked()
            {
                app.set_page(Page::YourDelegation);
            }
            ui.separator();
        });
        ui.separator();
    }

    if app.page == Page::YourKeys {
        ui.add_space(10.0);
        ui.heading("Your Keys");

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ScrollArea::vertical()
            .id_source("your_keys")
            .override_scroll_delta(Vec2 {
                x: 0.0,
                y: app.current_scroll_offset,
            })
            .show(ui, |ui| {
                if GLOBALS.signer.is_ready() {
                    ui.heading("Ready to sign events");

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    show_pub_key_detail(app, ctx, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    show_priv_key_detail(app, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    offer_change_password(app, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    offer_export_priv_key(app, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    offer_delete(app, ui);
                } else if GLOBALS.signer.is_loaded() {
                    Frame::none()
                        .stroke(Stroke {
                            width: 2.0,
                            color: Color32::RED,
                        })
                        .inner_margin(Margin {
                            left: 10.0,
                            right: 10.0,
                            top: 10.0,
                            bottom: 10.0,
                        })
                        .show(ui, |ui| {
                            ui.heading("Passphrase Needed");
                            offer_unlock_priv_key(app, ui);
                        });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    show_pub_key_detail(app, ctx, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    offer_delete(app, ui);
                } else if GLOBALS.signer.public_key().is_some() {
                    show_pub_key_detail(app, ctx, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    offer_import_priv_key(app, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    offer_delete_or_import_pub_key(app, ui);
                } else {
                    offer_generate(app, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    offer_import_priv_key(app, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    offer_delete_or_import_pub_key(app, ui);
                }
            });
    } else if app.page == Page::YourMetadata {
        metadata::update(app, ctx, _frame, ui);
    } else if app.page == Page::YourDelegation {
        delegation::update(app, ctx, _frame, ui);
    }
}

fn show_pub_key_detail(app: &mut GossipUi, ctx: &Context, ui: &mut Ui) {
    // Render public key if available
    if let Some(public_key) = GLOBALS.signer.public_key() {
        ui.heading("Public Key");
        ui.add_space(10.0);

        let pkhex: PublicKeyHex = public_key.into();
        ui.horizontal_wrapped(|ui| {
            ui.label(&format!("Public Key (Hex): {}", pkhex.as_str()));
            if ui.add(CopyButton {}).clicked() {
                ui.output_mut(|o| o.copied_text = pkhex.into_string());
            }
        });

        let bech32 = public_key.as_bech32_string();
        ui.horizontal_wrapped(|ui| {
            ui.label(&format!("Public Key (bech32): {}", bech32));
            if ui.add(CopyButton {}).clicked() {
                ui.output_mut(|o| o.copied_text = bech32.clone());
            }
        });
        ui.add_space(10.0);
        app.render_qr(ui, ctx, "you_npub_qr", &bech32);

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        if let Some(profile) = Globals::get_your_nprofile() {
            ui.heading("N-Profile");
            ui.add_space(10.0);

            let nprofile = profile.as_bech32_string();
            ui.horizontal_wrapped(|ui| {
                ui.label(&format!("Your Profile: {}", &nprofile));
                if ui.add(CopyButton {}).clicked() {
                    ui.output_mut(|o| o.copied_text = nprofile.clone());
                }
            });
            ui.add_space(10.0);
            app.render_qr(ui, ctx, "you_nprofile_qr", &nprofile);
        }
    }
}

pub(super) fn offer_unlock_priv_key(app: &mut GossipUi, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Passphrase: ");
        let response = ui.add(text_edit_line!(app, app.password).password(true));
        if app.unlock_needs_focus {
            response.request_focus();
            app.unlock_needs_focus = false;
        }
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let _ = GLOBALS
                .to_overlord
                .send(ToOverlordMessage::UnlockKey(app.password.clone()));
            app.password.zeroize();
            app.password = "".to_owned();
            app.draft_needs_focus = true;
        }
        if ui.button("Unlock Private Key").clicked() {
            let _ = GLOBALS
                .to_overlord
                .send(ToOverlordMessage::UnlockKey(app.password.clone()));
            app.password.zeroize();
            app.password = "".to_owned();
            app.draft_needs_focus = true;
        }
    });
}

fn show_priv_key_detail(_app: &mut GossipUi, ui: &mut Ui) {
    let key_security = GLOBALS.signer.key_security().unwrap();

    if let Some(epk) = GLOBALS.signer.encrypted_private_key() {
        ui.heading("Encrypted Private Key");
        ui.horizontal_wrapped(|ui| {
            ui.label(&epk.0);
            if ui.add(CopyButton {}).clicked() {
                ui.output_mut(|o| o.copied_text = epk.to_string());
            }
        });

        ui.add_space(10.0);

        ui.label(&*format!(
            "Private Key security is {}",
            match key_security {
                KeySecurity::Weak => "weak",
                KeySecurity::Medium => "medium",
            }
        ));
    }
}

fn offer_change_password(app: &mut GossipUi, ui: &mut Ui) {
    ui.heading("Change Passphrase");

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label("Enter Existing Passphrase: ");
        ui.add(text_edit_line!(app, app.password).password(true));
    });

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label("Enter New Passphrase: ");
        ui.add(text_edit_line!(app, app.password2).password(true));
    });

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label("Repeat New Passphrase: ");
        ui.add(text_edit_line!(app, app.password3).password(true));
    });

    if ui.button("Change Passphrase").clicked() {
        if app.password2 != app.password3 {
            *GLOBALS.status_message.blocking_write() = "Passphrases do not match.".to_owned();
            app.password2.zeroize();
            app.password2 = "".to_owned();
            app.password3.zeroize();
            app.password3 = "".to_owned();
        } else {
            let _ = GLOBALS
                .to_overlord
                .send(ToOverlordMessage::ChangePassphrase(
                    app.password.clone(),
                    app.password2.clone(),
                ));
            app.password.zeroize();
            app.password = "".to_owned();
            app.password2.zeroize();
            app.password2 = "".to_owned();
            app.password3.zeroize();
            app.password3 = "".to_owned();
        }
    }
}

fn offer_export_priv_key(app: &mut GossipUi, ui: &mut Ui) {
    let key_security = GLOBALS.signer.key_security().unwrap();

    ui.heading("Raw Export");
    if key_security == KeySecurity::Medium {
        ui.label("WARNING: This will downgrade your key security to WEAK");
    }

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.label("Enter Passphrase To Export: ");
        ui.add(text_edit_line!(app, app.password).password(true));
    });

    if ui.button("Export Private Key as bech32").clicked() {
        match GLOBALS.signer.export_private_key_bech32(&app.password) {
            Ok(mut bech32) => {
                println!("Exported private key (bech32): {}", bech32);
                bech32.zeroize();
                *GLOBALS.status_message.blocking_write() =
                    "Exported key has been printed to the console standard output.".to_owned();
            }
            Err(e) => *GLOBALS.status_message.blocking_write() = format!("{}", e),
        }
        app.password.zeroize();
        app.password = "".to_owned();
    }
    if ui.button("Export Private Key as hex").clicked() {
        match GLOBALS.signer.export_private_key_hex(&app.password) {
            Ok(mut hex) => {
                println!("Exported private key (hex): {}", hex);
                hex.zeroize();
                *GLOBALS.status_message.blocking_write() =
                    "Exported key has been printed to the console standard output.".to_owned();
            }
            Err(e) => *GLOBALS.status_message.blocking_write() = format!("{}", e),
        }
        app.password.zeroize();
        app.password = "".to_owned();
    }
}

fn offer_import_priv_key(app: &mut GossipUi, ui: &mut Ui) {
    ui.heading("Import a Private Key");

    ui.horizontal(|ui| {
        ui.label("Enter private key");
        ui.add(
            text_edit_line!(app, app.import_priv)
                .hint_text("nsec1, or hex")
                .desired_width(f32::INFINITY)
                .password(true),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Enter a passphrase to keep it encrypted under");
        ui.add(text_edit_line!(app, app.password).password(true));
    });
    ui.horizontal(|ui| {
        ui.label("Repeat passphrase to be sure");
        ui.add(text_edit_line!(app, app.password2).password(true));
    });
    if ui.button("import").clicked() {
        if app.password != app.password2 {
            *GLOBALS.status_message.blocking_write() = "Passwords do not match".to_owned();
        } else {
            let _ = GLOBALS.to_overlord.send(ToOverlordMessage::ImportPriv(
                app.import_priv.clone(),
                app.password.clone(),
            ));
            app.import_priv.zeroize();
            app.import_priv = "".to_owned();
        }
        app.password.zeroize();
        app.password = "".to_owned();
        app.password2.zeroize();
        app.password2 = "".to_owned();
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Import an Encrypted Private Key");

    ui.horizontal(|ui| {
        ui.label("Enter encrypted private key");
        ui.add(
            text_edit_line!(app, app.import_priv)
                .hint_text("ncryptsec1")
                .desired_width(f32::INFINITY)
                .password(true),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Enter the passphrase it is encrypted under");
        ui.add(text_edit_line!(app, app.password).password(true));
    });
    if ui.button("import").clicked() {
        let _ = GLOBALS.to_overlord.send(ToOverlordMessage::ImportPriv(
            app.import_priv.clone(),
            app.password.clone(),
        ));
        app.import_priv = "".to_owned();
        app.password.zeroize();
        app.password = "".to_owned();
    }
}

fn offer_delete_or_import_pub_key(app: &mut GossipUi, ui: &mut Ui) {
    if let Some(pk) = GLOBALS.signer.public_key() {
        ui.heading("Public Key");
        ui.add_space(10.0);

        let pkhex: PublicKeyHex = pk.into();
        ui.horizontal(|ui| {
            ui.label(&format!("Public Key (Hex): {}", pkhex.as_str()));
            if ui.add(CopyButton {}).clicked() {
                ui.output_mut(|o| o.copied_text = pkhex.into_string());
            }
        });

        let bech32 = pk.as_bech32_string();
        ui.horizontal(|ui| {
            ui.label(&format!("Public Key (bech32): {}", bech32));
            if ui.add(CopyButton {}).clicked() {
                ui.output_mut(|o| o.copied_text = bech32);
            }
        });

        if ui.button("Delete this public key").clicked() {
            let _ = GLOBALS.to_overlord.send(ToOverlordMessage::DeletePub);
        }
    } else {
        ui.heading("Import a Public Key");
        ui.add_space(10.0);

        ui.label("This won't let you post or react to posts, but you can view other people's posts (and fetch your following list) with just a public key.");

        ui.horizontal_wrapped(|ui| {
            ui.label("Enter your public key");
            ui.add(
                text_edit_line!(app, app.import_pub)
                    .hint_text("npub1 or hex")
                    .desired_width(f32::INFINITY),
            );
            if ui.button("Import a Public Key").clicked() {
                let _ = GLOBALS
                    .to_overlord
                    .send(ToOverlordMessage::ImportPub(app.import_pub.clone()));
                app.import_pub = "".to_owned();
            }
        });
    }
}

fn offer_delete(app: &mut GossipUi, ui: &mut Ui) {
    ui.heading("DELETE This Identity");

    ui.horizontal_wrapped(|ui| {
        if app.delete_confirm {
            ui.label("Please confirm that you really mean to do this: ");
            if ui.button("DELETE (Yes I'm Sure)").clicked() {
                let _ = GLOBALS.to_overlord.send(ToOverlordMessage::DeletePriv);
                app.delete_confirm = false;
            }
        } else {
            if ui.button("DELETE (Cannot be undone!)").clicked() {
                app.delete_confirm = true;
            }
        }
    });
}

fn offer_generate(app: &mut GossipUi, ui: &mut Ui) {
    ui.heading("Generate a Keypair");

    ui.horizontal(|ui| {
        ui.label("Enter a passphrase to keep it encrypted under");
        ui.add(text_edit_line!(app, app.password).password(true));
    });
    ui.horizontal(|ui| {
        ui.label("Repeat passphrase to be sure");
        ui.add(text_edit_line!(app, app.password2).password(true));
    });
    if ui.button("Generate Now").clicked() {
        if app.password != app.password2 {
            *GLOBALS.status_message.blocking_write() = "Passwords do not match".to_owned();
        } else {
            let _ = GLOBALS
                .to_overlord
                .send(ToOverlordMessage::GeneratePrivateKey(app.password.clone()));
        }
        app.password.zeroize();
        app.password = "".to_owned();
        app.password2.zeroize();
        app.password2 = "".to_owned();
    }
}
