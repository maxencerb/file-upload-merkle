extern crate fum_merkle;
extern crate fum_utils;

use fum_merkle::merkle::{get_proof, root_hash};
use std::fs::{create_dir, read_dir, rename, File};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use fum_utils::{
    hash::{compute_sha256, digest},
    network::{read_bytes, read_string, read_u32, send_bytes, send_string, send_u32},
    op::{DOWNLOAD_OP, LISTDIRS_OP, LISTFILES_OP, UPLOAD_OP},
};

const TEMP_PATH: &str = "temp";
const UPLOAD_PATH: &str = "uploads";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
      
        handle_client(stream.unwrap());
    }
}

fn handle_client(mut stream: std::net::TcpStream) {
    let mut op_buffer = [0; 1];
    stream.read_exact(&mut op_buffer).unwrap();

    match op_buffer[0] {
        UPLOAD_OP => upload_files(&mut stream),
        LISTDIRS_OP => list_dirs(&mut stream),
        LISTFILES_OP => list_files(&mut stream),
        DOWNLOAD_OP => download(&mut stream),
        _ => println!("Unsupported operation."),
    }
}

// first 4 bytes: number of files
// for each file:
//   first 4 bytes: file name length
//   next file name length bytes: file name
//   first 4 bytes: file size
//   next file size bytes: file content
fn upload_files(stream: &mut TcpStream) {
    let files_count = read_u32(stream).unwrap();

    if !Path::new(&TEMP_PATH).exists() {
        create_dir(&TEMP_PATH).unwrap();
    }

    let mut files_hash = Vec::new();

    for _i in 0..files_count {
        let file_name = read_string(stream).unwrap();
        let file_content = read_bytes(stream).unwrap();

        files_hash.push(compute_sha256(&file_content));

        let mut file = File::create(Path::new(&TEMP_PATH).join(&file_name)).unwrap();
        file.write_all(&file_content).unwrap();
    }

    let root_hash = root_hash(&files_hash);

    // move all files to a folder with the name of the root hash
    if !Path::new(&UPLOAD_PATH).exists() {
        create_dir(&UPLOAD_PATH).unwrap();
    }

    rename(TEMP_PATH, Path::new(&UPLOAD_PATH).join(digest(&root_hash))).unwrap();
}

// first 4 bytes: number of directories
// for each directory:
//   first 4 bytes: directory name length
//   next directory name length bytes: directory name
fn list_dirs(stream: &mut TcpStream) {
    let mut dirs = Vec::new();
    for entry in read_dir(&UPLOAD_PATH).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            dirs.push(entry.path());
        }
    }

    let dirs_count = dirs.len() as u32;
    send_u32(stream, dirs_count).unwrap();

    for dir in dirs.iter() {
        let dir_name = dir.file_name().unwrap().to_str().unwrap();
        send_string(stream, dir_name).unwrap();
    }
}

// READ
// first 4 bytes: directory name length
// next: directory name bytes
// WRITE
// first 4 bytes: number of files
// for each file:
//   first 4 bytes: file name length
//   next file name length bytes: file name
fn list_files(stream: &mut TcpStream) {
    let dir_name = read_string(stream).unwrap();

    let mut files = Vec::new();

    for entry in read_dir(Path::new(&UPLOAD_PATH).join(dir_name)).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_file() {
            files.push(entry.path());
        }
    }

    let files_count = files.len() as u32;
    send_u32(stream, files_count).unwrap();

    for file in files.iter() {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        send_string(stream, file_name).unwrap();
    }
}

// READ
// first 4 bytes: folder name length
// next: folder name bytes
// first 4 bytes: file name length
// next: file name bytes
// WRITE
// 4 bytes: file size
// next: file content
// first 4 bytes: proof size
// for each proof
//   first 4 bytes: proof length
//   next: proof bytes
fn download(stream: &mut TcpStream) {
    let dir_name = read_string(stream).unwrap();
    let file_name = read_string(stream).unwrap();

    let file_path = Path::new(&UPLOAD_PATH).join(&dir_name).join(&file_name);

    println!("Downloading file: {:?}", file_path);

    let mut file: File = File::open(&file_path).unwrap();
    let file_size = file.metadata().unwrap().len();

    let mut buffer = vec![0; file_size as usize];
    file.read_exact(&mut buffer).unwrap();

    send_string(stream, &file_name).unwrap();
    send_bytes(stream, &buffer).unwrap();

    let dir = Path::new(&UPLOAD_PATH).join(&dir_name);
    let mut files_hash = Vec::new();

    for file in read_dir(dir).unwrap() {
        let file = file.unwrap();
        if file.path().is_file() {
            let len = file.metadata().unwrap().len();
            let mut buffer = vec![0; len as usize];
            let mut file = File::open(file.path()).unwrap();
            file.read_exact(&mut buffer).unwrap();
            files_hash.push(compute_sha256(&buffer));
        }
    }

    let hash: [u8; 32] = compute_sha256(&buffer);
    let proof = get_proof(&files_hash, &hash);

    let proof_size = proof.len() as u32;
    println!("Proof size: {}", proof_size);
    send_u32(stream, proof_size).unwrap();

    for p in proof.iter() {
        send_bytes(stream, p).unwrap();
    }
}
