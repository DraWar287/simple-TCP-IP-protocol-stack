
/**
 * 返回校验和(已按位取反)
 */
pub fn generate_checksum(bytes: &Vec<u8>) -> u16{
    let mut checksum = 0;

    if bytes.len() & 1 == 1 {
        panic!("Ethernet header with odd length!");
    }

    for i in (0..bytes.len()).step_by(2) {
        checksum += ((bytes[i] as u32) << 8) + (bytes[i + 1] as u32);
        
        if checksum & 0xffff0000 != 0 { // 处理溢出
            checksum = (checksum & 0x0000ffff) + (checksum >> 16);
        }
    }
    
    !(checksum as u16)
}

pub fn check(bytes: &Vec<u8>) -> bool {
    generate_checksum(bytes) == 0
}