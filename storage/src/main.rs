use server::Server;

mod database;
mod server;

fn main() {
    let mut server = Server::new();

    server.listen("8080");
}
