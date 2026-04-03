use ratatui::{style::Style, text::Span};

use crate::ui::{ACCENT, CLOSED_PR, MERGED_PR, MUTED};

pub(super) struct Footer {}

impl Footer {
  pub(super) fn helper() -> Vec<Span<'static>> {
    vec![
      Span::raw(" "),
      Span::styled("↑↓ jk", Style::default().fg(ACCENT)),
      Span::styled(" navigate  ", Style::default().fg(MUTED)),
      Span::styled("↵", Style::default().fg(ACCENT)),
      Span::styled(" switch  ", Style::default().fg(MUTED)),
      Span::styled("q", Style::default().fg(ACCENT)),
      Span::styled(" quit  ", Style::default().fg(MUTED)),
      Span::styled("^s", Style::default().fg(ACCENT)),
      Span::styled(" sync  ", Style::default().fg(MUTED)),
      Span::styled("⇧D", Style::default().fg(ACCENT)),
      Span::styled(" delete", Style::default().fg(MUTED)),
    ]
  }

  pub(super) fn sync() -> Vec<Span<'static>> {
    vec![
      Span::raw(" "),
      Span::styled(
        "⚠️ All local branches linked to a merged (",
        Style::default().fg(MUTED),
      ),
      Span::styled("M", Style::default().fg(MERGED_PR).bold()),
      Span::styled(") or closed (", Style::default().fg(MUTED)),
      Span::styled("C", Style::default().fg(CLOSED_PR).bold()),
      Span::styled(") PR will be deleted. Press ", Style::default().fg(MUTED)),
      Span::styled("↵", Style::default().fg(ACCENT)),
      Span::styled(" to confirm. Press ", Style::default().fg(MUTED)),
      Span::styled("c", Style::default().fg(ACCENT)),
      Span::styled(" to cancel.", Style::default().fg(MUTED)),
    ]
  }
}
