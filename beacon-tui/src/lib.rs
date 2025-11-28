use kameo::prelude::*;

mod draw;
mod event;

#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Send error: {0}")]
    Send(String),
}

impl<T> From<SendError<T>> for TuiError {
    fn from(err: SendError<T>) -> Self {
        TuiError::Send(err.to_string())
    }
}

pub struct TuiActor<A: Message<Stop>> {
    terminal: ratatui::DefaultTerminal,
    supervisor: ActorRef<A>,
}

/// Message to poll for events and redraw the TUI.
struct Poll;

/// Message to stop the server.
pub struct Stop;

impl<A: Message<Stop>> Actor for TuiActor<A> {
    type Args = ActorRef<A>;
    type Error = TuiError;

    async fn on_start(
        supervisor: Self::Args,
        actor_ref: ActorRef<Self>,
    ) -> Result<Self, Self::Error> {
        let terminal = ratatui::init();
        let _ = actor_ref.tell(Poll).await;
        Ok(Self {
            terminal,
            supervisor,
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
        self.terminal.draw(|frame| draw::draw(frame))?;

        // process events
        if let Ok(event) = crossterm::event::read() {
            ctx.actor_ref().tell(event).await?;
        }

        // do it again
        ctx.actor_ref().tell(Poll).await.map_err(TuiError::from)
    }
}
