use std::collections::BTreeMap;

/**
 * 重组数据流器
 * 将字节流的子串或者小段按照正确顺序来拼接回连续字节流的模块
 * 如果 ByteStream 已满，则必须暂停装配，将未装配数据暂时保存起来
 * |         assembled_window             |<next_to_be_assembled>             unassembled_window              |
 * |                              buffer_window                                                               |
 * 
 */
struct StreamReassembler {
    unassembled_window: BTreeMap<usize, Vec<u8>>,
    assembled_window: Vec<u8>,
    next_to_be_assembled: usize,
    buffer_window_size: usize,
    eof_idx: usize, // EOF
}

impl StreamReassembler{
    pub fn new(buffer_window_size: usize) -> Self {
        StreamReassembler {
            unassembled_window: BTreeMap::new(),
            assembled_window: Vec::new(),
            next_to_be_assembled: 0,
            eof_idx: usize::MAX,
            buffer_window_size,
        }
    }

    /**
     * 返回已经按序接收的数据的引用，但不取出
     */
    pub fn view_assembled(&self) -> &[u8] {
        &self.assembled_window
    }
    /**
     * 返回已经按序接收的数据，并取出
     */
    pub fn get_and_remove_assembled(&mut self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.append(&mut self.assembled_window); // 清空assembled_window
        result
    }

    /**
     * 接收数据, 暂存或者拼接或丢弃
     * 尽可能合并区间，确保缓存区域的区间不重叠
     */
    pub fn recv(&mut self, data: &[u8], offset: usize, eof: bool) {
        let next_idx_from_data: usize = offset + data.len();
        if self.beyond_window(next_idx_from_data - 1) { // 超出窗口，直接返回
            return;
        }

        if offset <= self.next_to_be_assembled { /* 可以并入结果集 */
            self.merge_to_assembled(&data, offset);          
        }
        else { /* 不能并入结果集, 将unassembled缓冲区合并 */
            self.merge_from_unassemble(&data, offset);
        }

        if eof {
            self.eof_idx = self.next_to_be_assembled;
        }
    }

    /**
     * 新分组能加入assembled window
     * 将新一段数据加入assembled window后,对 unassembled 缓冲区的数据的处理
     */
    fn merge_to_assembled(&mut self, data: &[u8], offset: usize) {
        self.assembled_window.extend_from_slice(&data[(self.next_to_be_assembled - offset)..]); // 新添加到assembled段的数据
        self.next_to_be_assembled = data.len() + offset;

        let mut to_remove: Vec<usize> = Vec::new(); // 记录将要从unassembled buff 删除的数据

        /*
            unassembled_window中，每个区间[l, r)
            能参与合并的，只有满足l在 (..,self.next_to_be_assembled], r在(self.next_to_be_assembled, ..)
            被删除: 所有满足l在 (..,self.next_to_be_assembled]
            被删除但不被合并：满足l在 (..,self.next_to_be_assembled）, r 在 (..,self.next_to_be_assembled]
        */
        for (k, v) in self.unassembled_window.range(..=self.next_to_be_assembled) {
            if k + v.len() > self.next_to_be_assembled { // 只可能最多有一个
                self.assembled_window.extend_from_slice(&v[(self.next_to_be_assembled - k)..]);
                self.next_to_be_assembled = k + v.len();
            }
            to_remove.push(*k);
        }
        // 删除重叠的区间
        for key in to_remove {
            self.rm_from_unassembled_buff(key);
        }
    }
    /**
     * 处理区间合并
     */
    fn merge_from_unassemble(&mut self, data: &[u8], offset: usize) {
        let next_idx_from_data = data.len() + offset;
        let mut to_remove: Vec<usize> = Vec::new();
        // merged用于存储合并后的区间
        let mut merged: Vec<u8> = Vec::new();
        let mut merged_st = offset;

        /*
            unassembled_window中，每个区间[l, r)
            l 在 (.., offset):
                合并: l在(.., offset), r在[offset,..)
            上面的合并后，生成合并段[m_l, m_r), m_l=l, m_r=max{r, next_idx_from_data}

            l 在 [m_r, next_idx_from_data]:
                合并, r在 (next_idx_from_data, ..)
        */
        for (k, v) in self.unassembled_window.range(..offset) {
            if k + v.len() >= offset { // 至多有一个
                merged.extend_from_slice(&v);
                merged_st = *k;
                if k + v.len() < next_idx_from_data { 
                    merged.extend_from_slice(&data[(k + v.len() - offset)..]);
                }
            }
        }
        // 若合并后的窗口右侧大于data的右侧, 则不可能存在右边可以与之合并的
        if merged_st + merged.len() <= next_idx_from_data {
            for (k, v) in self.unassembled_window.range((merged_st + merged.len())..=next_idx_from_data) {
                if k + v.len() > next_idx_from_data { // 至多有一个
                    merged.extend_from_slice(&v[(next_idx_from_data - k)..]);
                }
                to_remove.push(*k);
            }
        }

        for key in to_remove {
            self.rm_from_unassembled_buff(key);
        }

        if merged.len() == 0 { // 以上两个合并均没有进行
            merged = data.to_vec();
        }

        self.add_to_unassembled_buff(merged_st, &merged);
        
    }

    fn rm_from_unassembled_buff(&mut self, key: usize) {
        self.unassembled_window.remove(&key);
    }

    fn add_to_unassembled_buff(&mut self, key: usize, val: &[u8]) {
        self.unassembled_window.insert(key, val.to_vec());
    }

    fn beyond_window(&self, last_idx: usize) -> bool {
        last_idx > self.buffer_window_size - self.assembled_window.len() + self.next_to_be_assembled - 1
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_assembly() {
        let mut reassembler = StreamReassembler::new(100);

        // 模拟接收数据
        reassembler.recv(&[1, 2, 3], 0, false);
        reassembler.recv(&[4, 5, 6], 3, false);

        // 验证拼接后的数据
        assert_eq!(reassembler.view_assembled(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_buffer_window_full() {
        let mut reassembler = StreamReassembler::new(10); // 小的缓冲区

        // 接收数据，模拟数据超出窗口
        reassembler.recv(&[0, 1, 2, 3], 0, false);
        reassembler.recv(&[4, 5, 6], 4, false);
        reassembler.recv(&[7, 8, 9, 10], 7, false); // 超过窗口

        // 验证是否被丢弃（缓冲区满了）
        assert_eq!(reassembler.view_assembled(), &[0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_eof_handling() {
        let mut reassembler = StreamReassembler::new(100);

        // 接收数据并设置 EOF
        reassembler.recv(&[1, 2, 3], 0, false);
        reassembler.recv(&[4, 5, 6], 3, true); // EOF 标志

        // 验证拼接后的数据
        assert_eq!(reassembler.view_assembled(), &[1, 2, 3, 4, 5, 6]);

        // 确保 EOF 正确标记
        assert_eq!(reassembler.eof_idx, 6);
    }

    /**
     * 验证接收一系列失序数据
     */
    #[test]
    fn test_merge_unassembled_data() {
        let mut reassembler = StreamReassembler::new(100);

        // 模拟接收分散的数据
        reassembler.recv(&[0, 1, 2], 0, false);  
        reassembler.recv(&[10, 11], 10, false);
        reassembler.recv(&[5, 6], 5, false);  
        reassembler.recv(&[7, 8, 9], 7, false);
        reassembler.recv(&[7, 8, 9], 7, false);
        reassembler.recv(&[3, 4], 3, false);    
        reassembler.recv(&[15, 16], 15, false);  
        reassembler.recv(&[10, 11, 12, 13,14], 10, false);  

        // 验证拼接后的数据
        assert_eq!(reassembler.view_assembled(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    }

    #[test]
    fn test_out_of_window_data() {
        let mut reassembler = StreamReassembler::new(10);

        // 接收数据，模拟超出窗口的数据
        reassembler.recv(&[1, 2, 3], 0, false);
        reassembler.recv(&[4, 5, 6], 10, false); // 超出窗口，应该被丢弃

        // 验证拼接后的数据
        assert_eq!(reassembler.view_assembled(), &[1, 2, 3]);
    }

    #[test]
    fn test_data_merge_with_overlap() {
        let mut reassembler = StreamReassembler::new(100);

        // 模拟接收数据，其中有部分重叠
        reassembler.recv(&[0, 1, 2, 3, 4], 0, false);
        reassembler.recv(&[5, 6], 5, false);  // 重叠部分，数据应正确合并

        // 验证拼接后的数据
        assert_eq!(reassembler.view_assembled(), &[0, 1, 2, 3, 4, 5, 6]);
    }
}
