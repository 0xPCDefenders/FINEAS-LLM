use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::process::Command;
use std::string::String;
use percent_encoding::percent_decode_str;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let prompt = if let Some(index) = request.find("prompt=") {
        let start_index = index + 7; // Length of "prompt=" is 7
        let end_index = request[start_index..].find(' ').map(|i| start_index + i).unwrap_or(request.len());
        let encoded_prompt = &request[start_index..end_index];
        let decoded_prompt = percent_decode_str(encoded_prompt).decode_utf8().unwrap().to_string();
        decoded_prompt.replace("+", " ")
    } else {
        "".to_string()
    };

    let cmd_output = Command::new("llm")
        .arg("llama")
        .arg("infer")
        .arg("-m")
        .arg("model/ggml-wizardLM-7B.q4_2.bin")
        .arg("-p")
        .arg(&prompt)
        .output();

    println!("{}", prompt);

    match cmd_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", remove_unwanted_parts(&stdout, prompt.to_string()));
                stream.write(response.as_bytes()).unwrap();
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let response = format!("HTTP/1.1 500 Internal Server Error\r\n\r\nCommand failed with error:\n{}", stderr);
                stream.write(response.as_bytes()).unwrap();
            }
        }
        Err(err) => {
            let response = format!("HTTP/1.1 500 Internal Server Error\r\n\r\nFailed to execute command: {}", err);
            stream.write(response.as_bytes()).unwrap();
        }
    }

    stream.flush().unwrap();
}

fn remove_unwanted_parts(output: &str, prompt: String) -> String {
    let mut lines = output.lines();
    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            // Skip empty lines
            continue;
        }
        if line.starts_with("✓ Loaded") || line.starts_with(&prompt) || line.contains(&prompt) {
            // Skip unwanted lines
            continue;
        }
        // Return the remaining lines as the inference output
        return lines.collect::<Vec<&str>>().join("\n");
    }

    String::new()
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8086").expect("Failed to bind address");
    println!("Server listening on 127.0.0.1:8086");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            handle_client(stream);
        });
    }
}
