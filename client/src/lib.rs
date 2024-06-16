pub fn add(left: usize, right: usize) -> usize {
    left + right
}

mod client;
mod comms;

#[cfg(test)]
mod tests {
    use client::Client;
    use commons::command::Command;
    use commons::messages::{GetRequest, GetResponse, KeyType, SetRequest, SetResponse};

    use super::*;

    #[test]
    fn it_works() {
        let mut c = Client::new("localhost:8080").unwrap();
        let set_request = SetRequest {
            key: KeyType::Str("hello".to_string()),
            value: String::from("world"),
        };

        let set_response: SetResponse = c.send(set_request).unwrap();
        println!("set_response: {:#?}", set_response);

        let get_request = GetRequest {
            key: KeyType::Str("hello".to_string()),
        };

        let get_response: GetResponse<String> = c.send(get_request).unwrap();
        println!("get_response: {:#?}", get_response);
    }
}
