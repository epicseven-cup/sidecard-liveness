use std::{fs, io};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Incoming, IpAddr, SocketAddr, TcpListener, TcpStream};
use std::ptr::null;
use std::time::Instant;
use httparse::Error;
use serde::Deserialize;

struct SideCard {
    config: SideCardConfig,
    listener: TcpListener,
    last_update: Instant,
}

#[derive(Deserialize)]
struct SideCardConfig {
    control_plan_uri: String,
    outgoing_address: SocketAddr,
    incoming_address: SocketAddr,
    header: String,
}

impl SideCard {
    fn new(config_filename:String) -> Self {
        let config_file_result: io::Result<String> = fs::read_to_string(config_filename);
        let config_content:String = config_file_result.unwrap();
        let config: SideCardConfig = toml::from_str(&config_content).unwrap();

        let listener:TcpListener = TcpListener::bind(config.incoming_address).unwrap();

        SideCard{
            config,
            listener,
            last_update: Instant::now(),
        }
    }

    /*
    Redirect the request from the incoming socket, to the
    */
    fn redirect(&mut self, tcp_stream: &mut TcpStream) {
        let mut socket = TcpStream::connect(self.config.outgoing_address).unwrap();
        // HTTP Header init
        let mut header = [httparse::EMPTY_HEADER; 16];
        let mut request = httparse::Request::new(&mut header);
        let mut request_packet:Vec<u8> = Vec::new();
        tcp_stream.read_to_end(&mut request_packet).expect("TODO: panic message");

        let size = request.parse(&*request_packet).unwrap();

        println!("{}", request.path.unwrap());
        // Sends the request and set it to nonblocking
        socket.set_nonblocking(true).unwrap();
        socket.write_all(&*request_packet).unwrap()
    }

    /*
    Start the SideCard service
     */
    fn start(&mut self) -> std::io::Result<()> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(s) => {
                    self.redirect(s);
                }
                _ => {}
            }
            self.redirect(&mut stream?);
        }
        Ok(())
    }
}