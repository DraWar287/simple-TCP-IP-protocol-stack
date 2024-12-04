
/* 以太网帧, 没设置前导码(7bytes)和起始定界符(1byte) */
#[derive(Debug)]
pub struct EthernetFrame {
    d_mac: [u8; 6],
    s_mac: [u8; 6],
    ether_type: u16,
    payload: Vec<u8>, // 46 ~ 1500 Bytes 
    fcs: u32,
}

impl EthernetFrame {

    pub fn new(d_mac: [u8; 6], s_mac: [u8; 6], ether_type: u16, payload: Vec<u8>) -> Self {
        let mut new_ins =  EthernetFrame {d_mac, s_mac, ether_type, payload, fcs: 0};
        new_ins.fcs = new_ins.generate_fcs();
        return new_ins;
    }

    // 字节流变成EthernetFrame对象
    pub fn deserialize(bytes: &[u8]) -> Self{ 
        let size = bytes.len();

        if size < 64 { // 确保 size 至少大于 14 + 46 + 4 == 64，才能成功解析以太网帧
            panic!("Invalid Ethernet frame: too small");
        }

        let d_mac = match bytes[0..6].try_into() {
            Ok(val) => val,
            Err(e) => panic!("{}", e),
        };
        let s_mac = match bytes[6..12].try_into() {
            Ok(val) => val,
            Err(e) => panic!("{}", e),
        };
        let ether_type = ((bytes[12] as u16) << 8) + (bytes[13] as u16);
        let payload = bytes[14..(size - 4)].to_vec();
        let fcs: u32 = bytes[(size - 4)..].iter().fold(0 , |acc, &x| (acc << 8) + (x as u32));
        

        return EthernetFrame {d_mac, s_mac, ether_type, payload, fcs};
    }

    /**
     * 更新对象的fcs, 并返回
     * 数据: D, fcs: R(r bit), 生成多项式: G(r + 1 bit), 这里r = 32
     * 双方协商 G
     * 模二运算
     * <D, R> 正好被 G 整除
     * R = reminder(D * 2.pow(r) / G)
     * fcs 是余数,它初始是被除数，经过运算逐渐变成最终结果的余数
     */
    pub fn generate_fcs(&self) -> u32 {
        const G: u32 = 0x04C11DB7; // 在以太网中，CRC-32使用的G
        let mut fcs: u32 = 0xffff_ffff;
        let serialzed_frame = self.serialized();
        let d = &serialzed_frame[0..serialzed_frame.len() - 4];

        /* CRC */
        for byte in d {
            fcs ^= (*byte as u32) << 24; // 此8位加上余数作为考虑了前面计算的8位

            for _i in 0..8 { // 遍历每一位
                if fcs & 0x8000_0000 != 0 { // 检查最高位
                    fcs = (fcs << 1) ^ G; // 商上1, 减去除数, 并从被除数多拿1位
                }
                else {
                    fcs <<= 1; // 从被除数多拿1位
                }
            }
        }

        return fcs;
    }
    

    pub fn check_fcs(&self) -> bool {
        // TODO
        self.fcs == self.generate_fcs()
    }

    // 序列化成字节流
    pub fn serialized(&self) -> Vec<u8> {
        let size: usize = 14 + self.payload.len() + 4; 
        let mut nums: Vec<u8> = vec![0; size];  //  存放字节流
        // 将数据从 d_mac、s_mac、ether_type 和 payload 填充到 nums 中
        nums[0..6].copy_from_slice(&self.d_mac[0..6]);
        nums[6..12].copy_from_slice(&self.s_mac[0..6]);
        nums[12..14].copy_from_slice(&[
            (self.ether_type >> 8) as u8,
            (self.ether_type & 0xFF) as u8,
        ]);
        nums[14..(size - 4)].copy_from_slice(&self.payload[0..self.payload.len()]);
        nums[(size - 4)..size].copy_from_slice(&[
            (self.fcs >> 24) as u8,
            (self.fcs >> 16) as u8,
            (self.fcs >> 8) as u8,
            self.fcs as u8,
        ]);

        return nums;
    }
}


/**
 * 单元测试
 */
#[cfg(test)]
mod tests {
    use super::EthernetFrame;

    #[test]
    fn test_new_ethernet() {
        let d_mac: [u8; 6] = [0x12, 0x12, 0x12, 0x12, 0x12, 0x12];
        let s_mac: [u8; 6] = [0x12, 0x12, 0x12, 0x12, 0x12, 0x12];
        let ether_type: u16 = 0x1313;
        let payload = "Hello! I am a test message.Hello! I am a test message.Hello! I am a test message.".as_bytes();
        let new_ins = EthernetFrame::new(d_mac, s_mac, ether_type, payload.to_vec());
        eprintln!("<New Instance>:\n {:?}", new_ins);
        eprintln!("<Result Of CRC>: {}", new_ins.generate_fcs());
        eprintln!("<Serialized>: \n {:?}", new_ins.serialized());

        let new_ins1 = EthernetFrame::deserialize(&new_ins.serialized());
        eprintln!("<Deserialized>: \n{:?}", new_ins1);
        eprintln!("<Result Of CRC>: {}", new_ins1.generate_fcs());
        eprintln!("<Check FCS>: \n {:?}", new_ins1.check_fcs());
    }
}