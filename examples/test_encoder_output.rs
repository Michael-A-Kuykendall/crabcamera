//! Test openh264 output format
use openh264::encoder::Encoder;
use openh264::formats::YUVBuffer;

fn main() {
    let mut encoder = Encoder::new().unwrap();
    
    // Create a simple test frame
    let width = 320usize;
    let height = 240usize;
    let y_size = width * height;
    let uv_size = y_size / 4;
    
    let yuv = vec![128u8; y_size + uv_size * 2];
    
    let yuv_buf = YUVBuffer::from_vec(yuv, width, height);
    let bs = encoder.encode(&yuv_buf).unwrap();
    
    let data = bs.to_vec();
    
    println!("Encoded {} bytes", data.len());
    println!("First 32 bytes: {:02x?}", &data[..data.len().min(32)]);
    
    // Check for start codes
    let mut i = 0;
    while i < data.len().saturating_sub(4) {
        if data[i..i+4] == [0, 0, 0, 1] {
            let nal_type = data[i+4] & 0x1f;
            println!("4-byte start code at {}: NAL type {}", i, nal_type);
            i += 4;
        } else if i + 3 <= data.len() && data[i..i+3] == [0, 0, 1] {
            let nal_type = data[i+3] & 0x1f;
            println!("3-byte start code at {}: NAL type {}", i, nal_type);
            i += 3;
        } else {
            i += 1;
        }
    }
}
