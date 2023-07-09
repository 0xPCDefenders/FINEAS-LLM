use std::io::{Read, Write};
use std::process::{Command, Output};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let request = match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            String::from_utf8_lossy(&buffer[..bytes_read]).to_string()
        }
        Err(err) => {
            eprintln!("Failed to read stream: {}", err);
            return;
        }
    };
    
    // Extract the value of the "prompt" query parameter
    let prompt = if let Some(index) = request.find("prompt=") {
        let start_index = index + 7; // Length of "prompt=" is 7
        let end_index = request[start_index..].find(' ').map(|i| start_index + i).unwrap_or(request.len());
        &request[start_index..end_index]
    } else {
        ""
    };
    
    // Build the command to execute
    let cmd = format!("llm llama infer -m C://Users//kobbyidun//OneDrive//CompSci//Code//Projects//PHINEAS-LLM//PHINEAS-LLM//llm-server//src//model//ggml-wizardLM-7B.q4_2.bin -p \"{}\"", prompt);
    // Print the command
    println!("{}", cmd);
    
    // Execute the command and capture the output
    let output: Output = match Command::new("cmd")
        .arg("-c")
        .arg(&cmd)
        .output()
    {
        Ok(output) => output,
        //print output.stdout
        Err(err) => {
            eprintln!("Failed to execute command: {}", err);
            return;
        } 
    };
    
    // Extract the output as a UTF-8 string
    let response = String::from_utf8_lossy(&output.stdout);
    
    // Send the response back to the client
    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", response);
    if let Err(err) = stream.write(response.as_bytes()) {
        eprintln!("Failed to write response: {}", err);
    }
    
    if let Err(err) = stream.flush() {
        eprintln!("Failed to flush stream: {}", err);
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind address");
    println!("Server listening on 127.0.0.1:8080");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(err) => {
                eprintln!("Failed to establish connection: {}", err);
            }
        }
    }
}
