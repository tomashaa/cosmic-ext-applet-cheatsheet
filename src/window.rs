// SPDX-License-Identifier: GPL-3.0-only
// Copyright (c) 2026 Tomas Haaland

use std::borrow::Cow;

use cosmic::app::{Core, Task};
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::core::window;
use cosmic::iced::window::Id;
use cosmic::iced::Length;
use cosmic::widget;
use cosmic::Element;
use cosmic_settings_config::shortcuts as cosmic_shortcuts;

use crate::config::{self, CustomShortcut};
use crate::i18n;

const ID: &str = "io.github.tomashaa.CosmicExtCheatsheet";
/// Panel width — matches the GTK Super+P cheat sheet.
const PANEL_WIDTH: f32 = 440.0;
/// Slide distance when the panel is fully off-screen to the right.
const SLIDE_OFF: f32 = PANEL_WIDTH;

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
    /// Keys-string of the row currently under the mouse (clickable feedback).
    hovered: Option<String>,
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
    /// Compact overview (merge directions / workspaces, hide media keys).
    compact: bool,
    /// Running as a standalone window (`--window`) rather than a panel applet.
    windowed: bool,
    /// Layer-surface id for a clean close.
    surface_id: Option<Id>,
    /// True from [`Self::open_sheet`] until [`Self::finish_close`] (even before SurfaceReady).
    sheet_open: bool,
    /// Horizontal offset from the right edge (PANEL_WIDTH = hidden, 0 = open).
    slide_x: f32,
    /// Animation target for `slide_x`.
    slide_target: f32,
    /// After slide-out finishes, exit the process.
    slide_closing: bool,
}

impl Default for Window {
    fn default() -> Self {
        let settings = config::load_settings();
        crate::i18n::init(settings.lang.as_deref());
        let state = if settings.remember {
            config::load_state()
        } else {
            config::State::default()
        };
        let lang = crate::i18n::current_lang();
        Self {
            core: Core::default(),
            search: state.search,
            search_id: cosmic::widget::Id::unique(),
            scroll_id: cosmic::widget::Id::unique(),
            shortcuts: crate::shortcuts::load(),
            custom: config::load(),
            lang,
            mode: Mode::default(),
            form: Form::default(),
            selected: 0,
            hovered: None,
            remember: settings.remember,
            scroll: state.scroll,
            restore_scroll: state.scroll,
            learned: config::load_learned(),
            learning: settings.learning,
            compact: settings.compact,
            windowed: false,
            surface_id: None,
            sheet_open: false,
            slide_x: SLIDE_OFF,
            slide_target: SLIDE_OFF,
            slide_closing: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Surface(cosmic::surface::Action),
    SurfaceReady(Id),
    Search(String),
    Launch(Vec<String>),
    /// Mouse entered/left a clickable row (`None` = left).
    HoverRow(Option<String>),
    ToggleWindow,
    /// Poll file IPC from `--window` / Super+C while the panel applet runs.
    IpcPoll,
    Close,
    Ignore,
    Focus,
    NavUp,
    NavDown,
    NavActivate,
    Scrolled(f32),
    ToggleRemember(bool),
    ToggleLearned(String, bool),
    ToggleLearning(bool),
    ToggleCompact(bool),
    /// Language picker index into [`i18n::LANGS`].
    SetLang(usize),
    ShortcutsChanged(cosmic_shortcuts::Config),
    /// Advance the right-edge slide animation one frame.
    AnimTick,
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

/// One cheat-sheet row: label above, shortcut badge below (right-aligned) so
/// long key strings never overlap the description.
fn row_view(
    label: String,
    keys: String,
    icon: Option<&'static str>,
    argv: Option<Vec<String>>,
    selected: bool,
    hovered: bool,
    learned: bool,
    show_checkbox: bool,
) -> Element<'static, Message> {
    let clickable = argv.is_some();
    let id = keys.clone();
    let hover_id = keys.clone();

    // Clickable actions get an accent-coloured label, like the old cheat sheet.
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

    let mut label_row: Vec<Element<Message>> = Vec::new();
    if show_checkbox {
        label_row.push(
            widget::checkbox(learned)
                .on_toggle(move |checked| Message::ToggleLearned(id.clone(), checked))
                .into(),
        );
    }
    if let Some(name) = icon {
        label_row.push(
            widget::icon::from_name(name)
                .size(16)
                .symbolic(true)
                .icon()
                .into(),
        );
    }
    label_row.push(label_widget.width(Length::Fill).into());

    let content = widget::column::with_children(vec![
        widget::row::with_children(label_row)
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        widget::container(badge)
            .width(Length::Fill)
            .align_x(cosmic::iced::alignment::Horizontal::Right)
            .into(),
    ])
    .spacing(4);

    let mut cell = widget::container(content)
        .width(Length::Fill)
        .padding(cosmic::iced::Padding {
            top: 6.0,
            right: 8.0,
            bottom: 6.0,
            left: 8.0,
        });

    // Visual feedback: keyboard selection > mouse hover > idle.
    if selected || (clickable && hovered) {
        let alpha = if selected { 0.22 } else { 0.12 };
        cell = cell.class(cosmic::theme::Container::custom(move |theme| {
            let mut c: cosmic::iced::Color = theme.cosmic().accent_color().into();
            c.a = alpha;
            cosmic::widget::container::Style {
                background: Some(cosmic::iced::Background::Color(c)),
                border: cosmic::iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));
    } else if clickable {
        // Subtle idle surface so clickable rows look tappable even before hover.
        cell = cell.class(cosmic::theme::Container::custom(|theme| {
            let mut c: cosmic::iced::Color = theme.cosmic().primary_container_divider().into();
            c.a = 0.18;
            cosmic::widget::container::Style {
                background: Some(cosmic::iced::Background::Color(c)),
                border: cosmic::iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));
    }

    match argv {
        Some(cmd) => widget::mouse_area(cell)
            .on_press(Message::Launch(cmd))
            .on_enter(Message::HoverRow(Some(hover_id)))
            .on_exit(Message::HoverRow(None))
            .interaction(cosmic::iced::mouse::Interaction::Pointer)
            .into(),
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

    fn animating(&self) -> bool {
        (self.slide_x - self.slide_target).abs() > 0.5
    }

    /// Start slide-out; surface is destroyed (and standalone process exits) after.
    fn begin_close(&mut self) -> Task<Message> {
        if !self.sheet_open && self.surface_id.is_none() {
            // Nothing to close — but standalone must still be able to exit.
            if self.windowed {
                return cosmic::iced::exit::<cosmic::Action<Message>>();
            }
            return Task::none();
        }
        self.sheet_open = true; // keep true until finish_close
        self.slide_closing = true;
        self.slide_target = SLIDE_OFF;
        // If we never got SurfaceReady, skip the animation and tear down now.
        if self.surface_id.is_none() {
            return self.finish_close();
        }
        Task::none()
    }

    /// Destroy the layer surface. Standalone `--window` exits; the panel applet stays.
    fn finish_close(&mut self) -> Task<Message> {
        let mut tasks = Vec::new();
        if let Some(id) = self.surface_id.take() {
            tasks.push(cosmic::task::message(cosmic::Action::Cosmic(
                cosmic::app::Action::Surface(cosmic::surface::action::destroy_layer_shell(id)),
            )));
        }
        self.sheet_open = false;
        self.slide_closing = false;
        self.slide_x = SLIDE_OFF;
        self.slide_target = SLIDE_OFF;
        crate::ipc::release_open_marker();
        if self.windowed {
            tasks.push(cosmic::iced::exit::<cosmic::Action<Message>>());
        }
        Task::batch(tasks)
    }

    fn sheet_is_open(&self) -> bool {
        self.sheet_open || self.surface_id.is_some()
    }

    /// Global open/close (panel button). Close anything visible system-wide;
    /// only open here when nothing else owns the sheet.
    fn toggle_sheet(&mut self) -> Task<Message> {
        if self.sheet_is_open() {
            if self.slide_closing {
                Task::none()
            } else {
                self.begin_close()
            }
        } else if crate::ipc::anything_open() {
            // Another instance (other output / orphan --window) is showing it.
            crate::ipc::close_everything();
            Task::none()
        } else {
            self.open_sheet()
        }
    }

    /// Full-screen layer: panel docks on the right; click outside / Esc closes.
    /// Exclusive keyboard so Esc always reaches us (search field won't swallow it).
    fn open_sheet(&mut self) -> Task<Message> {
        // Single-open guard across all applet/--window processes.
        if !crate::ipc::claim_open_marker() {
            log::warn!("cheatsheet already open in another process; not opening a second sheet");
            return Task::none();
        }
        self.sheet_open = true;
        self.slide_x = SLIDE_OFF;
        self.slide_target = 0.0;
        self.slide_closing = false;
        let surface = cosmic::task::message(cosmic::Action::Cosmic(cosmic::app::Action::Surface(
            cosmic::surface::action::app_layer_shell::<Window>(
                |_app| cosmic::surface::action::LiveSettings::default(),
                |_app: &mut Window| {
                    use cosmic::cctk::sctk::shell::wlr_layer::{Anchor, KeyboardInteractivity};
                    cosmic::iced::platform_specific::runtime::wayland::layer_surface::SctkLayerSurfaceSettings {
                        anchor: Anchor::TOP
                            .union(Anchor::BOTTOM)
                            .union(Anchor::LEFT)
                            .union(Anchor::RIGHT),
                        keyboard_interactivity: KeyboardInteractivity::Exclusive,
                        size: None,
                        exclusive_zone: 0,
                        namespace: "cheatsheet".to_string(),
                        ..Default::default()
                    }
                },
                Some(Box::new(|app: &Window| {
                    let panel = widget::container(app.body())
                        .width(Length::Fixed(PANEL_WIDTH))
                        .height(Length::Fill)
                        .class(cosmic::theme::Container::custom(|theme| {
                            let cosmic = theme.cosmic();
                            cosmic::widget::container::Style {
                                background: Some(cosmic::iced::Background::Color(
                                    cosmic.bg_color().into(),
                                )),
                                border: cosmic::iced::Border {
                                    radius: cosmic::iced::border::Radius {
                                        top_left: 12.0,
                                        top_right: 0.0,
                                        bottom_right: 0.0,
                                        bottom_left: 12.0,
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        }));
                    let panel = widget::mouse_area(panel).on_press(Message::Ignore);

                    let dock = widget::container(
                        widget::row::with_children(vec![
                            widget::Space::new()
                                .width(Length::Fixed(app.slide_x.max(0.0)))
                                .into(),
                            panel.into(),
                        ])
                        .height(Length::Fill),
                    )
                    .width(Length::Fixed(PANEL_WIDTH))
                    .height(Length::Fill)
                    .clip(true);

                    // Dim/empty left side: click closes (same as GTK Super+P).
                    let strip = widget::row::with_children(vec![
                        widget::mouse_area(
                            widget::container(widget::Space::new())
                                .width(Length::Fill)
                                .height(Length::Fill),
                        )
                        .on_press(Message::Close)
                        .into(),
                        dock.into(),
                    ])
                    .width(Length::Fill)
                    .height(Length::Fill);

                    Element::from(strip).map(cosmic::Action::App)
                })),
            ),
        )));
        Task::batch([
            surface,
            cosmic::widget::text_input::focus(self.search_id.clone()),
        ])
    }

    fn persist_settings(&self) {
        config::save_settings(&config::Settings {
            remember: self.remember,
            learning: self.learning,
            compact: self.compact,
            lang: Some(self.lang.to_string()),
        });
    }

    /// Full Cosmic list, optionally collapsed for compact overview.
    fn visible_shortcuts(&self) -> Vec<crate::shortcuts::Shortcut> {
        crate::shortcuts::for_display(&self.shortcuts, self.compact)
    }

    /// Visible clickable rows' commands, in render order (ACTIONS first, then
    /// custom shortcuts that have a command, grouped by section). The keyboard
    /// selection (`self.selected`) indexes into this.
    fn nav_commands(&self) -> Vec<Vec<String>> {
        let q = self.search.to_lowercase();
        let mut out: Vec<Vec<String>> = Vec::new();
        let visible = self.visible_shortcuts();
        for sec_key in crate::shortcuts::SECTION_ORDER {
            for s in &visible {
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

        // Title + hint (like the GTK sheet), then search + settings.
        children.push(
            widget::text(i18n::tr("ui.title"))
                .size(17)
                .class(cosmic::theme::Text::Accent)
                .into(),
        );
        children.push(
            widget::text(i18n::tr("ui.sub"))
                .size(11)
                .into(),
        );
        let mut header: Vec<Element<Message>> = vec![
            widget::text_input::search_input(i18n::tr("ui.search"), &self.search)
                .on_input(Message::Search)
                .id(self.search_id.clone())
                .width(Length::Fill)
                .into(),
        ];
        let compact_label = if self.compact {
            i18n::tr("ui.show_all")
        } else {
            i18n::tr("ui.compact_btn")
        };
        header.push(
            widget::button::standard(compact_label)
                .on_press(Message::ToggleCompact(!self.compact))
                .into(),
        );
        header.push(
            widget::button::icon(widget::icon::from_name("preferences-system-symbolic"))
                .on_press(Message::OpenEditor)
                .into(),
        );
        children.push(
            widget::row::with_children(header)
                .spacing(8)
                .align_y(cosmic::iced::Alignment::Center)
                .into(),
        );

        let visible = self.visible_shortcuts();

        // Actual COSMIC shortcuts, grouped by section. Clickable rows (Spawn
        // bindings) launch on click/Enter; the rest are reference.
        let mut ci = 0usize; // clickable-row index, matches nav_commands()
        for sec_key in crate::shortcuts::SECTION_ORDER {
            let mut rows: Vec<Element<Message>> = Vec::new();
            for s in &visible {
                if s.section != *sec_key || !matches(&q, &s.label, &s.keys) {
                    continue;
                }
                let is_learned = self.learned.contains(&s.keys);
                if is_learned && !self.learning {
                    continue;
                }
                let clickable = s.command.is_some();
                let sel = clickable && self.selected == ci;
                let hovered = clickable && self.hovered.as_deref() == Some(s.keys.as_str());
                rows.push(row_view(
                    s.label.clone(),
                    s.keys.clone(),
                    s.icon,
                    s.command.clone(),
                    sel,
                    hovered,
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
            let title = i18n::tr(sec_key);
            let heading = if title.is_empty() || title == *sec_key {
                (*sec_key).to_string()
            } else {
                title
            };
            children.push(heading_view(&heading));
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
                let hovered = clickable && self.hovered.as_deref() == Some(c.keys.as_str());
                rows.push(row_view(
                    c.label.clone(),
                    c.keys.clone(),
                    Some("preferences-other-symbolic"),
                    arg,
                    clickable && self.selected == ci,
                    hovered,
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
        // Full panel height — right-edge sheet fills the screen vertically.
        widget::scrollable(col)
            .id(self.scroll_id.clone())
            .on_scroll(|vp| Message::Scrolled(vp.relative_offset().y))
            .height(Length::Fill)
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
                widget::text(i18n::tr("ui.editor_title"))
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
                widget::text(i18n::tr("ui.remember")).into(),
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
                widget::text(i18n::tr("ui.learning")).into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        );

        // Setting: compact overview (default) vs full Cosmic dump.
        children.push(
            widget::row::with_children(vec![
                widget::toggler(self.compact)
                    .on_toggle(Message::ToggleCompact)
                    .into(),
                widget::text(i18n::tr("ui.compact")).into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        );

        // Language: same list / file as the GTK Super+P sheet.
        children.push(
            widget::row::with_children(vec![
                widget::text(i18n::tr("ui.lang"))
                    .width(Length::Fill)
                    .into(),
                widget::dropdown(
                    i18n::LANG_NAMES,
                    i18n::lang_index(self.lang),
                    Message::SetLang,
                )
                .into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        );

        // Existing custom shortcuts, each with a delete button.
        if self.custom.is_empty() {
            children.push(
                widget::text(i18n::tr("ui.custom_empty"))
                    .size(12)
                    .into(),
            );
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
        children.push(heading_view(&i18n::tr("ui.add")));
        // Type toggle: shortcut vs note (hugselapp).
        children.push(
            widget::row::with_children(vec![
                widget::toggler(self.form.is_note)
                    .on_toggle(Message::FormToggleNote)
                    .into(),
                widget::text(i18n::tr("ui.note_toggle")).into(),
            ])
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center)
            .into(),
        );
        children.push(
            widget::text_input::text_input(i18n::tr("ui.name_ph"), &self.form.label)
                .on_input(Message::FormLabel)
                .into(),
        );
        let value_ph = if self.form.is_note {
            i18n::tr("ui.text_ph")
        } else {
            i18n::tr("ui.keys_ph")
        };
        children.push(
            widget::text_input::text_input(value_ph, &self.form.keys)
                .on_input(Message::FormKeys)
                .into(),
        );
        if !self.form.is_note {
            children.push(
                widget::text_input::text_input(i18n::tr("ui.command_ph"), &self.form.command)
                    .on_input(Message::FormCommand)
                    .into(),
            );
        }
        children.push(
            widget::text_input::text_input(i18n::tr("ui.section_ph"), &self.form.section)
                .on_input(Message::FormSection)
                .into(),
        );
        children.push(
            widget::button::text(format!("+ {}", i18n::tr("ui.add")))
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
        if !windowed {
            crate::ipc::register_applet();
        }
        // Standalone `--window`: open immediately. Panel applet opens on demand.
        let task = if windowed {
            window.open_sheet()
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
                    if self.sheet_is_open() {
                        return self.begin_close();
                    }
                }
            }
            Message::AnimTick => {
                let speed = 48.0; // px per frame ≈ snappy ease toward target
                let delta = self.slide_target - self.slide_x;
                if delta.abs() <= speed {
                    self.slide_x = self.slide_target;
                } else {
                    self.slide_x += delta.signum() * speed;
                }
                if self.slide_closing && (self.slide_x - SLIDE_OFF).abs() < 0.5 {
                    return self.finish_close();
                }
            }
            Message::Scrolled(y) => {
                self.scroll = y;
                self.persist_state();
            }
            Message::ToggleRemember(b) => {
                self.remember = b;
                self.persist_settings();
            }
            Message::ToggleLearned(id, checked) => {
                if checked {
                    self.learned.insert(id);
                } else {
                    self.learned.remove(&id);
                }
                config::save_learned(&self.learned);
                self.selected = 0;
            }
            Message::ShortcutsChanged(_) => {
                self.shortcuts = crate::shortcuts::load();
                self.selected = 0;
            }
            Message::SurfaceReady(id) => {
                self.surface_id = Some(id);
                // Kick the slide-in from the right.
                self.slide_x = SLIDE_OFF;
                self.slide_target = 0.0;
                self.slide_closing = false;
                // Same as Message::Focus: autofocus search (+ restore scroll).
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
            Message::ToggleLearning(b) => {
                self.learning = b;
                self.persist_settings();
                self.selected = 0;
            }
            Message::ToggleCompact(b) => {
                self.compact = b;
                self.persist_settings();
                self.selected = 0;
            }
            Message::SetLang(i) => {
                if let Some(code) = i18n::LANGS.get(i).copied() {
                    if code != self.lang && i18n::set_lang(code) {
                        self.lang = i18n::current_lang();
                        self.persist_settings();
                        self.shortcuts = crate::shortcuts::load();
                        self.selected = 0;
                    }
                }
            }
            Message::IpcPoll => {
                // Sticky close: every instance with a sheet must see this.
                if crate::ipc::close_requested()
                    && self.sheet_is_open()
                    && !self.slide_closing
                {
                    return self.begin_close();
                }
                // One-shot open: claim_open_marker prevents a second sheet.
                if crate::ipc::take_open_request() && !self.sheet_is_open() {
                    return self.open_sheet();
                }
            }
            Message::ToggleWindow => {
                return self.toggle_sheet();
            }
            Message::Close => {
                if self.sheet_is_open() || self.windowed {
                    return self.begin_close();
                }
            }
            Message::Ignore => {}
            Message::HoverRow(id) => {
                self.hovered = id;
            }
            Message::Launch(argv) => {
                spawn(&argv);
                if self.sheet_is_open() {
                    return self.begin_close();
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
            i18n::tr("ui.tooltip"),
            false,
            Message::Surface,
            None,
        ))
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        let shortcuts_watch = cosmic::cosmic_config::config_subscription(
            0u64,
            Cow::Borrowed(cosmic_shortcuts::ID),
            cosmic_shortcuts::Config::VERSION,
        )
        .map(|update: cosmic::cosmic_config::Update<cosmic_shortcuts::Config>| {
            Message::ShortcutsChanged(update.config)
        });

        // Always listen while a sheet may be open (incl. before SurfaceReady).
        let keys = if self.sheet_is_open() || self.windowed {
            cosmic::iced::event::listen_with(|event, _status, id| {
                use cosmic::iced::keyboard::{key::Named, Event as KeyEvent, Key};
                match event {
                    cosmic::iced::Event::Keyboard(KeyEvent::KeyPressed {
                        key: Key::Named(named),
                        ..
                    }) => match named {
                        // Handle Esc even if the search field captured the event.
                        Named::Escape => Some(Message::Close),
                        Named::ArrowDown => Some(Message::NavDown),
                        Named::ArrowUp => Some(Message::NavUp),
                        Named::Enter => Some(Message::NavActivate),
                        _ => None,
                    },
                    cosmic::iced::Event::Window(window::Event::Opened { .. }) => {
                        Some(Message::SurfaceReady(id))
                    }
                    _ => None,
                }
            })
        } else {
            cosmic::iced::Subscription::none()
        };

        let anim = if self.animating() || self.slide_closing {
            cosmic::iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| Message::AnimTick)
        } else {
            cosmic::iced::Subscription::none()
        };

        let ipc = if !self.windowed {
            cosmic::iced::time::every(std::time::Duration::from_millis(150))
                .map(|_| Message::IpcPoll)
        } else {
            cosmic::iced::Subscription::none()
        };

        cosmic::iced::Subscription::batch([shortcuts_watch, keys, anim, ipc])
    }

    fn view_window(&self, _id: Id) -> Element<'_, Message> {
        "".into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
