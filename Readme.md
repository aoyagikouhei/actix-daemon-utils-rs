# actix-daemon-utils

Daemon Utilities by actix.

[Documentation](https://docs.rs/actix-daemon-utils)

## Features
- Graceful Stop by singals(hangup, interrupt, quit or terminate)
- Loop daemon(looper or delayer)

## Examples
```rust
use actix_daemon_utils::{
    actix::{
        prelude::*,
        System,
    },
    graceful_stop::{GracefulStop},
    looper::{Looper, Task},
};
use std::time::Duration;

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
    let sys = System::new("main");
    let graceful_stop = GracefulStop::new();
    let actor1 = MyActor { msg: "x".to_string(), seconds: 1 }.start();
    let actor2 = MyActor { msg: "y".to_string(), seconds: 3 }.start();
    let looper1 = Looper::new(actor1.recipient(), graceful_stop.clone_system_terminator()).start();
    let looper2 = Looper::new(actor2.recipient(), graceful_stop.clone_system_terminator()).start();
    graceful_stop
        .subscribe(looper1.recipient())
        .subscribe(looper2.recipient())
        .start();

    let _ = sys.run();
    println!("main terminated");
}
```