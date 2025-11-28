use crossterm::event::{Event, KeyCode, KeyModifiers};
use kameo::prelude::*;

use crate::{Stop, TuiActor, TuiError};

impl<A: Message<Stop>> Message<Event> for TuiActor<A> {
    type Reply = Result<(), TuiError>;

    async fn handle(&mut self, event: Event, ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        match event {
            Event::Key(event) if event.is_press() => match event.code {
                // exit on 'q' or Ctrl-C
                KeyCode::Char('q') => {
                    self.supervisor.tell(Stop).await?;
                    ctx.stop();
                }
                KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.supervisor.tell(Stop).await?;
                    ctx.stop();
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
}
