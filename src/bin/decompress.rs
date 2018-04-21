extern crate clap;
extern crate yaz0lib;

use clap::{App, Arg};
use std::fs::File;
use std::io::{BufReader, Write};
use yaz0lib::decompress;

fn main() {
    let matches = App::new("Yaz0 De-compressor")
        .version("1.0")
        .author("Marime Gui")
        .about("Decompresses Yaz0 compressed files")
        .arg(
            Arg::with_name("INPUT")
                .help("File to decompress")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("File name to use for decompressed data")
                .index(2),
        )
        .get_matches();

    let input_file = matches.value_of("INPUT").unwrap();
    let mut input_file_buf_reader =
        BufReader::new(File::open(&input_file).expect("Failed to open file for reading"));
    let output = decompress(&mut input_file_buf_reader).expect("Failed to decompress");
    let output_file_name_default = input_file.to_owned() + ".unyazed";
    let output_file_name = matches
        .value_of("OUTPUT")
        .unwrap_or(&output_file_name_default);
    let mut output_file = File::create(output_file_name).expect("Failed to open file for writing");
    output_file.write_all(&output).expect("Failed to write");
    println!("Data successfully decompressed !");
}
