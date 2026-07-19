// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Tomas Haaland

use cosmic::app::{Core, Task};
use cosmic::iced::core::window;
use cosmic::iced::window::Id;
use cosmic::iced::{Length, Rectangle};
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
        let mut actions: Vec<Element<Message>> = Vec::new();
        for a in ACTIONS {
            if !matches(&q, a.label, a.keys) {
                continue;
            }
            let label = format!("{}  {}", a.icon, a.label);
            let btn = widget::button::text(format!("{label}    [{}]", a.keys))
                .width(Length::Fill)
                .on_press(Message::Launch(a.command.iter().map(|s| s.to_string()).collect()));
            actions.push(btn.into());
        }
        if !actions.is_empty() {
            children.push(widget::text::heading("Actions").into());
            children.extend(actions);
        }

        // Informational sections.
        for s in SECTIONS {
            let rows: Vec<Element<Message>> = s
                .rows
                .iter()
                .filter(|(l, k)| matches(&q, l, k))
                .map(|(l, k)| widget::text(format!("{l}    {k}")).into())
                .collect();
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
        // Distinct sections, in first-seen order.
        let mut sections: Vec<&str> = Vec::new();
        for c in &custom {
            let s = c.section_or_default();
            if !sections.contains(&s) {
                sections.push(s);
            }
        }
        for sec in sections {
            children.push(widget::text::heading(sec).into());
            for c in custom.iter().filter(|c| c.section_or_default() == sec) {
                let argv = c.argv();
                if argv.is_empty() {
                    children.push(widget::text(format!("{}    {}", c.label, c.keys)).into());
                } else {
                    children.push(
                        widget::button::text(format!("{}    [{}]", c.label, c.keys))
                            .width(Length::Fill)
                            .on_press(Message::Launch(argv))
                            .into(),
                    );
                }
            }
        }

        let col = widget::column::with_children(children).spacing(4).padding(8);
        widget::scrollable(col)
            .height(Length::Fixed(520.0))
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
