// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland

use cosmic::app::{Core, Task};
use cosmic::iced::core::window;
use cosmic::iced::window::Id;
use cosmic::iced::Length;
use cosmic::widget;
use cosmic::Element;

use crate::config::{self, CustomShortcut};
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
    search: String,
    search_id: cosmic::widget::Id,
    scroll_id: cosmic::widget::Id,
    /// Actual COSMIC shortcuts read from config (defaults + user custom).
    shortcuts: Vec<crate::shortcuts::Shortcut>,
    custom: Vec<CustomShortcut>,
    lang: &'static str,
    mode: Mode,
    form: Form,
    /// Index of the keyboard-selected clickable row (into `nav_commands()`).
    selected: usize,
    /// Remember the last search + scroll across opens.
    remember: bool,
    /// Current scroll offset (relative y), tracked for persistence.
    scroll: f32,
    /// Scroll offset to restore once the surface opens.
    restore_scroll: f32,
    /// Shortcut ids (the keys string) the user has marked as learned.
    learned: std::collections::HashSet<String>,
    /// Learning mode: show per-row checkboxes + reveal learned shortcuts.
    learning: bool,
    /// Running as a standalone window (`--window`) rather than a panel applet.
    windowed: bool,
}

impl Default for Window {
    fn default() -> Self {
        let settings = config::load_settings();
        let state = if settings.remember {
            config::load_state()
        } else {
            config::State::default()
        };
        Self {
            core: Core::default(),
            search: state.search,
            search_id: cosmic::widget::Id::unique(),
            scroll_id: cosmic::widget::Id::unique(),
            shortcuts: crate::shortcuts::load(),
            custom: config::load(),
            lang: crate::i18n::current_lang(),
            mode: Mode::default(),
            form: Form::default(),
            selected: 0,
            remember: settings.remember,
            scroll: state.scroll,
            restore_scroll: state.scroll,
            learned: config::load_learned(),
            learning: settings.learning,
            windowed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Surface(cosmic::surface::Action),
    Search(String),
    Launch(Vec<String>),
    ToggleWindow,
    Close,
    Ignore,
    Focus,
    NavUp,
    NavDown,
    NavActivate,
    Scrolled(f32),
    ToggleRemember(bool),
    ToggleLearned(String),
    ToggleLearning(bool),
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
    selected: bool,
    learned: bool,
    show_checkbox: bool,
) -> Element<'static, Message> {
    let clickable = argv.is_some();
    let id = keys.clone();

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

    let mut row_children: Vec<Element<Message>> = Vec::new();
    if show_checkbox {
        row_children.push(
            widget::button::text(if learned { "☑" } else { "☐" })
                .on_press(Message::ToggleLearned(id))
                .into(),
        );
    }
    row_children.push(label_widget.width(Length::Fill).into());
    row_children.push(badge.into());
    let content = widget::row::with_children(row_children)
        .spacing(12)
        .align_y(cosmic::iced::Alignment::Center);

    let mut cell = widget::container(content)
        .width(Length::Fill)
        .padding(cosmic::iced::Padding {
            top: 6.0,
            right: 8.0,
            bottom: 6.0,
            left: 8.0,
        });
    if selected {
        // Keyboard-highlighted row: accent-tinted background.
        cell = cell.class(cosmic::theme::Container::custom(|theme| {
            let mut c: cosmic::iced::Color = theme.cosmic().accent_color().into();
            c.a = 0.20;
            cosmic::widget::container::Style {
                background: Some(cosmic::iced::Background::Color(c)),
                border: cosmic::iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));
    }

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

    /// Visible clickable rows' commands, in render order (ACTIONS first, then
    /// custom shortcuts that have a command, grouped by section). The keyboard
    /// selection (`self.selected`) indexes into this.
    fn nav_commands(&self) -> Vec<Vec<String>> {
        let q = self.search.to_lowercase();
        let mut out: Vec<Vec<String>> = Vec::new();
        for (sec_key, _) in crate::shortcuts::SECTION_ORDER {
            for s in &self.shortcuts {
                if s.section != *sec_key || !matches(&q, &s.label, &s.keys) {
                    continue;
                }
                if self.learned.contains(&s.keys) && !self.learning {
                    continue;
                }
                if let Some(cmd) = &s.command {
                    out.push(cmd.clone());
                }
            }
        }
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
            for c in custom.iter().filter(|c| c.section_or_default() == sec) {
                if self.learned.contains(&c.keys) && !self.learning {
                    continue;
                }
                let argv = c.argv();
                if !argv.is_empty() {
                    out.push(argv);
                }
            }
        }
        out
    }

    fn list_body(&self) -> Element<'_, Message> {
        let q = self.search.to_lowercase();
        let mut children: Vec<Element<Message>> = Vec::new();

        // Search field + (optional) show-learned toggle + settings (gear).
        let mut header: Vec<Element<Message>> = vec![
            widget::text_input::search_input(i18n::tr(self.lang, "ui.search"), &self.search)
                .on_input(Message::Search)
                .id(self.search_id.clone())
                .width(Length::Fill)
                .into(),
        ];
        header.push(widget::button::text("⚙").on_press(Message::OpenEditor).into());
        children.push(
            widget::row::with_children(header)
                .spacing(8)
                .align_y(cosmic::iced::Alignment::Center)
                .into(),
        );

        // Actual COSMIC shortcuts, grouped by section. Clickable rows (Spawn
        // bindings) launch on click/Enter; the rest are reference.
        let mut ci = 0usize; // clickable-row index, matches nav_commands()
        for (sec_key, sec_title) in crate::shortcuts::SECTION_ORDER {
            let mut rows: Vec<Element<Message>> = Vec::new();
            for s in &self.shortcuts {
                if s.section != *sec_key || !matches(&q, &s.label, &s.keys) {
                    continue;
                }
                let is_learned = self.learned.contains(&s.keys);
                if is_learned && !self.learning {
                    continue;
                }
                let clickable = s.command.is_some();
                let sel = clickable && self.selected == ci;
                rows.push(row_view(
                    s.label.clone(),
                    s.keys.clone(),
                    s.command.clone(),
                    sel,
                    is_learned,
                    self.learning,
                ));
                if clickable {
                    ci += 1;
                }
            }
            if rows.is_empty() {
                continue;
            }
            children.push(heading_view(sec_title));
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
            let mut rows: Vec<Element<Message>> = Vec::new();
            for c in custom.iter().filter(|c| c.section_or_default() == sec) {
                let is_learned = self.learned.contains(&c.keys);
                if is_learned && !self.learning {
                    continue;
                }
                let argv = c.argv();
                let clickable = !argv.is_empty();
                let arg = if clickable { Some(argv) } else { None };
                rows.push(row_view(
                    c.label.clone(),
                    c.keys.clone(),
                    arg,
                    clickable && self.selected == ci,
                    is_learned,
                    self.learning,
                ));
                if clickable {
                    ci += 1;
                }
            }
            if rows.is_empty() {
                continue;
            }
            children.push(heading_view(sec));
            children.extend(rows);
        }

        let col = widget::column::with_children(children).spacing(2).padding(8);
        widget::scrollable(col)
            .id(self.scroll_id.clone())
            .on_scroll(|vp| Message::Scrolled(vp.relative_offset().y))
            .height(Length::Fixed(540.0))
            .into()
    }

    /// Scroll the list so the keyboard-selected row stays roughly in view.
    fn scroll_to_selected(&self) -> Task<Message> {
        let n = self.nav_commands().len();
        let y = if n > 1 {
            self.selected as f32 / (n - 1) as f32
        } else {
            0.0
        };
        cosmic::iced::widget::scrollable::snap_to(
            self.scroll_id.clone(),
            cosmic::iced::widget::scrollable::RelativeOffset { x: None, y: Some(y) },
        )
    }

    /// Persist the current search + scroll if "remember" is on.
    fn persist_state(&self) {
        if self.remember {
            config::save_state(&config::State {
                search: self.search.clone(),
                scroll: self.scroll,
            });
        }
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

        // Setting: remember the last search + scroll across opens.
        children.push(
            widget::row::with_children(vec![
                widget::toggler(self.remember)
                    .on_toggle(Message::ToggleRemember)
                    .into(),
                widget::text("Remember last search & scroll").into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        );

        // Setting: learning mode (per-row checkboxes to mark shortcuts learned).
        children.push(
            widget::row::with_children(vec![
                widget::toggler(self.learning)
                    .on_toggle(Message::ToggleLearning)
                    .into(),
                widget::text("Learning mode (checkboxes to hide learned)").into(),
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
        // Disable auto corner-radius entirely so we never send a
        // cosmic_corner_radius_*_v1 request — that protocol mismatches across
        // compositor versions and killed the surface. We round the content
        // container ourselves instead. Applies to both applet and window modes.
        window.core.set_auto_corner_radius(Default::default());
        let task = if windowed {
            // Standalone: show the cheat sheet in a layer surface anchored to
            // the top edge (drops down from the top like the old GTK panel).
            let surface = cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(
                cosmic::surface::action::app_layer_shell::<Window>(
                    |_app| cosmic::surface::action::LiveSettings::default(),
                    |_app: &mut Window| {
                        use cosmic::cctk::sctk::shell::wlr_layer::{Anchor, KeyboardInteractivity};
                        // Full-screen modal: fills the screen (transparent except the
                        // panel) so clicks outside dismiss and Esc is captured.
                        cosmic::iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
                            anchor: Anchor::TOP
                                .union(Anchor::BOTTOM)
                                .union(Anchor::LEFT)
                                .union(Anchor::RIGHT),
                            keyboard_interactivity: KeyboardInteractivity::Exclusive,
                            size: None,
                            namespace: "cheatsheet".to_string(),
                            ..Default::default()
                        }
                    },
                    Some(Box::new(|app: &Window| {
                        // The panel: opaque, top corners flush (it meets the header),
                        // bottom corners rounded.
                        let panel = widget::container(app.body())
                            .width(Length::Fixed(480.0))
                            .class(cosmic::theme::Container::custom(|theme| {
                                let cosmic = theme.cosmic();
                                cosmic::widget::container::Style {
                                    background: Some(cosmic::iced::Background::Color(
                                        cosmic.bg_color().into(),
                                    )),
                                    border: cosmic::iced::Border {
                                        radius: cosmic::iced::border::Radius {
                                            top_left: 0.0,
                                            top_right: 0.0,
                                            bottom_right: 12.0,
                                            bottom_left: 12.0,
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            }));
                        // Clicking the panel itself must not dismiss.
                        let panel = widget::mouse_area(panel).on_press(Message::Ignore);
                        // Panel at top-centre; the rest of the screen dismisses on click.
                        let screen = widget::container(panel)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .align_x(cosmic::iced::alignment::Horizontal::Center);
                        let dismiss = widget::mouse_area(screen).on_press(Message::Close);
                        Element::from(dismiss).map(cosmic::Action::App)
                    })),
                ),
            )));
            // Focus the search field so typing filters immediately.
            Task::batch([
                surface,
                cosmic::widget::text_input::focus(window.search_id.clone()),
            ])
        } else {
            Task::none()
        };
        (window, task)
    }

    fn on_close_requested(&self, _id: window::Id) -> Option<Message> {
        None
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Search(q) => {
                self.search = q;
                self.selected = 0;
                self.persist_state();
            }
            Message::Focus => {
                let focus = cosmic::widget::text_input::focus(self.search_id.clone());
                if self.remember && self.restore_scroll > 0.0 {
                    let scroll = cosmic::iced::widget::scrollable::snap_to(
                        self.scroll_id.clone(),
                        cosmic::iced::widget::scrollable::RelativeOffset {
                            x: None,
                            y: Some(self.restore_scroll),
                        },
                    );
                    return Task::batch([focus, scroll]);
                }
                return focus;
            }
            Message::NavDown => {
                let n = self.nav_commands().len();
                if n > 0 {
                    self.selected = (self.selected + 1).min(n - 1);
                }
                return self.scroll_to_selected();
            }
            Message::NavUp => {
                self.selected = self.selected.saturating_sub(1);
                return self.scroll_to_selected();
            }
            Message::NavActivate => {
                if let Some(argv) = self.nav_commands().get(self.selected).cloned() {
                    spawn(&argv);
                    if self.windowed {
                        std::process::exit(0);
                    }
                }
            }
            Message::Scrolled(y) => {
                self.scroll = y;
                self.persist_state();
            }
            Message::ToggleRemember(b) => {
                self.remember = b;
                config::save_settings(&config::Settings {
                    remember: b,
                    learning: self.learning,
                });
            }
            Message::ToggleLearned(id) => {
                if !self.learned.remove(&id) {
                    self.learned.insert(id);
                }
                config::save_learned(&self.learned);
                self.selected = 0;
            }
            Message::ToggleLearning(b) => {
                self.learning = b;
                config::save_settings(&config::Settings {
                    remember: self.remember,
                    learning: b,
                });
                self.selected = 0;
            }
            Message::ToggleWindow => {
                // Open (or toggle) the standalone top-anchored surface, same as
                // the Super+C keybind, so the icon and keybind match.
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).arg("--window").spawn();
                }
            }
            Message::Close => {
                if self.windowed {
                    std::process::exit(0);
                }
            }
            Message::Ignore => {}
            Message::Launch(argv) => {
                spawn(&argv);
                if self.windowed {
                    std::process::exit(0);
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

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        if !self.windowed {
            return cosmic::iced::Subscription::none();
        }
        cosmic::iced::event::listen_with(|event, _status, _id| {
            use cosmic::iced::keyboard::{key::Named, Event as KeyEvent, Key};
            match event {
                cosmic::iced::Event::Keyboard(KeyEvent::KeyPressed {
                    key: Key::Named(named),
                    ..
                }) => match named {
                    Named::Escape => Some(Message::Close),
                    Named::ArrowDown => Some(Message::NavDown),
                    Named::ArrowUp => Some(Message::NavUp),
                    Named::Enter => Some(Message::NavActivate),
                    _ => None,
                },
                // Focus the search field once the surface is up.
                cosmic::iced::Event::Window(window::Event::Opened { .. }) => Some(Message::Focus),
                _ => None,
            }
        })
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        "".into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
