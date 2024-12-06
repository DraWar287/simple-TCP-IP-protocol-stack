use crate::utils::checksum;

#[derive(Debug)]
pub struct Ipv4Datagram {
    version: u8, // 4bits
    ihl: u8,     // 4bits, 单位32bits
    tos: u8,
    toltal_len: u16,
    id: u16,
    flag: u8,         // 3bits
    frag_offset: u16, // 13bits
    ttl: u8,
    protocol: u8,
    hdr_checksum: u16,
    s_addr: u32,
    d_addr: u32,
    // 省略options字段
    // 省略padding, 字节流中给头部字段补齐到 32bits 的倍数
    payload: Vec<u8>, // 载荷
}

impl Ipv4Datagram {
    // 静态方法
    /**   
     * 传入除了校验和以外的所有字段
     */
    pub fn new(version: u8, ihl: u8, tos: u8, toltal_len: u16, id: u16, flag: u8, frag_offset: u16, ttl: u8, protocol: u8,  s_addr: u32, d_addr: u32, payload: Vec<u8>) -> Self{
       let mut new_ins =  Ipv4Datagram {version, ihl, tos, toltal_len, id, flag, frag_offset, ttl, protocol, hdr_checksum: 0, s_addr, d_addr, payload };
       new_ins.generate_hdr_checksum();
       return new_ins;
    }


    pub fn deserialize(bytes:Vec<u8>) -> Ipv4Datagram{
        if bytes.len() < 20 { // IPv4头部的最小长度为20字节
            panic!("Invalid IPv4 datagram: too short (should be longer than 20Bytes)");
        }

        let version: u8 = bytes[0] >> 4;
        let ihl: u8 = bytes[0] & 0x0f;
        let tos: u8 = bytes[1];
        let toltal_len: u16 = ((bytes[2] as u16) << 8) + (bytes[3] as u16);
        let id: u16 =  ((bytes[4] as u16) << 8) + (bytes[5] as u16);
        let flag: u8 = bytes[6] >> 5;
        let frag_offset: u16 = (((bytes[6] as u16) & 0b00011111) << 8) + (bytes[7] as u16);
        let ttl: u8 = bytes[8];
        let protocol: u8 = bytes[9];
        let hdr_checksum: u16 = ((bytes[10] as u16) << 8) + (bytes[11] as u16);
        let s_addr: u32 = ((bytes[12] as u32) << 24) + ((bytes[13] as u32) << 16) + ((bytes[14] as u32) << 8) + (bytes[15] as u32);
        let d_addr: u32 = ((bytes[16] as u32) << 24) + ((bytes[17] as u32) << 16) + ((bytes[18] as u32) << 8) + (bytes[19] as u32);
        let payload :Vec<u8>= bytes[20..].to_vec();

        Ipv4Datagram {version, ihl, tos, toltal_len, id, flag, frag_offset, ttl, protocol, hdr_checksum, s_addr, d_addr, payload }
    }

    // 成员方法

    fn generate_hdr_checksum(&mut self) -> u16 {
        self.hdr_checksum = 0;
        let serialized_hdr = self.serialized_hdr();
        let checksum =  checksum::generate_checksum(&serialized_hdr);
        self.hdr_checksum = checksum;
        
        checksum
    }

    pub fn serialized_hdr(&self) -> Vec<u8> {
        vec![(self.version << 4) + (self.ihl), 
             self.tos, 
             (self.toltal_len >> 8) as u8, self.toltal_len as u8, 
             (self.id >> 8) as u8, self.id as u8, 
             (self.flag << 5) + ((self.frag_offset >> 10) as u8), self.frag_offset as u8,
             self.ttl,
             self.protocol,
             (self.hdr_checksum >> 8) as u8, self.hdr_checksum as u8,
             (self.s_addr >> 24) as u8, (self.s_addr >> 16) as u8, (self.s_addr >> 8) as u8, self.s_addr as u8]
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    // 测试反序列化功能
    #[test]
    fn test_deserialize_valid_ipv4() {
        let bytes: Vec<u8> = vec![
            0x45, // version, ihl
            0x00, // tos
            0x00, 0x3c, // toltal_len
            0x1c, 0x46, // id
            0b00000100, 0x00, // flag, frag_offset
            0x40, // ttl
            0x06, // protocol
            0x7a, 0x7a, // checksum
            0x0a, 0x00, 0x00, 0x01, // s_addr
            0x0a, 0x00, 0x00, 0x02, // d_addr
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let datagram = Ipv4Datagram::deserialize(bytes);
        // 测试字段的正确性
        assert_eq!(datagram.version, 4);
        assert_eq!(datagram.ihl, 5); 
        assert_eq!(datagram.tos, 0);
        assert_eq!(datagram.toltal_len, 60);
        assert_eq!(datagram.id, 0x1c46);
        assert_eq!(datagram.flag, 0);
        assert_eq!(datagram.frag_offset, 1024);
        assert_eq!(datagram.ttl, 64);
        assert_eq!(datagram.protocol, 6); // TCP
        assert_eq!(datagram.s_addr, 0x0a000001); // 10.0.0.1
        assert_eq!(datagram.d_addr, 0x0a000002); // 10.0.0.2
    }


    // 测试校验和计算逻辑
    #[test]
    fn test_generate_checksum_valid() {
        let bytes: Vec<u8> = vec![
            0x50, 0x00, 0xb0, 0x3c, 0x50, 0x00, 0xb0, 0x3c,0x50, 0x00, 0xb0, 0x3c,0x50, 0x00, 0xb0, 0x3c,0x50, 0x00, 0xb0, 0x3c,
        ];

        let checksum = checksum::generate_checksum(&bytes);
        assert_eq!(checksum, 0xFECE); // 预期的校验和
    }

}
