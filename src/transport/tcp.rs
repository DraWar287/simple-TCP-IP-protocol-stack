
use std::result;

use crate::utils::checksum;
use crate::utils::trans_bytes;

#[derive(Debug)]
struct TcpSegment {
    s_port: u16, d_port: u16,
    seq: u32,
    ack: u32,
    hl: u8/* 长度4bits, 单位32bits*/, rcvd: u8/* 长度4bits*/, ctrl: u8, win_size: u16,
    checksum: u16, ur_ptr: u16,
    options: Vec<u32>,
    data: Vec<u8> 
}

impl TcpSegment {
    pub fn new(s_port: u16, d_port: u16, seq: u32, ack: u32, hl: u8, rcvd: u8, ctrl: u8, win_size: u16, ur_ptr: u16, options: Vec<u32>, data: Vec<u8> ) -> Self {
        let mut new_ins = TcpSegment {s_port, d_port, seq, ack, hl, rcvd, ctrl, win_size, ur_ptr, options, data, checksum: 0 };
        new_ins.checksum = checksum::generate_checksum(&new_ins.serialized_hdr());
        
        new_ins
    }

    pub fn deserialize(bytes: &Vec<u8>) -> Self {
        let h_bytes: usize = (((bytes[12] >> 4) as u32) * 4).try_into().unwrap();
        TcpSegment {
            s_port: trans_bytes::bytes_vec_to_muilt_bytes(&bytes[0..=1]) as u16, d_port: trans_bytes::bytes_vec_to_muilt_bytes(&bytes[2..=3]) as u16,
            seq: trans_bytes::bytes_vec_to_muilt_bytes(&bytes[4..=7]) as u32,
            ack: trans_bytes::bytes_vec_to_muilt_bytes(&bytes[8..=11]) as u32,
            hl: bytes[12] >> 4, rcvd: bytes[12] & 0x0f, ctrl: bytes[13], win_size: trans_bytes::bytes_vec_to_muilt_bytes(&bytes[14..=15]) as u16,
            checksum: trans_bytes::bytes_vec_to_muilt_bytes(&bytes[16..=17]) as u16, ur_ptr: trans_bytes::bytes_vec_to_muilt_bytes(&bytes[18..=19]) as u16,
            options: trans_bytes::bytes_vec_to_muilt_bytes_vec_u32(&bytes[20..h_bytes]),
            data: bytes[h_bytes..].to_vec()
        }
    }

    pub fn serialized_hdr(&self) -> Vec<u8> {
        let mut bytes = vec![
            (self.s_port >> 8) as u8, self.s_port as u8, (self.d_port >> 8) as u8, self.d_port as u8, 
            (self.seq >> 24) as u8, (self.seq >> 16) as u8, (self.seq >> 8) as u8, self.seq as u8, 
            (self.ack >> 24) as u8, (self.ack >> 16) as u8, (self.ack >> 8) as u8, self.ack as u8, 
            ((self.hl << 4) & 0xf0) + (self.rcvd & 0x0f), self.ctrl, (self.win_size >> 8) as u8, self.win_size as u8,
            (self.checksum >> 8) as u8, self.checksum as u8, (self.ur_ptr >> 8) as u8, self.ur_ptr as u8
        ];
        bytes.append(&mut trans_bytes::multi_bytes_vec_to_bytes_vec(&self.options));

        return bytes;
    }

    pub fn serialized(&self) -> Vec<u8> {
        let mut result: Vec<u8> = self.serialized_hdr();
        result.append(&mut self.data.clone());
        
        result
    }
}


#[cfg(test)]
mod tests {
    use crate::utils::trans_bytes;
    use super::*;

    #[test]
    fn test_serialize() {
        // 先定义一个 TcpSegment 实例
        let segment = TcpSegment::new(
            12345,          // 源端口
            80,             // 目标端口
            1001,           // 序列号
            2002,           // 确认号
            5,              // 头部长度 (HL)
            0,              // 保留字段 (RCVD)
            0x18,           // 控制位 (比如 SYN + ACK)
            4096,           // 窗口大小
            0,              // 紧急指针
            vec![],     // 假设选项字段为空
            vec![1, 2, 3, 4],  // 数据字段 (示例数据)
        );

        // 生成该段的序列化字节
        let serialized = segment.serialized();

        // 验证源端口 (0x3039 => 12345)
        assert_eq!(serialized[0], 0x30);
        assert_eq!(serialized[1], 0x39);

        // 验证目标端口 (0x0050 => 80)
        assert_eq!(serialized[2], 0x00);
        assert_eq!(serialized[3], 0x50);

        // 验证序列号 (0x000003f9 => 1001)
        assert_eq!(serialized[4], 0x00);
        assert_eq!(serialized[5], 0x00);
        assert_eq!(serialized[6], 0x03);
        assert_eq!(serialized[7], 0xe9);

        // 验证确认号 (0x000007d2 => 2002)
        assert_eq!(serialized[8], 0x00);
        assert_eq!(serialized[9], 0x00);
        assert_eq!(serialized[10], 0x07);
        assert_eq!(serialized[11], 0xd2);

        // 验证头部长度
        assert_eq!(serialized[12] >> 4, 5);  
        assert_eq!(serialized[12] & 0x0f, 0);

        // 验证窗口大小 (0x1000 => 4096)
        assert_eq!(serialized[14], 0x10);
        assert_eq!(serialized[15], 0x00);


        // 验证紧急指针 (0)
        assert_eq!(serialized[18], 0x00);
        assert_eq!(serialized[19], 0x00);

        // 验证数据字段
        assert_eq!(serialized[20..], vec![1, 2, 3, 4]);

        // 反序列化字节数据
        let deserialized = TcpSegment::deserialize(&serialized);

        // 验证反序列化后的数据是否与原始数据相同
        assert_eq!(deserialized.s_port, segment.s_port);
        assert_eq!(deserialized.d_port, segment.d_port);
        assert_eq!(deserialized.seq, segment.seq);
        assert_eq!(deserialized.ack, segment.ack);
        assert_eq!(deserialized.hl, segment.hl);
        assert_eq!(deserialized.rcvd, segment.rcvd);
        assert_eq!(deserialized.ctrl, segment.ctrl);
        assert_eq!(deserialized.win_size, segment.win_size);
        assert_eq!(deserialized.checksum, segment.checksum);
        assert_eq!(deserialized.ur_ptr, segment.ur_ptr);
        assert_eq!(deserialized.options, segment.options);
        assert_eq!(deserialized.data, segment.data);

    }
}

/*
+------------------------------------------------------------------------+  
|  源端口号 (16 bits)  | 目标端口号 (16 bits)                            |  
+------------------------------------------------------------------------+  
|                            序列号 (32 bits)                            |  
+------------------------------------------------------------------------+  
|                            确认号 (32 bits)                            |  
+------------------------------------------------------------------------+  
| 头部长度 (4 bits) | 保留 (3 bits) |控制位 (9 bits)| 窗口大小 (16 bits)|  
+------------------------------------------------------------------------+  
|                            窗口大小 (16 bits)                          |  
+------------------------------------------------------------------------+  
|                     校验和 (16 bits) | 紧急指针 (16 bits)              |  
+------------------------------------------------------------------------+  
| 可选字段（可变长度，填充字节保证 32 位对齐）                           |  
+------------------------------------------------------------------------+  
| 数据部分（可变长度，填充字节保证 32 位对齐）                           |  
+------------------------------------------------------------------------+  
*/