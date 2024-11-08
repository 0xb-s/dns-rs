use crate::{Header, Question, Record};
use std::io::{self, Read, Write};

#[derive(Debug, Clone)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub additionals: Vec<Record>,
}

impl Message {
    pub fn new() -> Self {
        Message {
            header: Header::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let header = Header::read(reader)?;
        let mut questions = Vec::with_capacity(header.qdcount as usize);
        for _ in 0..header.qdcount {
            questions.push(Question::read(reader)?);
        }
        let mut answers = Vec::with_capacity(header.ancount as usize);
        for _ in 0..header.ancount {
            answers.push(Record::read(reader).unwrap());
        }
        let mut authorities = Vec::with_capacity(header.nscount as usize);
        for _ in 0..header.nscount {
            authorities.push(Record::read(reader).unwrap());
        }
        let mut additionals = Vec::with_capacity(header.arcount as usize);
        for _ in 0..header.arcount {
            additionals.push(Record::read(reader).unwrap());
        }
        Ok(Message {
            header,
            questions,
            answers,
            authorities,
            additionals,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.header.write(writer)?;
        for question in &self.questions {
            question.write(writer)?;
        }
        for answer in &self.answers {
            answer.write(writer)?;
        }
        for authority in &self.authorities {
            authority.write(writer)?;
        }
        for additional in &self.additionals {
            additional.write(writer)?;
        }
        Ok(())
    }
}
