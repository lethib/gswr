use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::App;

const ACCENT: Color = Color::Rgb(130, 170, 255);
const CURRENT: Color = Color::Rgb(100, 210, 130);
const MUTED: Color = Color::Rgb(90, 100, 120);
const SELECTED_BG: Color = Color::Rgb(35, 45, 65);
const TEXT: Color = Color::Rgb(190, 200, 220);
const BORDER: Color = Color::Rgb(55, 65, 85);

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

      let name = &branch.name;

      if branch.is_current {
        ListItem::new(Line::from(vec![
          Span::styled(" ● ", Style::default().fg(CURRENT).bold()),
          Span::styled(name.as_str(), Style::default().fg(CURRENT).bold()),
          Span::styled(format!("  {}", commit_date), Style::default().fg(MUTED)),
        ]))
      } else {
        ListItem::new(Line::from(vec![
          Span::styled("   ", Style::default()),
          Span::styled(name.as_str(), Style::default().fg(TEXT)),
          Span::styled(format!("  {}", commit_date), Style::default().fg(MUTED)),
        ]))
      }
    })
    .collect::<Vec<ListItem>>();

  let mut list_state = ListState::default();
  list_state.select(Some(app.selected as usize));

  let list = List::new(branches)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .title(Line::from(Span::styled(" gsw ", Style::default().fg(ACCENT).bold())))
        .title_alignment(Alignment::Left),
    )
    .highlight_style(
      Style::default()
        .bg(SELECTED_BG)
        .add_modifier(Modifier::BOLD),
    );

  frame.render_stateful_widget(list, chunks[0], &mut list_state);

  let hint = app
    .local_branches
    .get(app.selected as usize)
    .and_then(|b| b.last_commit_msg.as_deref())
    .unwrap_or("");

  let mut footer_spans = vec![
    Span::raw(" "),
    Span::styled("↑↓ jk", Style::default().fg(ACCENT)),
    Span::styled(" navigate  ", Style::default().fg(MUTED)),
    Span::styled("↵", Style::default().fg(ACCENT)),
    Span::styled(" switch  ", Style::default().fg(MUTED)),
    Span::styled("q", Style::default().fg(ACCENT)),
    Span::styled(" quit", Style::default().fg(MUTED)),
  ];

  if !hint.is_empty() {
    footer_spans.push(Span::styled("   Last commit: ", Style::default().fg(MUTED)));
    footer_spans.push(Span::styled(hint, Style::default().fg(TEXT).add_modifier(Modifier::ITALIC)));
  }

  let footer = Paragraph::new(Line::from(footer_spans));

  frame.render_widget(footer, chunks[1]);
}
