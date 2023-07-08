use std::io::{Read, Write};
use std::process::{Command, Output};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    
    let request = String::from_utf8_lossy(&buffer[..]);
    
    // Extract the value of the "prompt" query parameter
    let prompt = if let Some(index) = request.find("prompt=") {
        let start_index = index + 7; // Length of "prompt=" is 7
        let end_index = request[start_index..].find(' ').map(|i| start_index + i).unwrap_or(request.len());
        &request[start_index..end_index]
    } else {
        ""
    };
    
    // Build the command to execute
    let cmd = format!("llm llama infer -m model/ggml-wizardLM-7B.q4_2.bin -p \"{}\"", prompt);
    
    // Execute the command and capture the output
    let output: Output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute command");
    
    // Extract the output as a UTF-8 string
    let response = String::from_utf8_lossy(&output.stdout);
    
    // Send the response back to the client
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", response);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind address");
    println!("Server listening on 127.0.0.1:8080");
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_client(stream);
        });
    }
}
