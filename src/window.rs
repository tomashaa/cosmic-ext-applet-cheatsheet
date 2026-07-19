// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Tomas Haaland

use cosmic::app::{Core, Task};
use cosmic::iced::core::window;
use cosmic::iced::window::Id;
use cosmic::iced::{Length, Limits, Rectangle};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget;
use cosmic::Element;

use crate::config::{self, CustomShortcut};
use crate::data::{ACTIONS, SECTIONS};

const ID: &str = "io.github.tomashaa.CosmicExtCheatsheet";

pub struct Window {
    core: Core,
    popup: Option<Id>,
    search: String,
    custom: Vec<CustomShortcut>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            search: String::new(),
            custom: config::load(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
    Search(String),
    Launch(Vec<String>),
}

fn spawn(argv: &[String]) {
    if let Some((cmd, args)) = argv.split_first() {
        if let Err(e) = std::process::Command::new(cmd).args(args).spawn() {
            log::warn!("failed to launch {cmd}: {e}");
        }
    }
}

fn matches(q: &str, a: &str, b: &str) -> bool {
    q.is_empty() || a.to_lowercase().contains(q) || b.to_lowercase().contains(q)
}

/// One cheat-sheet row: name on the left, shortcut on the right, optional
/// zebra-dimmed background, clickable when `argv` is set.
fn row_view(
    label: String,
    keys: String,
    argv: Option<Vec<String>>,
    dim: bool,
) -> Element<'static, Message> {
    let content = widget::row::with_children(vec![
        widget::text(label).width(Length::Fill).into(),
        widget::text(keys).size(12).into(),
    ])
    .spacing(12);

    let clickable = argv.is_some();
    let mut cell = widget::container(content).width(Length::Fill).padding(8);
    if clickable {
        // Faint accent tint marks a row as clickable (a "link").
        cell = cell.class(cosmic::theme::Container::custom(|theme| {
            let mut c: cosmic::iced::Color = theme.cosmic().accent_color().into();
            c.a = 0.10;
            cosmic::widget::container::Style {
                background: Some(cosmic::iced::Background::Color(c)),
                ..Default::default()
            }
        }));
    } else if dim {
        // Subtle zebra stripe on informational rows for readability.
        cell = cell.class(cosmic::theme::Container::custom(|theme| {
            let mut c: cosmic::iced::Color = theme.cosmic().primary_container_divider().into();
            c.a = 0.14;
            cosmic::widget::container::Style {
                background: Some(cosmic::iced::Background::Color(c)),
                ..Default::default()
            }
        }));
    }

    match argv {
        Some(cmd) => widget::mouse_area(cell).on_press(Message::Launch(cmd)).into(),
        None => cell.into(),
    }
}

impl Window {
    /// Build the scrollable cheat-sheet body shown inside the popup.
    fn body(&self) -> Element<'_, Message> {
        let q = self.search.to_lowercase();
        let mut children: Vec<Element<Message>> = Vec::new();

        children.push(
            widget::text_input::search_input("Search shortcuts…", &self.search)
                .on_input(Message::Search)
                .into(),
        );

        // Clickable actions.
        let mut acts: Vec<Element<Message>> = Vec::new();
        let mut i = 0;
        for a in ACTIONS {
            if !matches(&q, a.label, a.keys) {
                continue;
            }
            let argv: Vec<String> = a.command.iter().map(|s| s.to_string()).collect();
            acts.push(row_view(
                format!("{}  {}", a.icon, a.label),
                a.keys.to_string(),
                Some(argv),
                i % 2 == 1,
            ));
            i += 1;
        }
        if !acts.is_empty() {
            children.push(widget::text::heading("Actions").into());
            children.extend(acts);
        }

        // Informational sections.
        for s in SECTIONS {
            let mut rows: Vec<Element<Message>> = Vec::new();
            let mut j = 0;
            for &(l, k) in s.rows {
                if !matches(&q, l, k) {
                    continue;
                }
                rows.push(row_view(l.to_string(), k.to_string(), None, j % 2 == 1));
                j += 1;
            }
            if rows.is_empty() {
                continue;
            }
            children.push(widget::text::heading(s.title).into());
            children.extend(rows);
        }

        // User-defined custom shortcuts, grouped by their optional section.
        let custom: Vec<&CustomShortcut> = self
            .custom
            .iter()
            .filter(|c| matches(&q, &c.label, &c.keys))
            .collect();
        let mut sections: Vec<&str> = Vec::new();
        for c in &custom {
            let s = c.section_or_default();
            if !sections.contains(&s) {
                sections.push(s);
            }
        }
        for sec in sections {
            children.push(widget::text::heading(sec).into());
            let mut j = 0;
            for c in custom.iter().filter(|c| c.section_or_default() == sec) {
                let argv = c.argv();
                let arg = if argv.is_empty() { None } else { Some(argv) };
                children.push(row_view(c.label.clone(), c.keys.clone(), arg, j % 2 == 1));
                j += 1;
            }
        }

        let col = widget::column::with_children(children).spacing(2).padding(8);
        widget::scrollable(col)
            .height(Length::Fixed(540.0))
            .into()
    }
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Message>) {
        (
            Window {
                core,
                ..Default::default()
            },
            Task::none(),
        )
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Search(q) => {
                self.search = q;
            }
            Message::Launch(argv) => {
                spawn(&argv);
                if let Some(id) = self.popup.take() {
                    return cosmic::task::message(cosmic::Action::Cosmic(
                        cosmic::app::Action::Surface(destroy_popup(id)),
                    ));
                }
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let have_popup = self.popup;
        let btn = self
            .core
            .applet
            .icon_button("input-keyboard-symbolic")
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = have_popup {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<Window>(
                        |_| Default::default(),
                        move |state: &mut Window| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            state.search.clear();
                            let mut popup_settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );
                            popup_settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            // Lock the popup size so it opens at its final size
                            // instead of momentarily rendering zoomed-in.
                            popup_settings.positioner.size_limits = Limits::NONE
                                .min_width(440.0)
                                .max_width(440.0)
                                .min_height(200.0)
                                .max_height(640.0);
                            popup_settings
                        },
                        Some(Box::new(move |state: &Window| {
                            Element::from(state.core.applet.popup_container(state.body()))
                                .map(cosmic::Action::App)
                        })),
                    ))
                }
            });

        Element::from(self.core.applet.applet_tooltip::<Message>(
            btn,
            "Keyboard shortcuts",
            self.popup.is_some(),
            Message::Surface,
            None,
        ))
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        "".into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
