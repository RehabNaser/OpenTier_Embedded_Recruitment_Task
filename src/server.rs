use crate::message::*; // Import the module containing messages
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        Mutex, // Mutual exclusion
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
    is_running: Arc<Mutex<AtomicBool>>, // Reference to the server's is_running flag wrapped in Arc<Mutex>
}

impl Client {
    pub fn new(stream: TcpStream, is_running: Arc<Mutex<AtomicBool>>) -> Self {
        Client { stream, is_running  } // Initialize with the TCP stream and the shared is_running flag
    }

    pub fn handle(&mut self) {
        let mut buffer = [0; 512]; // Create a buffer to store incoming data
        // Enter a loop to continuously handle client messages
        loop{
            // Check if the server is still running
            {
                let is_running = self.is_running.lock().unwrap(); // Lock the `is_running` flag to check its status
                if !is_running.load(Ordering::SeqCst) {  // If the server is shutting down, exit the loop
                    info!("Server is shutting down. Closing client connection.");
                    break;
                }
            }   

            // Attempt to read data from the client's stream         
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    info!("Client disconnected."); // If 0 bytes are read, the client has disconnected,  so exit the loop  
                    break;
                }
                Ok(bytes_read) => {
                    // Decode the incoming message from the buffer
                    match ClientMessage::decode(&buffer[..bytes_read]) {
                        Ok(ClientMessage {
                            message: Some(client_message::Message::AddRequest(add_request)),
                        }) => {
                            // Handle AddRequest messages
                            info!("Received AddRequest: a={}, b={}",add_request.a, add_request.b); // Log the request
                            let result = add_request.a + add_request.b; // Perform the addition operation
                            // Create the response with the result
                            let response = ServerMessage {
                                message: Some(server_message::Message::AddResponse(AddResponse {
                                    result, 
                                })),
                            };
                             // Encode the response and send it back to the client
                            let payload = response.encode_to_vec();
                            if let Err(e) = self.stream.write_all(&payload) { // Handle any write errors
                                error!("Error sending response: {}", e);
                                break;
                            }
                            if let Err(e) = self.stream.flush() { // Ensure the data is flushed to the stream
                                error!("Error flushing stream: {}", e);
                                break;
                            }
                        }
                        // Handle EchoMessage messages
                        Ok(ClientMessage {
                            message: Some(client_message::Message::EchoMessage(echo_message)),
                        }) => {
                            // Process EchoMessage
                            info!("Received EchoMessage: {}", echo_message.content); // Log the received message
                             // Create the echo response
                            let response = ServerMessage {
                                message: Some(server_message::Message::EchoMessage(EchoMessage {
                                    content: echo_message.content.clone(), // Echo back the same content
                                })),
                            };

                            // Encode the response and send it back to the client
                            let payload = response.encode_to_vec();
                            if let Err(e) = self.stream.write_all(&payload) { // Handle any write errors
                                error!("Error sending response: {}", e);
                                break;
                            }
                            if let Err(e) = self.stream.flush() { // Ensure the data is flushed to the stream
                                error!("Error flushing stream: {}", e);
                                break;
                            }
                        }
                        // Log and ignore unknown message types
                        Ok(_) => {
                            warn!("Received unknown message type.");
                        }
                        // Handle decoding errors
                        Err(e) => {
                            error!("Failed to decode message: {}", e);
                        }
                    }
                }
                 // Handle cases where no data is available yet
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available, just return and retry later
                    thread::sleep(Duration::from_millis(100)); // Sleep briefly to avoid busy waiting
                }
                // Handle unexpected errors while reading from the stream
                Err(e) => {
                    error!("Unexpected error while reading: {}", e);
                    break;
                }
            }
        }
    }
}

pub struct Server {
    listener: TcpListener,
   is_running: Arc<Mutex<AtomicBool>>, // Wrap `AtomicBool` in a `Mutex` so you can lock it for safe access across threads
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(Mutex::new(AtomicBool::new(false))); // Initialize the is_running flag with a Mutex
        Ok(Server {
            listener,
            is_running,
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        {
            let is_running = self.is_running.lock().unwrap(); // Lock the Mutex to access is_running
            is_running.store(true, Ordering::SeqCst); // Mark the server as running
        }
        info!("Server is running on {}", self.listener.local_addr()?);
        
        self.listener.set_nonblocking(true)?; // Set the listener to non-blocking mode

        while {
            let is_running = self.is_running.lock().unwrap(); // Lock the Mutex to check is_running
            is_running.load(Ordering::SeqCst) // Read the value inside the Mutex to continue the loop if the server is running
        } {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr); // log the new client address
                    let is_running_clone = Arc::clone(&self.is_running); // Clone the `is_running` Arc to pass a reference to the new thread safely
                    // Spawn a new thread to handle the client independently
                    thread::spawn(move || {
                        let mut client = Client::new(stream, is_running_clone);  // Create a new Client instance, passing the stream and the cloned `is_running` reference
                        client.handle(); // Call the `handle` method to process the client's requests in the separate thread
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

    /// Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        let is_running = self.is_running.lock().unwrap(); // Acquire a lock on the Mutex to safely access the `is_running` flag
        if is_running.load(Ordering::SeqCst) {
            is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
