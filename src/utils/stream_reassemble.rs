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
     * 接收数据, 暂存或者拼接或丢弃
     * 尽可能合并区间，确保缓存区域的区间不重叠
     */
    pub fn recv(&mut self, data: &[u8], st: usize, eof: bool) {
        let next_idx_from_data: usize = st + data.len();
        if(self.beyond_window(next_idx_from_data - 1)) { // 超出窗口，直接返回
            return;
        }

        if st <= self.next_to_be_assembled { /* 可以并入结果集 */
            self.merge_to_assembled(&data, st);          
        }
        else { /* 不能并入结果集, 将unassembled缓冲区合并 */
            self.merge_from_unassemble(&data, st);
        }

        if eof {
            self.eof_idx = self.next_to_be_assembled;
        }
    }

    /**
     * 将新一段数据加入assembled window后,对 unassembled 缓冲区的数据的处理
     */
    fn merge_to_assembled(&mut self, data: &[u8], st: usize) {
        self.assembled_window.extend_from_slice(data);
        self.next_to_be_assembled = data.len() + st;

        while self.unassembled_window.range(..self.next_to_be_assembled).count() > 0 {
            let mut to_remove: Vec<usize> = Vec::new();
            for (k, v) in self.unassembled_window.range(..=self.next_to_be_assembled) {
                if k + v.len() > self.next_to_be_assembled { // 只可能最多有一个
                    self.assembled_window.extend_from_slice(&v[self.next_to_be_assembled..(k + v.len())]);
                    self.next_to_be_assembled = k + v.len();
                }
                to_remove.push(*k);
            }
            // 删除重叠的区间
            for key in to_remove {
                self.rm_from_unassembled_buff(key);
            }
        }
    }
    /**
     * 处理区间合并
     */
    fn merge_from_unassemble(&mut self, data: &[u8], st: usize) {
        let next_idx_from_data = data.len() + st;
        let mut to_remove: Vec<usize> = Vec::new();
        let mut merged: Vec<u8> = Vec::new();
        let mut merged_st = st;

        for (k, v) in self.unassembled_window.range(..st) {
            if k + v.len() > st { // 至多有一个
                merged.extend_from_slice(&v);
                merged_st = *k;
                if k + v.len() < next_idx_from_data { // <1>
                    merged.extend_from_slice(&data[(k + v.len())..next_idx_from_data]);
                }
            }
        }
        
        // 若<1>满足, 下面的循环不会执行
        for (k, v) in self.unassembled_window.range((merged_st + merged.len())..next_idx_from_data) {
            if k + v.len() > next_idx_from_data { // 至多有一个
                merged.extend_from_slice(&v[next_idx_from_data..]);
            }
            to_remove.push(*k);
        }

        for key in to_remove {
            self.rm_from_unassembled_buff(key);
        }

        self.next_to_be_assembled += merged.len();
        self.add_to_unassembled_buff(merged_st, &merged);
        
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