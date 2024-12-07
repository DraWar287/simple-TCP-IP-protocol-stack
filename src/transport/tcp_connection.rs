use super::tcp;

struct TcpConnection {
    s_ip: u32,
    s_port: u16,
    d_ip: u32,
    d_port: u16
}

impl PartialEq for TcpConnection {
    fn eq(&self, other: &Self) -> bool {
        (self.s_ip, self.s_port, self.d_ip, self.d_port) == (other.s_ip, other.s_port, other.d_ip, other.d_port)
    }
}

impl TcpConnection {
    pub fn new(s_ip: u32, s_port: u16, d_ip: u32, d_port: u16) -> TcpConnection {
        TcpConnection {
            s_ip, s_port, d_ip, d_port
        }
    }

    pub fn connect() {

    }

    pub fn disconnect() {

    }

}

/**
 * 用以接收传入的 TCP segment 并将其转换成用户可读的数据流
 * 告诉发送者ack number, window size, 
 */
struct TcpReceiver{
    initial_seq: u32,
    syn_flag: bool,
    capacity: u16,
    
}

impl TcpReceiver {
    
}