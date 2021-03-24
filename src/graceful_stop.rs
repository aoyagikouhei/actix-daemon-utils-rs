use actix::prelude::*;
use actix_rt::signal::unix::{signal, SignalKind};
use crate::delayer::{Delayer, Task};
use futures_util::stream::once;
use std::{
    sync::{
        Arc,
        mpsc::Sender,
    },
    time::Duration,
};

/// This is a graceful stop for daemons.
pub struct GracefulStop {
    stop_request_recipients: Vec<Recipient<StopRequest>>,
    system_terminator: Arc<SystemTerminator>,
}

impl GracefulStop {
    pub fn new() -> GracefulStop {
        GracefulStop {
            stop_request_recipients: Vec::new(),
            system_terminator: Arc::new(SystemTerminator{sender: None}),
        }
    }

    pub fn new_with_sender(sender: Sender<()>) -> GracefulStop {
        GracefulStop {
            stop_request_recipients: Vec::new(),
            system_terminator: Arc::new(SystemTerminator{sender: Some(sender)}),
        }
    }

    pub fn subscribe(mut self, recipient: Recipient<StopRequest>) -> Self {
        self.stop_request_recipients.push(recipient);
        self
    }

    pub fn clone_system_terminator(&self) -> Arc<SystemTerminator> {
        Arc::clone(&self.system_terminator)
    }

    pub fn subscribe_ref(&mut self, recipient: Recipient<StopRequest>) {
        self.stop_request_recipients.push(recipient);
    }

    pub fn start_with_delayers(receipts: Vec<(Recipient<Task>, Duration)>) -> Addr<Self> {
        let mut graceful_stop = GracefulStop::new();
        for receipt in receipts {
            let delayer = Delayer::new(
                receipt.0,
                graceful_stop.clone_system_terminator(),
                receipt.1,
            )
            .start();
            graceful_stop.subscribe_ref(delayer.recipient());
        }
        graceful_stop.start()
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
            let mut s = signal(*signal_kind).unwrap();
            ctx.add_message_stream(once(async move {
                s.recv().await;
                StopEvent
            }));
        }
    }
}

/// This is a stop message for actors.
#[derive(Message)]
#[rtype(result = "()")]
pub struct StopRequest;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopEvent;

impl Handler<StopEvent> for GracefulStop {
    type Result = ();

    fn handle(&mut self, _signal_event: StopEvent, ctx: &mut Self::Context) {
        for recipient in self.stop_request_recipients.drain(..) {
            let _ = recipient.do_send(StopRequest);
        }
        ctx.stop();
    }
}

/// This is a system terminator. This will stop all Arc<SystemTerminator> released than is having actors.
pub struct SystemTerminator{
    sender: Option<Sender<()>>
}

impl Drop for SystemTerminator {
    fn drop(&mut self) {
        match self.sender {
            Some(ref sender) => {
                sender.send(()).unwrap();
            },
            None => {
                actix::System::current().stop();
            }
        }
    }
}
