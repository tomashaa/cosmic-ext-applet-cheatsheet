// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Tomas Haaland

use cosmic::app::{Core, Task};
use cosmic::iced::core::window;
use cosmic::iced::window::Id;
use cosmic::iced::Length;
use cosmic::surface::action::destroy_popup;
use cosmic::widget;
use cosmic::Element;

use crate::config::{self, CustomShortcut};
use crate::data::{ACTIONS, SECTIONS};
use crate::i18n;

const ID: &str = "io.github.tomashaa.CosmicExtCheatsheet";

#[derive(Default, PartialEq)]
enum Mode {
    #[default]
    List,
    Edit,
}

/// Draft fields for the "add custom shortcut" form.
#[derive(Default)]
struct Form {
    label: String,
    keys: String,
    command: String,
    section: String,
    /// A note (hugselapp) instead of a shortcut: no command, free-text value.
    is_note: bool,
}

pub struct Window {
    core: Core,
    popup: Option<Id>,
    search: String,
    custom: Vec<CustomShortcut>,
    lang: &'static str,
    mode: Mode,
    form: Form,
    /// Running as a standalone window (`--window`) rather than a panel applet.
    windowed: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            core: Core::default(),
            popup: None,
            search: String::new(),
            custom: config::load(),
            lang: crate::i18n::current_lang(),
            mode: Mode::default(),
            form: Form::default(),
            windowed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
    Search(String),
    Launch(Vec<String>),
    ToggleWindow,
    OpenEditor,
    CloseEditor,
    FormLabel(String),
    FormKeys(String),
    FormCommand(String),
    FormSection(String),
    FormToggleNote(bool),
    AddCustom,
    DeleteCustom(usize),
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

fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// One cheat-sheet row: name on the left, shortcut on the right, optional
/// zebra-dimmed background, clickable when `argv` is set.
fn row_view(
    label: String,
    keys: String,
    argv: Option<Vec<String>>,
    _dim: bool,
) -> Element<'static, Message> {
    let clickable = argv.is_some();

    // Clickable actions get an accent-coloured (blue) label, like the old cheat sheet.
    let label_widget = if clickable {
        widget::text(label).class(cosmic::theme::Text::Accent)
    } else {
        widget::text(label)
    };

    // The shortcut sits in a rounded "badge" with its own background.
    let badge = widget::container(widget::text(keys).size(12))
        .padding(cosmic::iced::Padding {
            top: 2.0,
            right: 8.0,
            bottom: 2.0,
            left: 8.0,
        })
        .class(cosmic::theme::Container::custom(|theme| {
            let mut c: cosmic::iced::Color = theme.cosmic().primary_container_divider().into();
            c.a = 0.6;
            cosmic::widget::container::Style {
                background: Some(cosmic::iced::Background::Color(c)),
                border: cosmic::iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));

    let content = widget::row::with_children(vec![
        label_widget.width(Length::Fill).into(),
        badge.into(),
    ])
    .spacing(12)
    .align_y(cosmic::iced::Alignment::Center);

    let cell = widget::container(content)
        .width(Length::Fill)
        .padding(cosmic::iced::Padding {
            top: 6.0,
            right: 8.0,
            bottom: 6.0,
            left: 8.0,
        });

    match argv {
        Some(cmd) => widget::mouse_area(cell).on_press(Message::Launch(cmd)).into(),
        None => cell.into(),
    }
}

/// A section heading: accent-coloured and set apart from the rows below it.
fn heading_view(title: &str) -> Element<'static, Message> {
    widget::container(
        widget::text(title.to_uppercase())
            .size(13)
            .class(cosmic::theme::Text::Accent),
    )
    .padding(cosmic::iced::Padding {
        top: 10.0,
        right: 8.0,
        bottom: 2.0,
        left: 8.0,
    })
    .into()
}

impl Window {
    /// Build the scrollable cheat-sheet body shown inside the popup.
    fn body(&self) -> Element<'_, Message> {
        match self.mode {
            Mode::List => self.list_body(),
            Mode::Edit => self.edit_body(),
        }
    }

    fn list_body(&self) -> Element<'_, Message> {
        let q = self.search.to_lowercase();
        let mut children: Vec<Element<Message>> = Vec::new();

        // Search field + settings (gear) button.
        children.push(
            widget::row::with_children(vec![
                widget::text_input::search_input(i18n::tr(self.lang, "ui.search"), &self.search)
                    .on_input(Message::Search)
                    .width(Length::Fill)
                    .into(),
                widget::button::text("⚙")
                    .on_press(Message::OpenEditor)
                    .into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        );

        // Clickable actions.
        let mut acts: Vec<Element<Message>> = Vec::new();
        for a in ACTIONS {
            let label = i18n::tr(self.lang, a.label_key);
            if !matches(&q, label, a.keys) {
                continue;
            }
            let argv: Vec<String> = a.command.iter().map(|s| s.to_string()).collect();
            acts.push(row_view(
                format!("{}  {}", a.icon, label),
                a.keys.to_string(),
                Some(argv),
                false,
            ));
        }
        if !acts.is_empty() {
            children.push(heading_view(i18n::tr(self.lang, "ui.actions")));
            children.extend(acts);
        }

        // Informational sections.
        for s in SECTIONS {
            let mut rows: Vec<Element<Message>> = Vec::new();
            for &(label_key, k) in s.rows {
                let l = i18n::tr(self.lang, label_key);
                if !matches(&q, l, k) {
                    continue;
                }
                rows.push(row_view(l.to_string(), k.to_string(), None, false));
            }
            if rows.is_empty() {
                continue;
            }
            children.push(heading_view(i18n::tr(self.lang, s.title_key)));
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
            children.push(heading_view(sec));
            let mut j = 0;
            for c in custom.iter().filter(|c| c.section_or_default() == sec) {
                let argv = c.argv();
                let arg = if argv.is_empty() { None } else { Some(argv) };
                children.push(row_view(c.label.clone(), c.keys.clone(), arg, j % 2 == 0));
                j += 1;
            }
        }

        let col = widget::column::with_children(children).spacing(2).padding(8);
        widget::scrollable(col)
            .height(Length::Fixed(540.0))
            .into()
    }

    fn edit_body(&self) -> Element<'_, Message> {
        let mut children: Vec<Element<Message>> = Vec::new();

        // Header: back button + title.
        children.push(
            widget::row::with_children(vec![
                widget::button::text("←").on_press(Message::CloseEditor).into(),
                widget::text("Custom shortcuts")
                    .size(15)
                    .class(cosmic::theme::Text::Accent)
                    .width(Length::Fill)
                    .into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        );

        // Existing custom shortcuts, each with a delete button.
        if self.custom.is_empty() {
            children.push(widget::text("No custom shortcuts yet.").size(12).into());
        }
        for (i, c) in self.custom.iter().enumerate() {
            let row = widget::row::with_children(vec![
                widget::text(c.label.clone()).width(Length::Fill).into(),
                widget::text(c.keys.clone()).size(12).into(),
                widget::button::text("✕").on_press(Message::DeleteCustom(i)).into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center);
            children.push(widget::container(row).padding(6).into());
        }

        // Add form.
        children.push(heading_view("Add"));
        // Type toggle: shortcut vs note (hugselapp).
        children.push(
            widget::row::with_children(vec![
                widget::toggler(self.form.is_note)
                    .on_toggle(Message::FormToggleNote)
                    .into(),
                widget::text("Note (hugselapp — no shortcut)").into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        );
        children.push(
            widget::text_input::text_input("Name", &self.form.label)
                .on_input(Message::FormLabel)
                .into(),
        );
        let value_ph = if self.form.is_note {
            "Text"
        } else {
            "Shortcut (e.g. Super + C)"
        };
        children.push(
            widget::text_input::text_input(value_ph, &self.form.keys)
                .on_input(Message::FormKeys)
                .into(),
        );
        if !self.form.is_note {
            children.push(
                widget::text_input::text_input("Command (optional)", &self.form.command)
                    .on_input(Message::FormCommand)
                    .into(),
            );
        }
        children.push(
            widget::text_input::text_input("Section (optional)", &self.form.section)
                .on_input(Message::FormSection)
                .into(),
        );
        children.push(
            widget::button::text("+ Add")
                .on_press(Message::AddCustom)
                .into(),
        );

        let col = widget::column::with_children(children).spacing(6).padding(8);
        widget::scrollable(col).height(Length::Fixed(540.0)).into()
    }
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = bool;
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, windowed: Self::Flags) -> (Self, Task<Message>) {
        let mut window = Window {
            core,
            windowed,
            ..Default::default()
        };
        let task = if windowed {
            // Layer surfaces always try to apply a corner radius, which errors
            // on cosmic_corner_radius_layer_v1 with this compositor. Drop the
            // System auto-corner-radius so no corner-radius request is sent.
            window
                .core
                .set_auto_corner_radius(cosmic::core::Auto::Window | cosmic::core::Auto::Popup);
            // Standalone: show the cheat sheet in a layer surface anchored to
            // the top edge (drops down from the top like the old GTK panel).
            cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(
                cosmic::surface::action::app_layer_shell::<Window>(
                    |_app| cosmic::surface::action::LiveSettings::default(),
                    |_app: &mut Window| {
                        use cosmic::cctk::sctk::shell::wlr_layer::{Anchor, KeyboardInteractivity};
                        cosmic::iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
                            anchor: Anchor::TOP,
                            keyboard_interactivity: KeyboardInteractivity::OnDemand,
                            size: Some((Some(480), Some(600))),
                            namespace: "cheatsheet".to_string(),
                            ..Default::default()
                        }
                    },
                    Some(Box::new(|app: &Window| {
                        // Opaque themed background so the surface isn't see-through.
                        let panel = widget::container(app.body())
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .class(cosmic::theme::Container::custom(|theme| {
                                let cosmic = theme.cosmic();
                                cosmic::widget::container::Style {
                                    background: Some(cosmic::iced::Background::Color(
                                        cosmic.bg_color().into(),
                                    )),
                                    border: cosmic::iced::Border {
                                        radius: 12.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            }));
                        Element::from(panel).map(cosmic::Action::App)
                    })),
                ),
            )))
        } else {
            Task::none()
        };
        (window, task)
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        if self.windowed {
            None
        } else {
            Some(Message::PopupClosed(id))
        }
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
            Message::ToggleWindow => {
                // Open (or toggle) the standalone top-anchored surface, same as
                // the Super+C keybind, so the icon and keybind match.
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).arg("--window").spawn();
                }
            }
            Message::Launch(argv) => {
                spawn(&argv);
                if self.windowed {
                    std::process::exit(0);
                }
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
            Message::OpenEditor => self.mode = Mode::Edit,
            Message::CloseEditor => self.mode = Mode::List,
            Message::FormLabel(s) => self.form.label = s,
            Message::FormKeys(s) => self.form.keys = s,
            Message::FormCommand(s) => self.form.command = s,
            Message::FormSection(s) => self.form.section = s,
            Message::FormToggleNote(b) => self.form.is_note = b,
            Message::AddCustom => {
                let label = self.form.label.trim().to_string();
                let keys = self.form.keys.trim().to_string();
                if !label.is_empty() && !keys.is_empty() {
                    // A note (hugselapp) never has a command.
                    let command = if self.form.is_note {
                        None
                    } else {
                        non_empty(&self.form.command)
                    };
                    let section = non_empty(&self.form.section);
                    self.custom.push(CustomShortcut {
                        label,
                        keys,
                        command,
                        section,
                    });
                    if let Err(e) = config::save(&self.custom) {
                        log::warn!("could not save custom.toml: {e}");
                    }
                    self.form = Form::default();
                }
            }
            Message::DeleteCustom(i) => {
                if i < self.custom.len() {
                    self.custom.remove(i);
                    if let Err(e) = config::save(&self.custom) {
                        log::warn!("could not save custom.toml: {e}");
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Standalone window: show the cheat sheet directly.
        if self.windowed {
            return self.body();
        }
        // Clicking the panel icon opens the same top-anchored surface as Super+C.
        let btn = self
            .core
            .applet
            .icon_button("input-keyboard-symbolic")
            .on_press(Message::ToggleWindow);

        Element::from(self.core.applet.applet_tooltip::<Message>(
            btn,
            "Keyboard shortcuts",
            false,
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
