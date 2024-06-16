use anyhow::Result;

pub trait Command<Request, Response> {
    fn send(&mut self, request: Request) -> Result<Response>;
}
