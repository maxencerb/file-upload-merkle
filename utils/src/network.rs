use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn read_u32(stream: &mut TcpStream) -> Result<u32, &'static str> {
    let mut buffer = [0; 4];
    match stream.read_exact(&mut buffer) {
        Ok(_) => Ok(u32::from_be_bytes(buffer)),
        Err(_) => Err("Failed to read u32 from stream."),
    }
}

pub fn read_bytes(stream: &mut TcpStream) -> Result<Vec<u8>, &'static str> {
    let len = read_u32(stream)?;
    let mut buffer = vec![0; len as usize];
    match stream.read_exact(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(_) => Err("Failed to read bytes from stream."),
    }
}

pub fn read_string(stream: &mut TcpStream) -> Result<String, &'static str> {
    match read_bytes(stream) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(string) => Ok(string),
            Err(_) => Err("Failed to convert bytes to string."),
        },
        Err(_) => Err("Failed to read string from stream."),
    }
}

pub fn send_u32(stream: &mut TcpStream, value: u32) -> Result<(), &'static str> {
    let buffer = value.to_be_bytes();
    match stream.write_all(&buffer) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to send u32 to stream."),
    }
}

pub fn send_bytes(stream: &mut TcpStream, value: &[u8]) -> Result<(), &'static str> {
    let len = value.len() as u32;
    send_u32(stream, len)?;
    match stream.write_all(value) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to send bytes to stream."),
    }
}

pub fn send_string(stream: &mut TcpStream, value: &str) -> Result<(), &'static str> {
    match send_bytes(stream, value.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to send string to stream."),
    }
}
