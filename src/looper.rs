use crate::graceful_stop::{StopRequest, SystemTerminator};
use actix::prelude::*;
use std::sync::Arc;
use std::time::Duration;

/// This is a loop actor.
#[allow(dead_code)]
pub struct Looper {
    task: Recipient<Task>,
    system_terminator: Arc<SystemTerminator>,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
enum NextLoop {
    Immediately,
    Later(Duration),
}

/// This is a task call by Looper after specified seconds spend.
#[derive(Message, Debug)]
#[rtype(result = "Option<Duration>")]
pub struct Task;

impl Looper {
    pub fn new(task: Recipient<Task>, system_terminator: Arc<SystemTerminator>) -> Self {
        Self {
            task: task,
            system_terminator: system_terminator,
        }
    }
}

impl Actor for Looper {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.notify(NextLoop::Immediately);
    }
}

impl Handler<NextLoop> for Looper {
    type Result = ();

    fn handle(&mut self, msg: NextLoop, ctx: &mut Self::Context) {
        if let NextLoop::Later(duration) = msg {
            ctx.notify_later(NextLoop::Immediately, duration);
            return;
        }
        self.task
            .send(Task)
            .into_actor(self)
            .then(move |res, act, ctx2| {
                ctx2.notify(match res {
                    Ok(Some(duration)) => NextLoop::Later(duration),
                    _ => NextLoop::Immediately,
                });
                async {}.into_actor(act)
            })
            .wait(ctx);
    }
}

impl Handler<StopRequest> for Looper {
    type Result = <StopRequest as Message>::Result;

    fn handle(&mut self, _msg: StopRequest, ctx: &mut Self::Context) {
        ctx.stop();
    }
}
