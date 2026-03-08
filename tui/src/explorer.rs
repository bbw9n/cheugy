use anyhow::Result;
use cheugy_core::pipeline::read_jsonl;
use cheugy_core::schema::{Entity, Relic};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::path::Path;
use std::time::Duration;

pub fn run(root: &Path) -> Result<()> {
    let relics = read_jsonl::<Relic>(&root.join(".cheugy/relics.jsonl")).unwrap_or_default();
    let entities = read_jsonl::<Entity>(&root.join(".cheugy/entities.jsonl")).unwrap_or_default();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut selected = 0usize;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(f.area());

            let header = Paragraph::new("Cheugy Explorer — press q to quit, j/k to navigate")
                .block(Block::default().borders(Borders::ALL).title("Cheugy"));
            f.render_widget(header, chunks[0]);

            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(35),
                    Constraint::Percentage(30),
                    Constraint::Percentage(35),
                ])
                .split(chunks[1]);

            let relic_items: Vec<ListItem> = relics
                .iter()
                .enumerate()
                .map(|(idx, c)| {
                    let text = if idx == selected {
                        format!("> {}", c.label)
                    } else {
                        c.label.clone()
                    };
                    ListItem::new(text)
                })
                .collect();

            let relic_list = List::new(relic_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Relics")
                    .style(Style::default().add_modifier(Modifier::BOLD)),
            );

            let entity_items: Vec<ListItem> = entities
                .iter()
                .take(25)
                .map(|e| ListItem::new(format!("{}: {}", e.entity_type, e.canonical_name)))
                .collect();

            let entity_list = List::new(entity_items).block(Block::default().borders(Borders::ALL).title("Entities"));

            let code_view = if relics.is_empty() {
                Paragraph::new("No relics found. Run `cheugy scan .` then `cheugy build`.")
            } else {
                let c = &relics[selected.min(relics.len() - 1)];
                let mut lines = vec![
                    format!("Theme: {}", c.theme),
                    format!("Feature: {}", c.distinguishing_feature),
                    "".to_string(),
                    "Paths:".to_string(),
                ];
                for p in c.paths.iter().take(20) {
                    lines.push(format!("- {p}"));
                }
                Paragraph::new(lines.join("\n"))
            }
            .block(Block::default().borders(Borders::ALL).title("Code View"));

            f.render_widget(relic_list, body[0]);
            f.render_widget(entity_list, body[1]);
            f.render_widget(code_view, body[2]);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if !relics.is_empty() {
                            selected = (selected + 1).min(relics.len() - 1)
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        selected = selected.saturating_sub(1);
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
