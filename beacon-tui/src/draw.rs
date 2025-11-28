use ratatui::prelude::*;
use tui_logger::TuiLoggerWidget;

use crate::format::Formatter;

#[derive(Default)]
pub struct TuiWidget;

impl Widget for &TuiWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        TuiLoggerWidget::default()
            .formatter(Box::new(Formatter))
            .render(area, buf);
    }
}
