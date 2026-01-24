// cosmic-connect-applet/src/ui/popup.rs
use cosmic::iced::{Alignment, Length};
use cosmic::{widget, Element};
use std::collections::HashMap;
use crate::models::Device;
use crate::messages::Message;

pub fn create_popup_view<'a>(devices: &'a HashMap<String, Device>, expanded_device: Option<&'a String>, _expanded_player_menu: Option<&'a String>) -> Element<'a, Message> {
    let spacing = cosmic::theme::active().cosmic().spacing;
    let mut content = widget::column().spacing(spacing.space_s).padding(spacing.space_s);

    // Header
    content = content.push(
        widget::row()
            .push(widget::text("Cosmic Connect").size(18).width(Length::Fill))
            .push(
                widget::button::standard("Settings")
                    .on_press(Message::OpenSettings)
            )
            .spacing(spacing.space_xs)
            .align_y(Alignment::Center)
    );

    content = content.push(widget::divider::horizontal::default());

    // Pairing requests - SORTED alphabetically
    let mut pairing_requests: Vec<_> = devices.values()
        .filter(|d| d.pairing_requests > 0 && !d.is_paired)
        .collect();
    
    // Sort pairing requests by device name
    pairing_requests.sort_by(|a, b| a.name.cmp(&b.name));

    if !pairing_requests.is_empty() {
        content = content.push(widget::text("Pairing Requests").size(14).font(cosmic::font::bold()));
        
        for device in pairing_requests {
            let device_id_accept = device.id.clone();
            let device_id_reject = device.id.clone();
            
            let request_card = widget::container(
                widget::column()
                    .push(
                        widget::row()
                            .push(widget::icon::from_name(device.device_icon()).size(24))
                            .push(
                                widget::column()
                                    .push(widget::text(&device.name).size(14))
                                    .push(widget::text(&device.device_type).size(11))
                                    .spacing(spacing.space_xxxs)
                            )
                            .spacing(spacing.space_s)
                            .align_y(Alignment::Center)
                    )
                    .push(widget::Space::with_height(Length::Fixed(spacing.space_xs as f32)))
                    .push(
                        widget::row()
                            .push(
                                widget::button::suggested("Accept")
                                    .on_press(Message::AcceptPairing(device_id_accept))
                                    .width(Length::Fill)
                            )
                            .push(
                                widget::button::destructive("Reject")
                                    .on_press(Message::RejectPairing(device_id_reject))
                                    .width(Length::Fill)
                            )
                            .spacing(spacing.space_xs)
                    )
                    .spacing(spacing.space_xs)
            )
            .padding(spacing.space_s)
            .class(cosmic::theme::Container::Card)
            .width(Length::Fill);
            
            content = content.push(request_card);
        }
        
        content = content.push(widget::divider::horizontal::default());
    }

    // Connected devices - SORTED alphabetically
    let mut connected_devices: Vec<_> = devices.values()
        .filter(|d| d.is_reachable && d.is_paired)
        .collect();
    
    // Sort connected devices by name
    connected_devices.sort_by(|a, b| a.name.cmp(&b.name));

    if !connected_devices.is_empty() {
        content = content.push(widget::text("Connected Devices").size(14).font(cosmic::font::bold()));
        
        for device in connected_devices {
            let is_expanded = expanded_device.is_some() && expanded_device.unwrap() == &device.id;
            
            let battery_text = if let Some(level) = device.battery_level {
                format!("{}%", level)
            } else {
                String::new()
            };
            
            let mut device_row = widget::row()
                .push(widget::icon::from_name(device.device_icon()).size(24))
                .push(
                    widget::column()
                        .push(widget::text(&device.name).size(14))
                        .push(widget::text(&device.device_type).size(11))
                        .spacing(spacing.space_xxxs)
                        .width(Length::Fill)
                )
                .spacing(spacing.space_s)
                .align_y(Alignment::Center);
            
            if !battery_text.is_empty() {
                device_row = device_row.push(widget::text(battery_text).size(11));
            }
            
            device_row = device_row.push(
                widget::button::icon(
                    widget::icon::from_name(if is_expanded { "go-up-symbolic" } else { "go-down-symbolic" })
                )
                .on_press(Message::ToggleDeviceMenu(device.id.clone()))
                .class(cosmic::theme::Button::Icon)
            );
            
            let device_button = widget::button::custom(device_row)
                .on_press(Message::ToggleDeviceMenu(device.id.clone()))
                .width(Length::Fill)
                .class(cosmic::theme::Button::Text);
            
            content = content.push(device_button);
            
            if is_expanded {
                let mut menu_items = widget::column().spacing(spacing.space_xxs);
                
                // Quick actions section
                menu_items = menu_items.push(widget::text("Quick Actions").size(12).font(cosmic::font::bold()));
                
                menu_items = menu_items.push(
                    widget::button::text("Ping")
                        .on_press(Message::PingDevice(device.id.clone()))
                        .width(Length::Fill)
                        .class(cosmic::theme::Button::Text)
                );
                
                if device.has_findmyphone {
                    menu_items = menu_items.push(
                        widget::button::text("Find my phone")
                            .on_press(Message::RingDevice(device.id.clone()))
                            .width(Length::Fill)
                            .class(cosmic::theme::Button::Text)
                    );
                }
                
                if device.has_clipboard {
                    menu_items = menu_items.push(
                        widget::button::text("Share clipboard")
                            .on_press(Message::ShareClipboard(device.id.clone()))
                            .width(Length::Fill)
                            .class(cosmic::theme::Button::Text)
                    );
                }
                
                // File operations section
                if device.has_share || device.has_sftp {
                    menu_items = menu_items.push(widget::divider::horizontal::light());
                    menu_items = menu_items.push(widget::text("Files").size(12).font(cosmic::font::bold()));
                    
                    if device.has_share {
                        menu_items = menu_items.push(
                            widget::button::text("Send file")
                                .on_press(Message::SendFiles(device.id.clone()))
                                .width(Length::Fill)
                                .class(cosmic::theme::Button::Text)
                        );
                    }
                    
                    if device.has_sftp {
                        menu_items = menu_items.push(
                            widget::button::text("Browse device")
                                .on_press(Message::BrowseDevice(device.id.clone()))
                                .width(Length::Fill)
                                .class(cosmic::theme::Button::Text)
                        );
                    }
                }
                
                let menu_container = widget::container(menu_items)
                    .padding([spacing.space_xs, spacing.space_m])
                    .class(cosmic::theme::Container::Background);
                
                content = content.push(menu_container);
            }
        }
    } else {
        content = content.push(
            widget::container(
                widget::text("No connected devices")
                    .size(12)
            )
            .padding(spacing.space_s)
            .width(Length::Fill)
            .center_x(Length::Fill)
        );
    }

    widget::scrollable(content)
        .height(Length::Fill)
        .into()
}