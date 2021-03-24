use crate::graceful_stop::{StopRequest, SystemTerminator};
use actix::prelude::*;
use std::sync::Arc;
use std::time::Duration;

/// This is a delay actor.
#[allow(dead_code)]
pub struct Delayer {
    task: Recipient<Task>,
    system_terminator: Arc<SystemTerminator>,
    error_duration: Duration,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub enum Timing {
    Immediately,
    Later(Duration),
}

/// This is a task call by Looper after specified seconds spend.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Task(pub Addr<Delayer>);

impl Delayer {
    pub fn new(
        task: Recipient<Task>,
        system_terminator: Arc<SystemTerminator>,
        error_duration: Duration,
    ) -> Self {
        Self {
            task: task,
            system_terminator: system_terminator,
            error_duration: error_duration,
        }
    }
}

impl Actor for Delayer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.notify(Timing::Immediately);
    }
}

impl Handler<Timing> for Delayer {
    type Result = ();

    fn handle(&mut self, msg: Timing, ctx: &mut Self::Context) {
        if let Timing::Later(duration) = msg {
            ctx.notify_later(Timing::Immediately, duration);
            return;
        }
        match self.task.do_send(Task(ctx.address())) {
            Err(SendError::Full(_)) => {
                ctx.notify(Timing::Later(self.error_duration));
            }
            _ => {}
        }
    }
}

impl Handler<StopRequest> for Delayer {
    type Result = <StopRequest as Message>::Result;

    fn handle(&mut self, _msg: StopRequest, ctx: &mut Self::Context) {
        ctx.stop();
    }
}
