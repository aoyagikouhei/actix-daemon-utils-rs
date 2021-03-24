use actix_daemon_utils::{
    actix::{
        prelude::*,
        System,
    },
    graceful_stop::{GracefulStop},
    looper::{Looper, Task},
};
use std::{
    sync::mpsc,
    thread,
    time::Duration,
};

struct MyActor { msg: String, seconds: u64 }

impl Actor for MyActor {
    type Context = Context<Self>;
}

impl Handler<Task> for MyActor {
    type Result = Option<std::time::Duration>;

    fn handle(&mut self, _msg: Task, _ctx: &mut Self::Context) -> Self::Result {
        println!("{}", self.msg);
        Some(Duration::from_secs(self.seconds))
    }
}

fn main() {
    let (tx, rx) = mpsc::channel::<()>();

    let sys = System::new();
    let graceful_stop = GracefulStop::new_with_sender(tx);
    //let graceful_stop = GracefulStop::new();
    let actor1 = MyActor { msg: "x".to_string(), seconds: 1 }.start();
    let actor2 = MyActor { msg: "y".to_string(), seconds: 3 }.start();
    let looper1 = Looper::new(actor1.recipient(), graceful_stop.clone_system_terminator()).start();
    let looper2 = Looper::new(actor2.recipient(), graceful_stop.clone_system_terminator()).start();
    graceful_stop
        .subscribe(looper1.recipient())
        .subscribe(looper2.recipient())
        .start();

    let sys2 = System::current();

    thread::spawn(move || {
        rx.recv().unwrap();

        println!("ended");

        sys2.stop();
    });

    let _ = sys.run();

    println!("main terminated");
}