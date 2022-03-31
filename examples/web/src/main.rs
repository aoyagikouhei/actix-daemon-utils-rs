// refer from https://github.com/actix/examples/tree/master/shutdown-server

use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder};
use std::{sync::mpsc, thread};

use actix_daemon_utils::{
    actix::prelude::*,
    graceful_stop::{GracefulStop, StopEvent},
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

#[get("/hello")]
async fn hello() -> impl Responder {
    "Hello world!"
}

#[get("/stop")]
async fn stop(stopper: web::Data<mpsc::Sender<()>>) -> impl Responder {
    // make request that sends message through the Sender
    stopper.send(()).unwrap();

    HttpResponse::NoContent().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=debug,actix_web=debug");
    env_logger::init();

    // create a channel
    let (tx, rx) = mpsc::channel::<()>();

    let bind = "127.0.0.1:8080";

    // start server as normal but don't .await after .run() yet
    let server = HttpServer::new(move || {
        // give the server a Sender in .data
        App::new()
            .app_data(web::Data::new(tx.clone()))
            .wrap(middleware::Logger::default())
            .service(hello)
            .service(stop)
    })
    .bind(&bind)?
    .disable_signals()
    .run();

    let (tx2, rx2) = mpsc::channel::<()>();
    let graceful_stop = GracefulStop::new_with_sender(tx2);
    let actor1 = MyActor { msg: "x".to_string(), seconds: 1 }.start();
    let actor2 = MyActor { msg: "y".to_string(), seconds: 3 }.start();
    let looper1 = Looper::new(actor1.recipient(), graceful_stop.clone_system_terminator()).start();
    let looper2 = Looper::new(actor2.recipient(), graceful_stop.clone_system_terminator()).start();
    let addr = graceful_stop
        .subscribe(looper1.recipient())
        .subscribe(looper2.recipient())
        .start();

    thread::spawn(move || {
        let _ = rx.recv();

        addr.do_send(StopEvent);
    });

    let srv = server.handle();
    thread::spawn(move || {
        rx2.recv().unwrap();

        // stop server gracefully
        srv.stop(true)
    });

    // run server
    server.await
}
