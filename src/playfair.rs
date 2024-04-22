mod constants;
use itertools::Itertools;

pub fn generate_key_matrix(string: &String) -> Vec<char> {
    let mut key = string.to_uppercase();
    key = key.replace("J", "I");
    let mut alpha: Vec<char> = "ABCDEFGHIKLMNOPQRSTUVWXYZ".chars().collect();
    let mut key: Vec<char> = key.chars().collect();
    key.retain(|&c| !c.is_whitespace());
    key.retain(|&c| c.is_alphabetic());
    key.append(&mut alpha);
    let key: Vec<char> = key.into_iter().unique().collect::<Vec<char>>();
    key
}

fn is_dna_cipher(c: char) -> bool {
    c == 'A' || c == 'U' || c == 'C' || c == 'G' || c == 'N' || c == '-'
}

pub fn split_cipher(text: &String, before: bool) -> (Vec<char>, Vec<u8>) {
    let mut text: Vec<char> = text.chars().collect();
    text.retain(|&c| is_dna_cipher(c));
    let text: String = text.iter().collect();
    let text = text.split("-").collect::<Vec<&str>>();
    if text.len() != 2 {
        return (Vec::new(), Vec::new());
    }
    let dna: Vec<char>;
    let (dna_index, ambig_index) = match before {
        true => (1usize, 0usize),
        false => (0usize, 0usize),
    };
    dna = text[dna_index].chars().collect();
    let mut ambig: Vec<u8> = Vec::with_capacity(text[ambig_index].len());
    for c in text[ambig_index].chars() {
        ambig.push(constants::DNA_REVERSE[&c]);
    }
    (dna, ambig)
}

pub fn utf8_to_binary(text: &String) -> Vec<u8> {
    let mut bin = text.as_bytes().to_vec();
    while bin.len() % 3 != 0 {
        bin.push(b' ');
    }
    bin
}

pub fn utf16_to_binary(text: &Vec<u16>) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::with_capacity(text.len() * 2);
    for two_bytes in text.iter() {
        let split = two_bytes.to_be_bytes();
        bin.push(split[0]);
        bin.push(split[1]);
    }
    while (bin.len() / 2) % 3 != 0 {
        bin.push(0b00100000);
        bin.push(0b00000000);
    }
    bin
}

pub fn binary_to_dna(bin: &Vec<u8>) -> Vec<char> {
    let mut dna_vec: Vec<char> = Vec::with_capacity(bin.len() * 4);
    for byte in bin.iter() {
        for j in 0..4 {
            dna_vec.push(constants::DNA[&byte_fourth(j * 2, *byte)]);
        }
    }
    dna_vec
}

pub fn dna_to_binary(dna: &String) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(dna.len() / 4);
    let dna: Vec<char> = dna.chars().collect();
    let offset = match dna.is_empty() {
        true => 0usize,
        false => 3,
    };
    for i in (0..dna.len() - offset).step_by(4) {
        let mut b: u8 = 0;
        for j in 0..4usize {
            let c = match constants::DNA_REVERSE.get(&dna[i + j]) {
                Some(&v) => v,
                None => {
                    println!("error in dna_to_bin, substituting with a 0");
                    0u8
                }
            };
            b = b | c << (6 - j * 2);
        }
        // let b: u8 = (constants::DNA_REVERSE[&dna[i]] << 6)
        //     | (constants::DNA_REVERSE[&dna[i + 1]] << 4)
        //     | (constants::DNA_REVERSE[&dna[i + 2]] << 2)
        //     | (constants::DNA_REVERSE[&dna[i + 3]]);
        bytes.push(b);
    }
    bytes
}

pub fn dna_to_acids(dna_vec: &Vec<char>) -> (Vec<char>, Vec<u8>) {
    let mut triplets: Vec<String> = Vec::with_capacity(dna_vec.len() / 3);
    let offset = dna_vec.len() % 3;
    for i in (0..dna_vec.len() - offset).step_by(3) {
        let mut triplet = String::new();
        triplet.push(dna_vec[i]);
        triplet.push(dna_vec[i + 1]);
        triplet.push(dna_vec[i + 2]);
        triplets.push(triplet);
    }
    let mut acid_vec: Vec<char> = Vec::with_capacity(triplets.len());
    let mut ambig_vec: Vec<u8> = Vec::with_capacity(triplets.len());
    for acid in triplets.iter() {
        let (c, ambig) = match constants::ACID.get(acid) {
            Some(&(c, ambig)) => (c, ambig),
            None => {
                println!("something went wrong; wrong ACID key");
                ('X', 0) // shouldnt happen
            }
        };
        acid_vec.push(c);
        ambig_vec.push(ambig);
    }
    (acid_vec, ambig_vec)
}

pub fn acids_to_dna(acid_vec: &Vec<char>, ambig_vec: &Vec<u8>) -> Vec<char> {
    let mut dna_vec: Vec<char> = Vec::with_capacity(acid_vec.len() * 3);
    for i in 0..acid_vec.len() {
        let key = format!("{}{}", acid_vec[i], ambig_vec[i]);
        let v = match constants::ACID_REVERSE.get(&key) {
            Some(&v) => v,
            None => {
                println!("something went wrong; wrong ACID key actodna");
                "---" // shouldnt happen
            }
        };
        dna_vec.append(&mut v.chars().collect::<Vec<char>>());
    }
    dna_vec
}

fn byte_fourth(index: usize, byte: u8) -> u8 {
    let c: u8 = 0b00000011;
    c & (byte >> 6 - index)
}

pub fn dna_plus_ambig(dna: &String, ambig_vec: &Vec<u8>, before: bool) -> String {
    let mut string = String::new();
    if dna.is_empty() && ambig_vec.is_empty() {
        return string;
    }
    if ambig_vec.len() * 3 != dna.len() {
        println!("dna+ambig error: length mismatch");
        return string;
    }
    if !before {
        string += dna;
        string.push('-');
        for ambig in ambig_vec.iter() {
            string.push(constants::DNA[ambig]);
        }
    } else {
        for ambig in ambig_vec.iter() {
            string.push(constants::DNA[ambig]);
        }
        string.push('-');
        string += dna;
    }
    string
}

pub fn encrypt(key: &Vec<char>, text: &String, ambig_vec: &mut Vec<u8>) -> String {
    if key.len() != 25 {
        println!("key format error");
        panic!()
    }
    let text = text.to_uppercase();
    let key = key
        .iter()
        .collect::<String>()
        .to_uppercase()
        .chars()
        .collect::<Vec<char>>();

    let mut text = text.chars().collect::<Vec<char>>();
    text.retain(|&c| !c.is_whitespace());
    if text.is_empty() {
        return String::new();
    }

    let mut chunks: Vec<char> = Vec::new();
    let mut index = 0usize;
    let mut offset = 0usize;

    // creating chunks
    while index < text.len() - 1 {
        chunks.push(text[index]);
        if text[index] == text[index + 1] {
            chunks.push('X');
            index += 1;
            ambig_vec.insert(index + offset, 4u8);
            offset += 1;
        } else {
            chunks.push(text[index + 1]);
            index += 2;
        }
    }
    if index == text.len() - 1 {
        chunks.push(text[index]);
        chunks.push('X');
        ambig_vec.push(4u8);
    }

    let mut encrypted: Vec<char> = Vec::with_capacity(chunks.len());

    for k in (0..chunks.len()).step_by(2) {
        let i1: usize = key.iter().position(|&c| c == chunks[k]).unwrap();
        let i2: usize = key.iter().position(|&c| c == chunks[k + 1]).unwrap(); // should not be a
                                                                               // problem unless text
                                                                               // has special or lowercase chars
        let diff = i1.abs_diff(i2);
        if i1 / 5 == i2 / 5 {
            // same row
            encrypted.push(key[wrap_around(i1 / 5 * 5, i1 + 1)]); // modular arithmetic
            encrypted.push(key[wrap_around(i2 / 5 * 5, i2 + 1)]);
        } else if (diff % 5) == 0 {
            // same column
            encrypted.push(key[(i1 + 5) % 25]);
            encrypted.push(key[(i2 + 5) % 25]);
        } else {
            // rectangle
            encrypted.push(key[(i2 % 5) + i1 / 5 * 5]);
            encrypted.push(key[(i1 % 5) + i2 / 5 * 5]);
        }
    }

    encrypted.iter().collect::<String>()
}

pub fn sanitize_acids(acids: &String, ambig: &Vec<u8>) -> String {
    let mut sanitized_acids = String::new();
    if acids.len() != ambig.len() {
        println!("something went wrong; unsanitized_acids/ambig_vec length mismatch");
        return sanitized_acids;
    }
    let unsanitized_acids = acids.chars().collect::<Vec<char>>();
    for (i, byte) in ambig.iter().enumerate() {
        if *byte != 4u8 {
            sanitized_acids.push(unsanitized_acids[i]);
        } else {
            println!("added {}  at index {} \n", unsanitized_acids[i], i);
        }
    }
    sanitized_acids
}

pub fn sanitize_ambig(ambig: &mut Vec<u8>) {
    ambig.retain(|&byte| byte != 4u8);
}

pub fn decrypt(key: &String, text: &String) -> String {
    if key.len() != 25 {
        println!("key format error");
        panic!()
    }
    let text = text.to_uppercase();
    let key = key.to_uppercase().chars().collect::<Vec<char>>();

    let text = text.chars().collect::<Vec<char>>();
    if text.is_empty() {
        return String::new();
    }
    let mut decrypted: Vec<char> = Vec::with_capacity(text.len());

    for k in (0..text.len()).step_by(2) {
        let i1: usize = key.iter().position(|&c| c == text[k]).unwrap();
        let i2: usize = key.iter().position(|&c| c == text[k + 1]).unwrap(); // should not be a
                                                                             // has special or lowercase chars
        let diff = i1.abs_diff(i2);
        if i1 / 5 == i2 / 5 {
            // same row
            decrypted.push(key[wrap_around_l(i1 / 5 * 5, i1, 1) % 25]); // modular arithmetic
            decrypted.push(key[wrap_around_l(i2 / 5 * 5, i2, 1) % 25]);
        } else if (diff % 5) == 0 {
            // same column
            decrypted.push(key[wrap_around_d(i1)]);
            decrypted.push(key[wrap_around_d(i2)]);
        } else {
            // rectangle
            decrypted.push(key[(i2 % 5) + i1 / 5 * 5]);
            decrypted.push(key[(i1 % 5) + i2 / 5 * 5]);
        }
    }

    decrypted.iter().collect::<String>()
}

fn wrap_around(min: usize, value: usize) -> usize {
    return min + (value % 5);
}
fn wrap_around_l(min: usize, value: usize, minus: usize) -> usize {
    if value < minus || value - (minus % 5) < min {
        return value + 5 - (minus % 5);
    } else {
        return value - (minus % 5);
    }
}

fn wrap_around_d(value: usize) -> usize {
    if value < 5 {
        return 20 + value;
    } else {
        return value - 5;
    }
}
