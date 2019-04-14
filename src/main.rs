mod huffman;

use std::env;
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read};

fn main() -> std::io::Result<()> {
    // Get the file path from the first command line argument
    let arg_error = Error::new(ErrorKind::Other, "Not enough command line arguments");
    let file_path = env::args().nth(1).ok_or(arg_error)?;

    // Read the content of the file
    let file = File::open(file_path)?;
    let mut buf_reader = BufReader::new(file);
    let mut content = Vec::new();
    buf_reader.read_to_end(&mut content)?;

    // When the text given is empty
    let content_error = Error::new(ErrorKind::Other, "Encoding error");
    let (key, encoded) = huffman::encode(&content).ok_or(content_error)?;
    let decoded = huffman::decode(&key, &encoded);

    // println!("encoded: {:?}", encoded);
    // println!("decoded: {:?}", decoded);

    // if let Some(Ok(x)) = Option::map(decoded, String::from_utf8) {
    //     println!("{}", x);
    // }

    let content_size = content.len();
    let encoded_size = encoded.len();

    println!(
        "Size reduction: {} => {} ({}%)",
        content_size,
        encoded_size,
        100. * encoded_size as f32 / content_size as f32
    );

    Ok(())
}
