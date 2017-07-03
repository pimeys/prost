extern crate bytes;
extern crate env_logger;
extern crate test_all_types;
extern crate prost;
#[macro_use]
extern crate prost_derive;

include!(concat!(env!("OUT_DIR"), "/conformance.rs"));

use std::io::{
    Read,
    Write,
    self,
};

use bytes::{
    Buf,
    BufMut,
    BytesMut,
    LittleEndian,
};
use prost::Message;

use test_all_types::protobuf_test_messages::proto3::TestAllTypes;
use test_all_types::{
    RoundtripResult,
    roundtrip,
};

fn main() {
    env_logger::init().unwrap();
    let mut buf = &mut BytesMut::new();

    loop {
        eprintln!("loop!");
        buf.reserve(4);

        if let Err(error) = io::copy(&mut io::stdin().take(4), &mut buf.writer()) {
            eprintln!("error: {}", error);
            // No more test cases.
            break;
        }

        let len = buf.take().freeze().get_u32::<LittleEndian>() as usize;

        buf.reserve(len);
        io::copy(&mut io::stdin().take(len as u64), &mut buf.writer()).unwrap();

        let result = match ConformanceRequest::decode(&mut buf.take().freeze()) {
            Ok(request) => handle_request(request),
            Err(error) => conformance_response::Result::ParseError(format!("{:?}", error)),
        };

        let mut response = ConformanceResponse::default();
        response.result = Some(result);

        let len = response.encoded_len();
        // TODO: drop the reserve.
        buf.reserve(len + 4);
        buf.put_u32::<LittleEndian>(len as u32);
        response.encode(buf);
        assert_eq!(len + 4, buf.len());

        let mut stdout = io::stdout();
        stdout.lock().write_all(&buf[..]).unwrap();
        stdout.flush().unwrap();
    }
}

fn handle_request(request: ConformanceRequest) -> conformance_response::Result {
    match request.requested_output_format() {
        Some(WireFormat::Json) => {
            return conformance_response::Result::Skipped("JSON output is not supported".to_string());
        },
        None => {
            return conformance_response::Result::ParseError("unrecognized requested output format".to_string());
        },
        _ => (),
    };

    let buf = match request.payload {
        None => return conformance_response::Result::ParseError("no payload".to_string()),
        Some(conformance_request::Payload::JsonPayload(_)) =>
            return conformance_response::Result::Skipped("JSON input is not supported".to_string()),
        Some(conformance_request::Payload::ProtobufPayload(buf)) => buf,
    };

    match roundtrip::<TestAllTypes>(&buf) {
        RoundtripResult::Ok(buf) => {
            conformance_response::Result::ProtobufPayload(buf)
        },
        RoundtripResult::DecodeError(error) => {
            conformance_response::Result::ParseError(error.to_string())
        },
        RoundtripResult::Error(error) => {
            conformance_response::Result::RuntimeError(error.to_string())
        },
    }
}
