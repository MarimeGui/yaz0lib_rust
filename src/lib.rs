extern crate ez_io;

use ez_io::ReadE;
use std::io::SeekFrom;
use std::io::{Read, Seek};
use std::error::Error;

pub fn decompress<R: Read + Seek>(reader: &mut R) -> Result<Vec<u8>, Box<Error>> {
    // Checks the Yaz0 Magic Number
    let mut magic_number: [u8; 4] = [0; 4];
    reader.read_exact(&mut temp)?;
    assert_eq!(magic_number, [b'Y', b'A', b'Z', b'0'], "Magic Number did not match");

    // Read the output data size
    let output_buffer_size: u32 = reader.read_be_to_u32()?;

    // Go passed the padding
    reader.seek(SeekFrom::Current(8))?;

    // Create the Vector object responsible for holding the output data
    let mut output_data_vector: Vec<u8> = Vec::with_capacity(output_buffer_size as usize);

    // Start reading the actual data
    let mut operations_left: u32 = 0;
    let mut current_code_byte: u8 = 0;

    // While the output vector is not filled ...
    while output_data_vector.len() < output_data_vector.capacity() {
        // If we ran out of usable operations, read a new code byte
        if operations_left == 0 {
            current_code_byte = reader.read_to_u8();
            operations_left = 8;
        }
        // Check bit per bit the current code byte
        if (current_code_byte & 2u8.pow(operations_left - 1)) == 2u8.pow(operations_left - 1) {
            // Straight copy
            let to_copy: u8 = reader.read_to_u8()?;
            output_data_vector.push(to_copy);
        } else {
            // Compressed data
            // Read two bytes from input file
            let mut bytes: [u8; 2] = [0, 0];
            reader.read_exact(&mut bytes[..])?;

            // Calculate where to copy from
            let dist: u32 = (((bytes[0] as u32) & 0xF) << 8) | (bytes[1] as u32);
            let copy_from: u32 = (output_data_vector.len() as u32) - (dist + 1);

            // Calculate how many bytes need to copy
            let mut nb_bytes_to_copy: u32 = (bytes[0] as u32) >> 4;
            if nb_bytes_to_copy == 0 {
                // If needed, read a third byte
                let byte3: u8 = reader.read_to_u8()?;
                nb_bytes_to_copy = (byte3 as u32) + 0x12;
            } else {
                nb_bytes_to_copy = nb_bytes_to_copy + 2;
            }

            // Copy the data
            for i in 0..nb_bytes_to_copy {
                let to_copy: u8 = output_data_vector[((copy_from + i) as usize)];
                output_data_vector.push(to_copy);
            }
        }
        operations_left = operations_left - 1;
    }
    // Return the output data
    Ok(output_data_vector)
}

#[test]
fn test_decompress() {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    // Open input file
    let input_file_reader = File::open("test_files/input").expect("File not found");
    let mut input_file_buf_reader = BufReader::new(input_file_reader);

    // Decompress it
    let decompressed_data = decompress(&mut input_file_buf_reader).unwrap();

    // Open expected results file
    let expected_output_file_reader = File::open("test_files/expected_output").expect("File not found");
    let mut expected_output_file_buf_reader = BufReader::new(expected_output_file_reader);

    // Load it all into memory
    let correct_decompressed_data = expected_output_file_buf_reader.fill_buf().unwrap();

    // Check for any different byte
    let mut pos = 0;
    for byte in correct_decompressed_data {
        assert_eq!(byte[0], decompressed_data[pos]);
        pos = pos + 1;
    }
}
