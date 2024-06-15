pub fn add(left: usize, right: usize) -> usize {
    left + right
}

mod client;
mod command;
mod comms;

#[cfg(test)]
mod tests {
    use client::Client;
    use command::Command;
    use commons::messages::{GetRequest, GetResponse, SetRequest, SetResponse};

    use super::*;

    #[test]
    fn it_works() {
        let mut c = Client::new("localhost:8080").unwrap();
        let set_request = SetRequest {
            key: "hello",
            value: String::from("world"),
        };

        let set_response: SetResponse = c.send(set_request).unwrap();
        println!("set_response: {:#?}", set_response);

        let get_request = GetRequest { key: "hello" };

        let get_response: GetResponse<String> = c.send(get_request).unwrap();
        println!("get_response: {:#?}", get_response);
    }
}
