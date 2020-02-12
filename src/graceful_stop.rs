use actix::prelude::*;
use actix_rt::signal::unix::{signal, SignalKind};
use std::sync::Arc;
use tokio::stream::StreamExt;

/// This is a graceful stop for daemons.
pub struct GracefulStop {
    stop_request_recipients: Vec<Recipient<StopRequest>>,
    system_terminator: Arc<SystemTerminator>,
}

impl GracefulStop {
    pub fn new() -> GracefulStop {
        GracefulStop {
            stop_request_recipients: Vec::new(),
            system_terminator: Arc::new(SystemTerminator),
        }
    }

    pub fn subscribe(mut self, recipient: Recipient<StopRequest>) -> Self {
        self.stop_request_recipients.push(recipient);
        self
    }

    pub fn clone_system_terminator(&self) -> Arc<SystemTerminator> {
        Arc::clone(&self.system_terminator)
    }
}

impl Actor for GracefulStop {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let signals = vec![
            Box::new(SignalKind::hangup()),
            Box::new(SignalKind::interrupt()),
            Box::new(SignalKind::quit()),
            Box::new(SignalKind::terminate()),
        ];
        for signal_kind in signals.into_iter() {
            let s = signal(*signal_kind).unwrap();
            ctx.add_message_stream(s.map(move |_| SignalEvent))
        }
    }
}

/// This is a stop message for actors.
#[derive(Message)]
#[rtype(result = "()")]
pub struct StopRequest;

#[derive(Message)]
#[rtype(result = "()")]
struct SignalEvent;

impl Handler<SignalEvent> for GracefulStop {
    type Result = ();

    fn handle(&mut self, _signal_event: SignalEvent, ctx: &mut Self::Context) {
        for recipient in self.stop_request_recipients.drain(..) {
            let _ = recipient.do_send(StopRequest);
        }
        ctx.stop();
    }
}

/// This is a system terminator. This will stop all Arc<SystemTerminator> released than is having actors.
pub struct SystemTerminator;
impl Drop for SystemTerminator {
    fn drop(&mut self) {
        actix::System::current().stop();
    }
}
