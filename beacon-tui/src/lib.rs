use kameo::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tui_logger::{LevelFilter, TuiTracingSubscriberLayer};

use crate::draw::TuiWidget;

mod draw;
mod event;
mod format;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("logger error: {0}")]
    Logger(#[from] tui_logger::TuiLoggerError),
    #[error("send error: {0}")]
    Send(String),
}

impl<T> From<SendError<T>> for TuiError {
    fn from(err: SendError<T>) -> Self {
        TuiError::Send(err.to_string())
    }
}

pub struct TuiActor<A: Message<Stop>> {
    terminal: ratatui::DefaultTerminal,
    widget: TuiWidget,
    supervisor: ActorRef<A>,
}

/// Message to poll for events and redraw the TUI.
struct Poll;

/// Message to stop the server.
pub struct Stop;

pub fn register() -> Result<(), TuiError> {
    tui_logger::init_logger(LevelFilter::Debug)?;
    tui_logger::set_default_level(LevelFilter::Info);
    tracing_subscriber::registry()
        .with(TuiTracingSubscriberLayer)
        .init();
    Ok(())
}

impl<A: Message<Stop>> Actor for TuiActor<A> {
    type Args = ActorRef<A>;
    type Error = TuiError;

    async fn on_start(
        supervisor: Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let terminal = ratatui::init();
        let widget = TuiWidget::default();
        let _ = actor_ref.tell(Poll).await;

        Ok(Self {
            terminal,
            supervisor,
            widget,
        })
    }

    async fn on_stop(
        &mut self,
        _: WeakActorRef<Self>,
        _: ActorStopReason,
    ) -> Result<(), Self::Error> {
        ratatui::restore();

        Ok(())
    }
}

impl<A: Message<Stop>> Message<Poll> for TuiActor<A> {
    type Reply = Result<(), TuiError>;

    async fn handle(&mut self, _msg: Poll, ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // draw
        self.terminal
            .draw(|frame| frame.render_widget(&self.widget, frame.area()))?;

        // process events
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Ok(event) = crossterm::event::read() {
                ctx.actor_ref().tell(event).await?;
            }
        }

        // do it again
        ctx.actor_ref()
            .tell(Poll)
            .try_send()
            .map_err(TuiError::from)
    }
}
