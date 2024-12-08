use super::tcp_segment;

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

