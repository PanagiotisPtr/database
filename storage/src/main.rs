use server::Server;

mod database;
mod server;

fn main() {
    let server = Server::new("8080");

    server.listen();
}
