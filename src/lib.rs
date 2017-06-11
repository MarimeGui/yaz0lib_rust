extern crate fancy_read;

use std::str::from_utf8;
use std::mem;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Seek;


pub fn decompress<T: Read + Seek>(reader: &mut T) -> Vec<u8> {
    // Checks the Yaz0 Magic Number
    let mut temp: [u8; 4] = [0; 4];
    reader.read_exact(&mut temp[..]).expect("Failed to read");
    let magic_number_str = from_utf8(&temp).unwrap();
    if magic_number_str != "Yaz0" {
        panic!("Magic Number did not match, probably not a Yaz0 compressed file.");
    }

    // Read the output data size
    let output_buffer_size: u32 = fancy_read::read_to_u32_le(reader);

    // Go passed the padding
    reader.seek(SeekFrom::Current(8)).expect("Failed to read");

    // Create the Vector object responsible for holding the output data
    let mut output_data_vector: Vec<u8> = Vec::with_capacity(output_buffer_size as usize);

    // Start reading the actual data
    let mut operations_left: u32 = 0;
    let mut current_code_byte: u8 = 0;

    // While the output vector is not filled ...
    while output_data_vector.len() < output_data_vector.capacity() {
        // If we ran out of usable operations, read a new code byte
        if operations_left == 0 {
            current_code_byte = fancy_read::read_to_u8(reader);
            operations_left = 8;
        }
        // Check bit per bit the current code byte
        if (current_code_byte & 2u8.pow(operations_left - 1)) == 2u8.pow(operations_left - 1) {
            // Straight copy
            let to_copy: u8 = fancy_read::read_to_u8(reader);
            output_data_vector.push(to_copy);
        } else {
            // Compressed data
            // Read two bytes from input file
            let mut bytes: [u8; 2] = [0, 0];
            reader.read_exact(&mut bytes[..]).expect("Failed to read");

            // Calculate where to copy from
            let dist: u32 = (((bytes[0] as u32) & 0xF) << 8) | (bytes[1] as u32);
            let copy_from: u32 = (output_data_vector.len() as u32) - (dist + 1);

            // Calculate how many bytes need to copy
            let mut nb_bytes_to_copy: u32 = (bytes[0] as u32) >> 4;
            if nb_bytes_to_copy == 0 {
                // If needed, read a third byte
                let byte3: u8 = fancy_read::read_to_u8(reader);
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
        operations_left = operations_left - 1
    }
    // Return the output data
    output_data_vector
}

#[test]
fn test_decompress() {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    let input_file_reader = File::open("test_files/input").expect("File not found");
    let mut input_file_buf_reader = BufReader::new(input_file_reader);
    let decompressed_data = decompress(&mut input_file_buf_reader);
    let expected_output_file_reader = File::open("test_files/expected_ouput").expect("File not found");
    let mut expected_output_file_buf_reader = BufReader::new(expected_output_file_reader);
    let correct_decompressed_data = expected_output_file_buf_reader.fill_buf();

    let mut pos = 0;
    for byte in correct_decompressed_data {
        assert_eq!(byte[0], decompressed_data[pos]);
        pos = pos + 1;
    }
}
