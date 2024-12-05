
#[derive(Debug)]
pub struct IcmpV4 {
    icmp_type: u8,
    code: u8,
    check_sum: u16,
    data: Vec<u8>
}

impl IcmpV4 {

    pub fn new(icmp_type: u8, code: u8, data: Vec<u8>) -> Self {
        let mut new_ins = IcmpV4 {icmp_type, code, check_sum: 0, data};
        new_ins.check_sum = Self::generate_checksum(&new_ins.serialized());
        return  new_ins;
    }

    pub fn deserialize(bytes: &Vec<u8>) -> Self {
        IcmpV4 {
            icmp_type: bytes[0],
            code: bytes[1],
            check_sum: ((bytes[2] as u16) << 8) + (bytes[3] as u16),
            data: bytes[4..].to_vec()
        }
    }

    pub fn serialized(&self) -> Vec<u8>{
        let mut result: Vec<u8> = vec![self.icmp_type, self.code, (self.check_sum >> 8) as u8, self.check_sum as u8];
        result.append(&mut self.data.clone());
        return result;
    }

    fn generate_checksum(bytes: &Vec<u8>) -> u16{
        let mut checksum = 0;

        if bytes.len() & 1 == 1 {
            panic!("odd length!");
        }

        for i in (0..bytes.len()).step_by(2) {
            checksum += ((bytes[i] as u32) << 8) + (bytes[i + 1] as u32);
            
            if checksum & 0xffff0000 != 0 { // 处理溢出
                checksum = (checksum & 0x0000ffff) + (checksum >> 16);
            }
        }
        
        checksum as u16
    }

    pub fn check(bytes: &Vec<u8>) -> bool {
        Self::generate_checksum(bytes) == 0
    }

}