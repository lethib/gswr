use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout},
  style::{Color, Style},
  text::{Line, Span},
  widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(2)])
    .split(frame.area());

  let branches = app
    .local_branches
    .iter()
    .map(|branch| {
      let commit_date = branch
        .last_commit_date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "---".to_string());
      let date_style = Style::default().fg(Color::DarkGray);

      let (prefix, style) = if branch.is_current {
        ("✓ ", Style::default().fg(Color::Green).bold())
      } else {
        ("  ", Style::default().fg(Color::White))
      };

      let line = Line::from(vec![
        Span::raw(prefix),
        Span::styled(&branch.name, style),
        Span::styled(commit_date, date_style),
      ]);

      ListItem::new(line)
    })
    .collect::<Vec<ListItem>>();

  let mut list_state = ListState::default();

  list_state.select(Some(app.selected as usize));

  let list = List::new(branches)
    .block(Block::default().borders(Borders::ALL).title(" gsw "))
    .highlight_style(Style::default().bg(Color::DarkGray).bold());

  frame.render_stateful_widget(list, chunks[0], &mut list_state);

  let hint = app
    .local_branches
    .get(app.selected as usize)
    .and_then(|b| b.last_commit_msg.as_deref())
    .unwrap_or("");

  let footer = Paragraph::new(format!(" ↑↓ navigate  ↵ switch  q quit   {}", hint))
    .style(Style::default().fg(Color::DarkGray));

  frame.render_widget(footer, chunks[1]);
}
