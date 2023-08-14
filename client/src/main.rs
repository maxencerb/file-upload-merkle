extern crate fum_merkle;
extern crate fum_utils;
extern crate rand;

use fum_merkle::merkle::{root_hash, verify_proof};
use fum_utils::op::UploadOp;
use rand::Rng;
use std::fs::{create_dir, remove_dir_all, File};
use std::{env, path::Path};
use std::io::{Write, Read};
use std::net::TcpStream;

use fum_utils::{
    hash::{compute_sha256, digest, hex_to_bytes, Hash},
    network::{read_bytes, read_string, read_u32, send_bytes, send_string, send_u32},
    op::{DOWNLOAD_OP, LISTDIRS_OP, LISTFILES_OP, UPLOAD_OP, UPLOAD, LISTDIRS, LISTFILES, DOWNLOAD, GENERATE},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: client operation <args> ...");
        println!("Operation: upload / download / listfiles / listdirs / generate");
        println!("The folder will be replaced by a .fum file with the same name containing the root hash.");
        println!("Example 1: client upload folder");
        println!("Example 2: client download folder file1 fum_file dest");
        println!("Example 3: client listfiles folder");
        println!("Example 4: client listdirs");
        println!("Example 5: client generate folder n_files");
        return;
    }

    let operation = &args[1];

    if operation == GENERATE {
      generate(&args);
      return;
    }

    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();

    match operation.as_str() {
        UPLOAD => upload_files(&mut stream, &args),
        LISTDIRS => list_dirs(&mut stream),
        LISTFILES => list_files(&mut stream, &args),
        DOWNLOAD => download(&mut stream, &args),
        "test" => test(),
        _ => println!("Unsupported operation."),
    }
}

fn test() {
    // let test = compute_sha256(&[12]);
    println!("{}", digest(&[12, 10]));
}

fn generate(args: &Vec<String>) {
    let dir = &args[2];
    
    let path = Path::new(dir);
    
    if !path.exists() {
        create_dir(path).unwrap();
    }

    let n_files: u32 = args[3].parse().unwrap();

    let mut rng = rand::thread_rng();
    for i in 0..n_files {
        let file_name = format!("file{}", i);
        let mut file = std::fs::File::create(path.join(file_name)).unwrap();
        for _ in 0..1000 {
            let str: char = rng.gen();
            file.write_all(str.to_string().as_bytes()).unwrap();
        }
    }

}

//WRITE
// number of files
// for each file
//   name size
//   name
//   file size
//  file content
fn upload_files(stream: &mut TcpStream, args: &Vec<String>) {
    // Upload operation
    stream.write_all(&[UPLOAD_OP]).unwrap();

    let upload_op: UploadOp = args.try_into().unwrap();
    let dir = upload_op.dir_path.clone();
    let dir = Path::new(&dir);

    send_u32(stream, upload_op.files_count).unwrap();

    let mut files_hash = Vec::new();

    for file in upload_op {
        files_hash.push(compute_sha256(&file.bytes));

        send_string(stream, &file.file_name).unwrap();
        send_bytes(stream, &file.bytes).unwrap();
    }

    remove_dir_all(&dir).unwrap();
    let mut file = std::fs::File::create(dir.with_extension("fum")).unwrap();

    let root_hash = root_hash(&files_hash);
    file.write_all(&root_hash).unwrap();
    println!("Root hash: {}", digest(&root_hash));
}

fn list_dirs(stream: &mut TcpStream) {
    // List dirs operation
    stream.write_all(&[LISTDIRS_OP]).unwrap();

    let ndir = read_u32(stream).unwrap();

    for _i in 0..ndir {
        let dir_name = read_string(stream).unwrap();
        println!("{}", dir_name);
    }
}

fn list_files(stream: &mut TcpStream, args: &Vec<String>) {
    let dir = &args[2];

    stream.write_all(&[LISTFILES_OP]).unwrap();

    // send the dir name
    send_string(stream, &dir).unwrap();

    // READ files
    let nfiles = read_u32(stream).unwrap();

    for _i in 0..nfiles {
        let file_name = read_string(stream).unwrap();
        println!("{}", file_name);
    }
}

// WRITE
// first 4 bytes: folder name length
// next: folder name bytes
// first 4 bytes: file name length
// next: file name bytes
// READ
// 4 bytes: file size
// next: file content
// first 4 bytes: proof size
// for each proof
//   first 4 bytes: proof length
//   next: proof bytes
fn download(stream: &mut TcpStream, args: &Vec<String>) {
    let folder = &args[2];
    let file_name = &args[3];
    let mut buffer = Vec::new();

    let mut fum_file = File::open(Path::new(&args[4]).with_extension("fum")).unwrap();
    fum_file.read_to_end(&mut buffer).unwrap();
    let root_hash: Hash = buffer.try_into().unwrap();
    
    stream.write_all(&[DOWNLOAD_OP]).unwrap();

    send_string(stream, &folder).unwrap();
    send_string(stream, &file_name).unwrap();

    let file_name = read_string(stream).unwrap();
    let file_content = read_bytes(stream).unwrap();

    let proof_size = read_u32(stream).unwrap();

    println!("Proof size: {}", proof_size);

    let mut proof: Vec<Hash> = Vec::new();

    for _i in 0..proof_size {
        proof.push(read_bytes(stream).unwrap().try_into().unwrap());
    }

    let file_hash = compute_sha256(&file_content);

    let is_ok = verify_proof(&proof, &root_hash, &file_hash);

    if !Path::new(&args[5]).exists() {
        create_dir(Path::new(&args[5])).unwrap();
    }

    let mut file = std::fs::File::create(Path::new(&args[5]).join(&file_name)).unwrap();
    file.write_all(&file_content).unwrap();

    if is_ok {
        println!("File {} is ok.", file_name);
    } else {
        println!("File {} is corrupted.", file_name);
    }
}
