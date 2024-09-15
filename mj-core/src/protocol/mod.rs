use ractor::RpcReplyPort;

mod file;
pub mod handler;
mod http;

pub enum MjProtocolMessage {
    Read(RpcReplyPort<String>),
    Write,
}