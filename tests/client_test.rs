use embedded_recruitment_task::{
    message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server,
};
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

mod client;

fn setup_server_thread(server: Arc<Server>) -> JoinHandle<()> {
    thread::spawn(move || {
        server.run().expect("Server encountered an error");
    })
}

fn create_server() -> Arc<Server> {
    Arc::new(Server::new("localhost:8080").expect("Failed to start server"))
}

//Test Case 1
#[test]
//#[ignore]
fn test_client_connection() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

//Test Case 2
#[test]
//#[ignore]
fn test_client_echo_message() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the echoed message
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for EchoMessage"
    );

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(
                echo.content, echo_message.content,
                "Echoed message content does not match"
            );
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}

//Test Case 3
#[test]
//#[ignore]
fn test_multiple_echo_messages() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message);

        // Send the message to the server
        assert!(client.send(message).is_ok(), "Failed to send message");

        // Receive the echoed message
        let response = client.receive();
        assert!(
            response.is_ok(),
            "Failed to receive response for EchoMessage"
        );

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(
                    echo.content, message_content,
                    "Echoed message content does not match"
                );
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}
/*
 * Bug: Only the first client receives the echo message and when the second client 
 *      sends the message, it runs for indefinite time when "client.receive()" is called
 *      briefly, the server is only handling the first client in a single thread
 *      while it is connected to it and can't handle more clients.
 * Fix: Whenever a client connects to the server, it creates new thread
 *      and handle the client in this thread
 */

//Test Case 4
#[test]
//#[ignore]
fn test_multiple_clients() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect multiple clients
    let mut clients = vec![
        client::Client::new("localhost", 8080, 1000),
        client::Client::new("localhost", 8080, 1000),
        client::Client::new("localhost", 8080, 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    // Prepare multiple messages
    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    // Send and receive multiple messages for each client
    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        //let mut clients_index = 0;

        for client in clients.iter_mut() {

            //println!("Sending {:?} for client number: {}",message_content,clients_index);

            // Send the message to the server
            assert!(
                client.send(message.clone()).is_ok(),
                "Failed to send message"
            );

            // Receive the echoed message
            let response = client.receive();

            //println!("Received message for client number: {}",clients_index);

            assert!(
                response.is_ok(),
                "Failed to receive response for EchoMessage"
            );

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {

                    //println!("Matching {:?} for client number: {}",echo.content,clients_index);
                    
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            
            }

            //clients_index += 1;

        }
    }

    // Disconnect the clients
    for client in clients.iter_mut() {
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
        );
    }

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}
/*
 * Bug: Server was only handling EchoMessages for clients, any other type of messages would fire error
 * Fix: Server Client handling method was modified to handle in general different ClientMessages
 *      and behaves differently with every type.
 */
//Test Case 5
#[test]
//#[ignore]
fn test_client_add_request() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");
    
    
    // Prepare the message
    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    // Send the message to the server
    assert!(client.send(message).is_ok(), "Failed to send message");

    // Receive the response
    let response = client.receive();
    assert!(
        response.is_ok(),
        "Failed to receive response for AddRequest"
    );

    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {

            println!("Response received: {:?}",add_response.result);

            assert_eq!(
                add_response.result,
                add_request.a + add_request.b,
                "AddResponse result does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );

    println!("Disconnected and server stopped");
}

//Test Case 6 - Test different clients handling with add_request messages
#[test]
//#[ignore]
fn test_multiple_clients_add_request() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    //I think more clients need to be connected, reaching merely upper limit to assure good performence
    // Create and connect multiple clients - 129 clients on my machine, to assure performence
    let clients_num = 50;
    let mut clients = Vec::new();
    for _ in 0..clients_num {
        clients.push(client::Client::new("localhost", 8080, 1000));
    }

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    for client in clients.iter_mut() {
        // Prepare the message
        let mut add_request = AddRequest::default();
        add_request.a = 10;
        add_request.b = 20;
        let message = client_message::Message::AddRequest(add_request.clone());

        // Send the message to the server
        assert!(client.send(message).is_ok(), "Failed to send message");

        // Receive the response
        let response = client.receive();
        assert!(
            response.is_ok(),
            "Failed to receive response for AddRequest"
        );

        match response.unwrap().message {
            Some(server_message::Message::AddResponse(add_response)) => {

                println!("Response received: {:?}",add_response.result);

                assert_eq!(
                    add_response.result,
                    add_request.a + add_request.b,
                    "AddResponse result does not match"
                );
            }
            _ => panic!("Expected AddResponse, but received a different message"),
        }
    }

    // Disconnect the clients
    for client in clients.iter_mut() {
        assert!(
            client.disconnect().is_ok(),
            "Failed to disconnect from the server"
       );
    }
    
    // Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or failed to join"
    );
}
//Test Case 7 - Server stops suddenly, it may send the message but it should not receive anything back
#[test]
//#[ignore]
fn test_server_stops_suddenly() {
    // Set up the server in a separate thread
    let server = create_server();
    let handle = setup_server_thread(server.clone());

    // Create and connect the client
    let mut client = client::Client::new("localhost", 8080, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    // Prepare the message
    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    //Stop the server and wait for thread to finish
    server.stop();
    assert!(
        handle.join().is_ok(),
        "Server thread panicked or to join"
    );

    println!("Server stopped suddenly");

    // Send the echo message to the server 
    assert!(client.send(message).is_ok(), "Failed to send message");
    
    // Call receive and assert that error is expected
    let result = client.receive();
    assert!(result.is_err(), "Expected error result, but got {:?}", result);

    // Disconnect the client
    assert!(
        client.disconnect().is_ok(),
        "Failed to disconnect from the server"
    );

}

//Test Case 8 - try to connect to stopped server
#[test]
//#[ignore]
fn test_connect_to_stopped_server() {
    // Create and connect the client and assert that error is expected
    let mut client = client::Client::new("localhost", 8080, 1000);
    let result = client.connect();
    assert!(result.is_err(), "Expected error result, but got {:?}", result);
}
