use ratatui::prelude::*;
use tui_logger::TuiLoggerWidget;

use crate::format::Formatter;

#[derive(Default)]
pub struct HomeWidget;

impl Widget for &HomeWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        TuiLoggerWidget::default()
            .formatter(Box::new(Formatter))
            .render(area, buf);
    }
}
