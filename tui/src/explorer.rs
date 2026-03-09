use anyhow::{Context, Result};
use cheugy_core::pipeline::read_jsonl;
use cheugy_core::schema::{Entity, Evidence, Observation, Relic, Relation};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const SECTION_ORDER: [&str; 5] = ["Relics", "Entities", "Relations", "Observations", "Evidence"];

pub fn run(root: &Path) -> Result<()> {
    let data = ArtifactData::load(root);
    let mut app = App::new(data, root.to_path_buf());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw_ui(f, &app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.search_mode {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => app.search_mode = false,
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.clamp_item_idx();
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.clamp_item_idx();
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') | KeyCode::Left => app.focus = FocusPane::Sections,
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => app.focus = FocusPane::Items,
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.prev(),
                    KeyCode::Char('g') => app.first(),
                    KeyCode::Char('G') => app.last(),
                    KeyCode::Char('/') => {
                        app.search_query.clear();
                        app.search_mode = true;
                        app.focus = FocusPane::Items;
                        app.clamp_item_idx();
                    }
                    KeyCode::Char('o') => {
                        let message = if let Some(target) = app.current_open_target() {
                            suspend_tui_for_open(&mut terminal, &target)
                                .unwrap_or_else(|e| format!("open failed: {e}"))
                        } else {
                            "No openable target for selected artifact".to_string()
                        };
                        app.status = message;
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn suspend_tui_for_open(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    target: &OpenTarget,
) -> Result<String> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let open_result = open_in_editor(target);

    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;

    open_result
}

fn open_in_editor(target: &OpenTarget) -> Result<String> {
    let editor_cfg = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let mut parts = editor_cfg.split_whitespace();
    let editor_cmd = parts
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("vim")
        .to_string();
    let mut args: Vec<String> = parts.map(ToString::to_string).collect();

    let path = target.path.to_string_lossy().to_string();
    let line = target.line.unwrap_or(1);

    match editor_cmd.as_str() {
        "code" | "cursor" => {
            args.push("-g".to_string());
            args.push(format!("{path}:{line}"));
        }
        "hx" => args.push(format!("{path}:{line}")),
        "vim" | "nvim" | "vi" => {
            args.push(format!("+{line}"));
            args.push(path);
        }
        _ => args.push(path),
    }

    let status = Command::new(&editor_cmd)
        .args(&args)
        .status()
        .with_context(|| format!("failed to launch editor '{editor_cmd}'"))?;

    if status.success() {
        Ok(format!(
            "Opened {}{}",
            target.path.display(),
            target
                .line
                .map(|l| format!(":{l}"))
                .unwrap_or_default()
        ))
    } else {
        Ok(format!("Editor exited with status: {status}"))
    }
}

#[derive(Clone)]
struct ArtifactData {
    relics: Vec<Relic>,
    entities: Vec<Entity>,
    relations: Vec<Relation>,
    observations: Vec<Observation>,
    evidence: Vec<Evidence>,
}

impl ArtifactData {
    fn load(root: &Path) -> Self {
        Self {
            relics: read_jsonl::<Relic>(&root.join(".cheugy/relics.jsonl")).unwrap_or_default(),
            entities: read_jsonl::<Entity>(&root.join(".cheugy/entities.jsonl")).unwrap_or_default(),
            relations: read_jsonl::<Relation>(&root.join(".cheugy/relations.jsonl")).unwrap_or_default(),
            observations: read_jsonl::<Observation>(&root.join(".cheugy/observations.jsonl")).unwrap_or_default(),
            evidence: read_jsonl::<Evidence>(&root.join(".cheugy/evidence.jsonl")).unwrap_or_default(),
        }
    }

    fn section_count(&self, section: &str) -> usize {
        match section {
            "Relics" => self.relics.len(),
            "Entities" => self.entities.len(),
            "Relations" => self.relations.len(),
            "Observations" => self.observations.len(),
            "Evidence" => self.evidence.len(),
            _ => 0,
        }
    }

    fn filtered_items(&self, section: &str, query: &str) -> Vec<ItemEntry> {
        let query = query.to_lowercase();
        let mut out = match section {
            "Relics" => self
                .relics
                .iter()
                .enumerate()
                .map(|(idx, r)| ItemEntry {
                    label: format!("{} ({})", r.label, r.paths.len()),
                    item: ItemRef::Relic(idx),
                })
                .collect(),
            "Entities" => self
                .entities
                .iter()
                .enumerate()
                .map(|(idx, e)| ItemEntry {
                    label: format!("{}: {}", e.entity_type, e.canonical_name),
                    item: ItemRef::Entity(idx),
                })
                .collect(),
            "Relations" => self
                .relations
                .iter()
                .enumerate()
                .map(|(idx, r)| ItemEntry {
                    label: format!("{} -> {} ({})", r.src_entity, r.dst_entity, r.relation_type),
                    item: ItemRef::Relation(idx),
                })
                .collect(),
            "Observations" => self
                .observations
                .iter()
                .enumerate()
                .map(|(idx, o)| ItemEntry {
                    label: format!("{}: {}", o.kind, o.canonical_name),
                    item: ItemRef::Observation(idx),
                })
                .collect(),
            "Evidence" => self
                .evidence
                .iter()
                .enumerate()
                .map(|(idx, e)| ItemEntry {
                    label: format!("{}:{} [{}]", e.path, e.line, e.extractor),
                    item: ItemRef::Evidence(idx),
                })
                .collect(),
            _ => Vec::new(),
        };

        if !query.is_empty() {
            out.retain(|row| row.label.to_lowercase().contains(&query));
        }

        out
    }

    fn preview_for(&self, item: ItemRef) -> String {
        match item {
            ItemRef::Relic(index) => self
                .relics
                .get(index)
                .map(|r| {
                    let mut lines = vec![
                        format!("Label: {}", r.label),
                        format!("Theme: {}", r.theme),
                        format!("Distinguishing feature: {}", r.distinguishing_feature),
                        String::new(),
                        "Paths:".to_string(),
                    ];
                    for p in r.paths.iter().take(40) {
                        lines.push(format!("- {p}"));
                    }
                    lines.join("\n")
                })
                .unwrap_or_else(|| "No relic selected".to_string()),
            ItemRef::Entity(index) => self
                .entities
                .get(index)
                .map(|e| {
                    format!(
                        "ID: {}\nType: {}\nName: {}\nObservations: {}",
                        e.id,
                        e.entity_type,
                        e.canonical_name,
                        e.observations.join(", ")
                    )
                })
                .unwrap_or_else(|| "No entity selected".to_string()),
            ItemRef::Relation(index) => self
                .relations
                .get(index)
                .map(|r| {
                    format!(
                        "Relation type: {}\nSource: {}\nTarget: {}",
                        r.relation_type, r.src_entity, r.dst_entity
                    )
                })
                .unwrap_or_else(|| "No relation selected".to_string()),
            ItemRef::Observation(index) => self
                .observations
                .get(index)
                .map(|o| {
                    format!(
                        "ID: {}\nKind: {}\nCanonical: {}\nPath: {}",
                        o.id, o.kind, o.canonical_name, o.path
                    )
                })
                .unwrap_or_else(|| "No observation selected".to_string()),
            ItemRef::Evidence(index) => self
                .evidence
                .get(index)
                .map(|e| {
                    let mut capture_lines: Vec<String> = e
                        .captures
                        .iter()
                        .map(|(k, v)| format!("- {k}: {v}"))
                        .collect();
                    capture_lines.sort();
                    format!(
                        "ID: {}\nExtractor: {}\nLocation: {}:{}\n\nRaw:\n{}\n\nCaptures:\n{}",
                        e.id,
                        e.extractor,
                        e.path,
                        e.line,
                        e.raw.trim_end(),
                        if capture_lines.is_empty() {
                            "(none)".to_string()
                        } else {
                            capture_lines.join("\n")
                        }
                    )
                })
                .unwrap_or_else(|| "No evidence selected".to_string()),
        }
    }

    fn open_target(&self, item: ItemRef) -> Option<OpenTarget> {
        match item {
            ItemRef::Evidence(index) => self.evidence.get(index).map(|e| OpenTarget {
                path: PathBuf::from(&e.path),
                line: Some(e.line),
            }),
            ItemRef::Observation(index) => self.observations.get(index).map(|o| OpenTarget {
                path: PathBuf::from(&o.path),
                line: None,
            }),
            ItemRef::Relic(index) => self.relics.get(index).and_then(|r| {
                r.paths.first().map(|path| OpenTarget {
                    path: PathBuf::from(path),
                    line: None,
                })
            }),
            ItemRef::Entity(index) => {
                let entity = self.entities.get(index)?;
                let first_obs_id = entity.observations.first()?;
                let obs = self.observations.iter().find(|o| &o.id == first_obs_id)?;
                Some(OpenTarget {
                    path: PathBuf::from(&obs.path),
                    line: None,
                })
            }
            ItemRef::Relation(index) => {
                let rel = self.relations.get(index)?;
                let src_entity = self.entities.iter().find(|e| e.id == rel.src_entity)?;
                let obs_id = src_entity.observations.first()?;
                let obs = self.observations.iter().find(|o| &o.id == obs_id)?;
                Some(OpenTarget {
                    path: PathBuf::from(&obs.path),
                    line: None,
                })
            }
        }
    }
}

#[derive(Clone, Copy)]
enum ItemRef {
    Relic(usize),
    Entity(usize),
    Relation(usize),
    Observation(usize),
    Evidence(usize),
}

struct ItemEntry {
    label: String,
    item: ItemRef,
}

struct OpenTarget {
    path: PathBuf,
    line: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Sections,
    Items,
}

struct App {
    data: ArtifactData,
    root: PathBuf,
    section_idx: usize,
    item_idx: HashMap<&'static str, usize>,
    focus: FocusPane,
    search_mode: bool,
    search_query: String,
    status: String,
}

impl App {
    fn new(data: ArtifactData, root: PathBuf) -> Self {
        let mut item_idx = HashMap::new();
        for section in SECTION_ORDER {
            item_idx.insert(section, 0usize);
        }
        Self {
            data,
            root,
            section_idx: 0,
            item_idx,
            focus: FocusPane::Sections,
            search_mode: false,
            search_query: String::new(),
            status: "Ready".to_string(),
        }
    }

    fn current_section(&self) -> &'static str {
        SECTION_ORDER[self.section_idx]
    }

    fn current_item_index(&self) -> usize {
        *self.item_idx.get(self.current_section()).unwrap_or(&0)
    }

    fn set_current_item_index(&mut self, idx: usize) {
        self.item_idx.insert(self.current_section(), idx);
    }

    fn current_items(&self) -> Vec<ItemEntry> {
        self.data
            .filtered_items(self.current_section(), &self.search_query)
    }

    fn clamp_item_idx(&mut self) {
        let count = self.current_items().len();
        if count == 0 {
            self.set_current_item_index(0);
            return;
        }
        let current = self.current_item_index();
        if current >= count {
            self.set_current_item_index(count - 1);
        }
    }

    fn selected_item(&self) -> Option<ItemRef> {
        let items = self.current_items();
        let idx = self.current_item_index();
        items.get(idx).map(|e| e.item)
    }

    fn current_open_target(&self) -> Option<OpenTarget> {
        let mut target = self.data.open_target(self.selected_item()?)?;
        if target.path.is_relative() {
            target.path = self.root.join(target.path);
        }
        Some(target)
    }

    fn next(&mut self) {
        match self.focus {
            FocusPane::Sections => {
                self.section_idx = (self.section_idx + 1).min(SECTION_ORDER.len() - 1);
                self.clamp_item_idx();
            }
            FocusPane::Items => {
                let count = self.current_items().len();
                if count > 0 {
                    let idx = self.current_item_index();
                    self.set_current_item_index((idx + 1).min(count - 1));
                }
            }
        }
    }

    fn prev(&mut self) {
        match self.focus {
            FocusPane::Sections => {
                self.section_idx = self.section_idx.saturating_sub(1);
                self.clamp_item_idx();
            }
            FocusPane::Items => {
                let idx = self.current_item_index();
                self.set_current_item_index(idx.saturating_sub(1));
            }
        }
    }

    fn first(&mut self) {
        match self.focus {
            FocusPane::Sections => {
                self.section_idx = 0;
                self.clamp_item_idx();
            }
            FocusPane::Items => self.set_current_item_index(0),
        }
    }

    fn last(&mut self) {
        match self.focus {
            FocusPane::Sections => {
                self.section_idx = SECTION_ORDER.len() - 1;
                self.clamp_item_idx();
            }
            FocusPane::Items => {
                let count = self.current_items().len();
                if count > 0 {
                    self.set_current_item_index(count - 1);
                }
            }
        }
    }
}

fn draw_ui(f: &mut ratatui::Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(f.area());

    let title = Paragraph::new("Cheugy Artifact Browser")
        .block(Block::default().borders(Borders::ALL).title("Explorer"));
    f.render_widget(title, root[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(root[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(body[0]);

    let section_items: Vec<ListItem> = SECTION_ORDER
        .iter()
        .map(|name| {
            let count = app.data.section_count(name);
            ListItem::new(format!("{name} ({count})"))
        })
        .collect();

    let mut section_state = ListState::default();
    section_state.select(Some(app.section_idx));
    let section_block_title = if app.focus == FocusPane::Sections {
        "Structural Meta [focus]"
    } else {
        "Structural Meta"
    };
    let sections = List::new(section_items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ")
        .block(Block::default().borders(Borders::ALL).title(section_block_title));
    f.render_stateful_widget(sections, left[0], &mut section_state);

    let current_items = app.current_items();
    let item_rows: Vec<ListItem> = if current_items.is_empty() {
        vec![ListItem::new("(empty)")]
    } else {
        current_items
            .iter()
            .map(|row| ListItem::new(row.label.clone()))
            .collect()
    };

    let mut item_state = ListState::default();
    let item_select = if current_items.is_empty() {
        Some(0)
    } else {
        Some(app.current_item_index().min(current_items.len() - 1))
    };
    item_state.select(item_select);

    let item_block_title = if app.focus == FocusPane::Items {
        "Artifacts [focus]"
    } else {
        "Artifacts"
    };
    let items = List::new(item_rows)
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("• ")
        .block(Block::default().borders(Borders::ALL).title(item_block_title));
    f.render_stateful_widget(items, left[1], &mut item_state);

    let preview = app
        .selected_item()
        .map(|item| app.data.preview_for(item))
        .unwrap_or_else(|| "No artifact selected".to_string());
    let preview_widget = Paragraph::new(preview)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title("Quick Preview"));
    f.render_widget(preview_widget, body[1]);

    let search_title = if app.search_mode {
        "Search [typing]"
    } else {
        "Search"
    };
    let search = Paragraph::new(format!("/{}", app.search_query))
        .block(Block::default().borders(Borders::ALL).title(search_title));
    f.render_widget(search, root[2]);

    let help = Paragraph::new(format!(
        "j/k: move  h/l/tab: switch pane  /: filter  o: open file  g/G: first/last  q: quit  | {}",
        app.status
    ))
    .block(Block::default().borders(Borders::ALL).title("Keys / Status"));
    f.render_widget(help, root[3]);
}
