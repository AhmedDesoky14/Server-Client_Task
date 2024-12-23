use crate::message::{EchoMessage,ClientMessage};
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        
        // Read data from the client
        let bytes_read = self.stream.read(&mut buffer)?;
        if bytes_read == 0 {
            info!("Client disconnected.");
            return Ok(());
        }
        /* 
         * Originally the server was only handling and deserializing EchoMessages
         * so I make it decode ClientMessage in general and match on which type of message
         */
        if let Ok(client_message) = ClientMessage::decode(&buffer[..bytes_read]) {

            match client_message.message{
                //In case of EchoMessage
                Some(crate::message::client_message::Message::EchoMessage(ref echo_message)) => {
                    // Handle EchoMessage logic
                    println!("Received EchoMessage: {}", echo_message.content);
                    info!("Received EchoMessage: {}", echo_message.content);

                    // Create a response EchoMessage to send back
                    let response_echo_message = EchoMessage {
                        content: echo_message.content.clone(),  // Echo the content back
                    };

                    // Wrap the response EchoMessage inside a ServerMessage
                    // In client_tests.rs, test cases generally handle server_messages
                    let server_message = crate::message::ServerMessage {
                        message: Some(crate::message::server_message::Message::EchoMessage(response_echo_message)),
                    };

                    // Serialize ServerMessage
                    let mut payload = Vec::new();
                    server_message.encode(&mut payload).expect("Failed to encode server message");
                    //Echo back the message
                    self.stream.write_all(&payload)?;
                    self.stream.flush()?;
                },
                //In case of AddRequest
                Some(crate::message::client_message::Message::AddRequest(ref add_request)) => {
                    // Handle AddRequest logic
                    println!("Received AddRequest: {} + {}", add_request.a, add_request.b);
                    info!("Received AddRequest: {} + {}", add_request.a, add_request.b);
                    // Compute the sum of 'a' and 'b'
                    let result = add_request.a + add_request.b;
                    // Create an AddResponse with the computed result
                    let add_response = crate::message::AddResponse {
                        result, // Store the computed sum
                    };
                    // Wrap the AddResponse inside a ServerMessage
                    let server_message = crate::message::ServerMessage {
                        message: Some(crate::message::server_message::Message::AddResponse(add_response)),
                    };
                    // Serialize ServerMessage
                    let mut payload = Vec::new();
                    server_message.encode(&mut payload).expect("Failed to encode server message");
                    // Send the encoded AddResponse result back to the client
                    self.stream.write_all(&payload)?;
                    self.stream.flush()?;
                },
                // If no message is found
                None => {
                    println!("Received an empty ClientMessage.");
                    info!("Received an empty ClientMessage.");
                },            
            }
        } else {
            error!("Failed to decode message");
        }
        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {

        let listener = TcpListener::bind(addr)?;

        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    // Runs the server, listening for incoming connections and handling them
    // When server stop, close listening
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);

        // Set the listener to non-blocking mode
        self.listener.set_nonblocking(true)?;

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);

                    // Handle the client request in separate thread
                    let is_running = self.is_running.clone();

                    //-> create new thread and move context to it to run the following
                    thread::spawn(move || {
                        let mut client = Client::new(stream);
                        while is_running.load(Ordering::SeqCst) {
                            if let Err(e) = client.handle() {
                                error!("Error handling client: {}", e);
                                break;
                            }
                        }
                    });

                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
        info!("Server stopped.");
        Ok(())
    }

    // Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
