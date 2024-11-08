
use dns_core::message::Message;
use dns_core::record::Record;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZoneTransferError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid AXFR request")]
    InvalidAxfrRequest,
    #[error("Invalid IXFR request")]
    InvalidIxfrRequest,
}
pub fn handle_axfr_rf(request: &Message, zone_data: &[Record]) -> io::Result<Message> {
    let mut response = Message::new();
    response.header.id = request.header.id;
    response.header.qr = true;
    response.header.aa = true;
    response.header.rd = request.header.rd;
    response.header.ra = true;
    response.header.rcode = 0;

    response.questions = request.questions.clone();
    response.answers = zone_data.to_vec();
    response.header.ancount = response.answers.len() as u16;

    Ok(response)
}

pub fn handle_ixfr(
    request: &Message,
    incremental_changes: &[Record],
) -> Result<Message, ZoneTransferError> {
    if request.questions.len() != 1 {
        return Err(ZoneTransferError::InvalidIxfrRequest);
    }

    let question = &request.questions[0];
    if question.qtype != 251 {

        return Err(ZoneTransferError::InvalidIxfrRequest);
    }

    let mut response = Message::new();
    response.header.id = request.header.id;
    response.header.qr = true;
    response.header.aa = true;
    response.header.rd = request.header.rd;
    response.header.ra = true;
    response.header.rcode = 0;

    response.questions.push(question.clone());
    response.answers.extend_from_slice(incremental_changes);
    response.header.ancount = response.answers.len() as u16;

    Ok(response)
}
