use std::{mem, vec};
use std::any::TypeId;

/**
 * 多字节数，多字节数组转为单字节数组
 */


pub fn multi_bytes_to_bytes_vec<T>(num: T) -> Vec<u8>
where
    T: Copy + Into<u64>,  // 限制 T 可以转换为 u64
{
    let size = mem::size_of::<T>();  // 这里固定为 u64 的字节大小

    let mut bytes: Vec<u8> = vec![0; size];  // 创建一个大小为 8 字节的空 Vec<u8>
    let num_u64: u64 = num.into();  // 将 num 转换为 u64，避免越界
    // 将 num 转换为字节
    for i in 0..size {
        bytes[i] = (num_u64 >> ((size - 1 - i)* 8)) as u8;  // 按字节拆解
    }

    bytes
}


pub fn multi_bytes_vec_to_bytes_vec<T>(nums: &Vec<T>) -> Vec<u8> 
where 
    T: Copy + Into<u64>
{
    nums.iter().fold(vec![], |mut acc, num| {
        acc.append(&mut multi_bytes_to_bytes_vec(*num));
        acc
    })
}

pub fn bytes_vec_to_muilt_bytes(bytes: &[u8]) -> u64{
    bytes.iter().fold(0 as u64, |acc: u64, byte: &u8| {
        (acc << 8) + (*byte as u64)
    })
}

macro_rules! bytes_vec_to_muilt_bytes_vec {
    ($type:ty, $func_name:ident) => {
        pub fn $func_name(bytes: &[u8]) -> Vec<$type>{
            let size = mem::size_of::<$type>();
                let mut result: Vec<$type> = Vec::new();
                let len = bytes.len();
        
                for i in (0..(len - (len % size))).step_by(size) {
                    result.push(bytes_vec_to_muilt_bytes(&bytes[i..i + size]) as $type);
                }
        
                result
        }
    };
}
bytes_vec_to_muilt_bytes_vec!(u8, bytes_vec_to_muilt_bytes_vec_u8);
bytes_vec_to_muilt_bytes_vec!(u16, bytes_vec_to_muilt_bytes_vec_u16);
bytes_vec_to_muilt_bytes_vec!(u32, bytes_vec_to_muilt_bytes_vec_u32);
bytes_vec_to_muilt_bytes_vec!(u64, bytes_vec_to_muilt_bytes_vec_u64);

mod tests {
    use crate::utils::trans_bytes;

    #[test]
    fn test_trans_to_muilt() {
        assert_eq!(trans_bytes::multi_bytes_to_bytes_vec(1 as u64), vec![0, 0, 0, 0, 0 , 0, 0, 1]);
        assert_eq!(trans_bytes::multi_bytes_vec_to_bytes_vec(&vec![1 as u64, 1 as u64]), vec![0, 0, 0, 0, 0 , 0, 0, 1, 0, 0, 0, 0, 0 , 0, 0, 1])
    }

    #[test]
    fn test_muilt_trans_to() {
        assert_eq!(trans_bytes::bytes_vec_to_muilt_bytes(&[1 as u8, 0 as u8]) as u16, 0x0100);
        assert_eq!(trans_bytes::bytes_vec_to_muilt_bytes_vec_u32(&[1,0,1,0]), vec![0x01000100]);
    }
}