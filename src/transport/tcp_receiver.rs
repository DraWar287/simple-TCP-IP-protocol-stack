use crate::utils::stream_reassemble::{self, StreamReassembler};

use super::tcp_segment::TcpSegment;
/**
 * 用以接收传入的 TCP segment 并将其转换成用户可读的数据流
 * 告诉发送者ack number, window size, 
 */
struct TcpReceiver{
    initial_seq: u32,
    syn_flag: bool,
    capacity: usize,
    reassembler: stream_reassemble::StreamReassembler
}

impl TcpReceiver {
    pub fn new(initial_seq: u32, capacity: usize) -> Self {
        TcpReceiver {
            initial_seq,
            syn_flag: false,
            capacity,
            reassembler: StreamReassembler::new(capacity)
        }
    }

    /**
     * 每次接收tcp报文段时被调用
     */
    pub fn segment_received(&mut self, segment: &TcpSegment) {
        if self.syn_flag == false { 
            if segment.SYN() == false { // 丢弃非SYN包
                return;
            }
            self.syn_flag = true;
            self.initial_seq = segment.seq;
        }

        let abs_offset: usize = Self::rel_offset_to_abs(self.initial_seq, segment.seq, self.reassembler.assembled_cnt()).try_into().unwrap();
        self.reassembler.recv(&segment.data, abs_offset, segment.FIN());
    }

    fn ack_num(&self) -> u32 {
        Self::abs_offset_to_rel(self.initial_seq, self.reassembler.assembled_cnt()) 
    }

    fn window_size(&self) -> u32 {
        self.reassembler.unassembled_window_size()
    }

    /**
     * 相对偏移转为绝对偏移
     * recent_point: 最近的已经接收了的offset
     */
    fn rel_offset_to_abs(initial_seq: u32, rel_offset: u32, recent_point: u64) -> u64 {
        const U32_RANGE: u64 = 1 << 32;
        
        let offset_this_round: u64  = rel_offset.wrapping_sub(initial_seq) as u64;
        let round_cnt: u64 = recent_point / U32_RANGE;
        let rel_of_recent_point: u64 = recent_point % U32_RANGE;

        if (offset_this_round as u64) >= rel_of_recent_point { // 二者在同一轮
            return offset_this_round + round_cnt * U32_RANGE;
        }
        else { // offset_this_round 在新一轮
            return offset_this_round  + (round_cnt + 1) * U32_RANGE;
        }
    }

    fn abs_offset_to_rel(initial_seq: u32, abs_offset: u64) -> u32{
        initial_seq.wrapping_add((abs_offset % (1 << 32)) as u32)
    }
}
