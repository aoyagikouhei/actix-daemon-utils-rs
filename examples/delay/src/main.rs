use actix::prelude::*;
use actix_daemon_utils::{
    graceful_stop::{GracefulStop},
    delayer::{Delayer, Task, Timing},
};
use std::time::Duration;

struct MyActor { msg: String, seconds: u64 }

impl Actor for MyActor {
    type Context = Context<Self>;
}

impl Handler<Task> for MyActor {
    type Result = ();

    fn handle(&mut self, task: Task, _ctx: &mut Self::Context) -> Self::Result {
        println!("{}", self.msg);
        task.0.do_send(Timing::Later(Duration::from_secs(self.seconds)));
    }
}

fn main() {
    let sys = actix::System::new("main");
    let graceful_stop = GracefulStop::new();
    let actor1 = MyActor { msg: "x".to_string(), seconds: 1 }.start();
    let actor2 = MyActor { msg: "y".to_string(), seconds: 3 }.start();
    let delayer1 = Delayer::new(actor1.recipient(), graceful_stop.clone_system_terminator(), Duration::from_secs(10)).start();
    let delayer2 = Delayer::new(actor2.recipient(), graceful_stop.clone_system_terminator(), Duration::from_secs(10)).start();
    graceful_stop
        .subscribe(delayer1.recipient())
        .subscribe(delayer2.recipient())
        .start();

    let _ = sys.run();
    println!("main terminated");
}