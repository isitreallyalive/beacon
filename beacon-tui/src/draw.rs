use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub(crate) fn draw(frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new("Press 'q' to quit")
            .block(Block::default().title("Beacon").borders(Borders::ALL)),
        frame.area(),
    );
}
