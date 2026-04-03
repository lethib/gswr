use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};

use throbber_widgets_tui::BOX_DRAWING;

use crate::{GSWRError, app::App, git::PRStatus, ui::footers::Footer};

pub mod footers;

pub(super) const ACCENT: Color = Color::Rgb(130, 170, 255);
const CURRENT: Color = Color::Rgb(100, 210, 130);
pub(super) const MUTED: Color = Color::Rgb(90, 100, 120);
const SELECTED_BG: Color = Color::Rgb(35, 45, 65);
const TEXT: Color = Color::Rgb(190, 200, 220);
const BORDER: Color = Color::Rgb(55, 65, 85);

pub(super) const MERGED_PR: Color = Color::Rgb(100, 160, 255);
const OPENED_PR: Color = Color::Rgb(100, 210, 130);
pub(super) const CLOSED_PR: Color = Color::Rgb(180, 60, 60);

fn truncate(s: &str, max_chars: usize) -> String {
  if max_chars == 0 {
    return String::new();
  }
  if s.chars().count() <= max_chars {
    s.to_string()
  } else {
    let truncated: String = s.chars().take(max_chars - 3).collect();
    format!("{}…", truncated)
  }
}

pub fn draw(frame: &mut Frame, app: &App) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(2)])
    .split(frame.area());

  // 2 border chars + 3 column spacing chars (1 gap between each of the 4 columns)
  let inner_width = (frame.area().width as usize).saturating_sub(5);
  let col1_max = inner_width * 20 / 100;
  let col2_max: usize = 12; // "YYYY-MM-DD" + padding
  let col3_status: usize = 3;
  let col4_max = inner_width
    .saturating_sub(col1_max)
    .saturating_sub(col2_max)
    .saturating_sub(col3_status);

  let rows = app
    .local_branches
    .iter()
    .map(|branch| {
      let commit_date = branch
        .last_commit_date
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "---".to_string());

      let spinner_sym = {
        let idx = (app.throbber_state.index() as isize)
          .rem_euclid(BOX_DRAWING.symbols.len() as isize) as usize;
        BOX_DRAWING.symbols[idx]
      };

      let (status_cell, pr_text, pr_style) = match &branch.pr {
        Ok(pr) => match pr {
          None => (
            Cell::from(""),
            format!(" {}", spinner_sym),
            Style::default().fg(MUTED),
          ),
          Some(defined_pr) => {
            let (letter, color) = match defined_pr.status {
              PRStatus::OPENED => ("O", OPENED_PR),
              PRStatus::MERGED => ("M", MERGED_PR),
              PRStatus::CLOSED => ("C", CLOSED_PR),
            };
            (
              Cell::from(letter).style(Style::default().fg(color).bold()),
              defined_pr.title.clone(),
              Style::default().fg(color).add_modifier(Modifier::ITALIC),
            )
          }
        },
        Err(error) => match error {
          GSWRError::PR_NOT_FOUND => (
            Cell::from(""),
            "No PR".to_string(),
            Style::default().fg(MUTED),
          ),
          GSWRError::Custom(msg) => (Cell::from(""), msg.clone(), Style::default().fg(Color::Red)),
        },
      };

      if branch.is_current {
        Row::new(vec![
          Cell::from(truncate(&format!(" ● {}", branch.name), col1_max))
            .style(Style::default().fg(CURRENT).bold()),
          Cell::from(truncate(&commit_date, col2_max)).style(Style::default().fg(MUTED)),
          status_cell,
          Cell::from(truncate(&pr_text, col4_max)).style(pr_style),
        ])
      } else {
        Row::new(vec![
          Cell::from(truncate(&format!("   {}", branch.name), col1_max))
            .style(Style::default().fg(TEXT)),
          Cell::from(truncate(&commit_date, col2_max)).style(Style::default().fg(MUTED)),
          status_cell,
          Cell::from(truncate(&pr_text, col4_max)).style(pr_style),
        ])
      }
    })
    .collect::<Vec<Row>>();

  let mut table_state = TableState::default();
  table_state.select(Some(app.selected as usize));

  let table = Table::new(
    rows,
    [
      Constraint::Length(col1_max as u16),
      Constraint::Length(col2_max as u16),
      Constraint::Length(col3_status as u16),
      Constraint::Length(col4_max as u16),
    ],
  )
  .block(
    Block::default()
      .borders(Borders::ALL)
      .border_type(BorderType::Rounded)
      .border_style(Style::default().fg(BORDER))
      .title(Line::from(Span::styled(
        format!(" gswr v{} ", env!("CARGO_PKG_VERSION")),
        Style::default().fg(ACCENT).bold(),
      )))
      .title_alignment(Alignment::Left),
  )
  .row_highlight_style(
    Style::default()
      .bg(SELECTED_BG)
      .add_modifier(Modifier::BOLD),
  );

  frame.render_stateful_widget(table, chunks[0], &mut table_state);

  let hint = app
    .local_branches
    .get(app.selected as usize)
    .and_then(|b| b.last_commit_msg.as_deref())
    .unwrap_or("");

  let footer_spans = match &app.error_message {
    Some(message) => vec![
      Span::raw(" "),
      Span::styled(message, Style::default().fg(Color::Red)),
    ],
    None => {
      if app.confirming_sync {
        Footer::sync()
      } else {
        let mut spans = Footer::helper();

        if !hint.is_empty() {
          spans.push(Span::styled("   Last commit: ", Style::default().fg(MUTED)));
          spans.push(Span::styled(
            hint,
            Style::default().fg(TEXT).add_modifier(Modifier::ITALIC),
          ));
        }

        spans
      }
    }
  };

  let footer = Paragraph::new(Line::from(footer_spans));

  frame.render_widget(footer, chunks[1]);
}
