extern crate fum_utils;
use fum_utils::hash::{compute_sha256, digest, Hash};

fn combined_hash(hash1: &Hash, hash2: &Hash) -> Hash {
    let mut hash = Vec::new();

    if hash1 < hash2 {
        hash.extend_from_slice(hash1);
        hash.extend_from_slice(hash2);
    } else {
        hash.extend_from_slice(hash2);
        hash.extend_from_slice(hash1);
    }
    compute_sha256(&hash)
}

fn print_multiple_hashes(hash_vec: &Vec<Hash>) {
    for hash in hash_vec {
        print!("{}", digest(hash));
        print!(" ");
    }
    println!("");
}

fn root_hash_step(hash_vec: &Vec<Hash>) -> Vec<Hash> {
    let mut new_hash_vec = Vec::new();

    for i in (0..hash_vec.len()).step_by(2) {
        if i < hash_vec.len() - 1 {
            new_hash_vec.push(combined_hash(&hash_vec[i], &hash_vec[i + 1]));
        } else {
            new_hash_vec.push(hash_vec[i].clone());
        }
    }
    new_hash_vec
}

pub fn root_hash(hash_vec: &Vec<Hash>) -> Hash {
    let mut hash_vec = hash_vec.clone();
    hash_vec.sort();

    while hash_vec.len() > 1 {
        hash_vec = root_hash_step(&hash_vec);
    }

    return hash_vec[0].clone();
}

fn find_proof_pair(hash_vec: &Vec<Hash>, hash: &Hash) -> Result<Hash, bool> {
    for i in (0..hash_vec.len()).step_by(2) {
        if i == hash_vec.len() - 1 {
            let is_ok = &hash_vec[i] == hash;
            return Err(is_ok);
        } else {
            if &hash_vec[i] == hash {
                return Ok(hash_vec[i + 1].clone());
            } else if &hash_vec[i + 1] == hash {
                return Ok(hash_vec[i].clone());
            }
        }
    }
    return Err(false);
}

pub fn get_proof(hash_vec: &Vec<Hash>, hash: &Hash) -> Vec<Hash> {
    let mut hash_vec = hash_vec.clone();
    hash_vec.sort();

    let mut current_proof: Hash = hash.clone();
    let mut proof = Vec::new();

    while hash_vec.len() > 1 {
        match find_proof_pair(&hash_vec, &current_proof) {
            Err(res) => {
                if !res {
                    panic!("Error while getting the proof");
                }
            }
            Ok(res) => {
                proof.push(res.clone());
                current_proof = combined_hash(&current_proof, &res);
            }
        };
        hash_vec = root_hash_step(&hash_vec)
    }

    match find_proof_pair(&hash_vec, &current_proof) {
        Err(res) => {
            if !res {
                panic!("Error while getting the proof");
            }
        }
        Ok(res) => {
            proof.push(res.clone());
            current_proof = combined_hash(&current_proof, &res);
        }
    };

    proof
}

pub fn verify_proof(proof: &Vec<Hash>, root_hash: &Hash, hash: &Hash) -> bool {
    let mut hash = hash.clone();
    for h in proof {
        hash = combined_hash(h, &hash);
    }
    hash == *root_hash
}
