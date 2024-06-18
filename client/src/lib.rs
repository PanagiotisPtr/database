pub fn add(left: usize, right: usize) -> usize {
    left + right
}

mod client;
mod comms;

#[cfg(test)]
mod tests {
    use client::Client;
    use commons::command::Command;
    use commons::messages::{
        DelRequest, DelResponse, ExitRequest, ExitResponse, GetRequest, GetResponse, KeyType,
        PingRequest, PingResponse, SetRequest, SetResponse,
    };

    use super::*;

    #[test]
    fn it_works() {
        let mut c = Client::new("127.0.0.1:8080").unwrap();
        let key = KeyType::Str("hello".to_string());
        let value = String::from("world");

        println!("hello");
        let set_response = c
            .send(SetRequest {
                key: key.clone(),
                value: value.clone(),
            })
            .unwrap();
        assert_eq!(set_response, SetResponse {});

        println!("hello");
        let mut get_response: GetResponse<String> =
            c.send(GetRequest { key: key.clone() }).unwrap();
        assert_eq!(get_response, GetResponse { value: Some(value) });

        println!("hello");
        let del_response = c.send(DelRequest { key: key.clone() }).unwrap();
        assert_eq!(del_response, DelResponse {});

        println!("hello");
        get_response = c.send(GetRequest { key: key.clone() }).unwrap();
        assert_eq!(get_response, GetResponse { value: None });

        println!("hello");
        get_response = c.send(GetRequest { key: key.clone() }).unwrap();
        assert_eq!(get_response, GetResponse { value: None });

        println!("hello");
        let ping_response = c.send(PingRequest {}).unwrap();
        assert_eq!(ping_response, PingResponse {});

        println!("hello");
        let exit_response = c.send(ExitRequest {}).unwrap();
        assert_eq!(exit_response, ExitResponse {});
    }
}
