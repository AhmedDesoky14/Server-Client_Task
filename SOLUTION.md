# Solution

Here you can document all bugs and design flaws.
---------------------------------------------------------------------------------------------
Bugs:
    1- Server can't handle more than 1 connected client at a time.
    2- Server can only handle EchoMessage type, nothing else.
    3- After applying multithreading for clients handling, server can't handle more than 129 connected client at a time. It can't be considered a bug but a limitation. Tested on my Ubuntu virtual machine.
    4- Client receives messages in blocking mode, using "read()" in blocking mode. This may cause hanging if server is shutdown suddenly or unreachable.

Modifications:
    1- In "server.rs" file, "server" structure, "run" function is modified to apply Rust multithreading to create a seperate thread whenever new client is connected to the server to be handled. To fix bug #1.
    2- In "server.rs" file, "Client" structure, "handle" function is modified to handle all ClientMessages and its messages types in general. To fix bug #2.
    3- In "client.rs" file, "Client" structure, "connect" function is modified. Only 1 line is added to set "read" call to use timeout as set. To fix bug #4

    I couldn't fix bug #3

New Test cases:
    - Test Case #6: This test case is created to measure clients handling performence by the server and its upper limit. And to test multiple clients adding requests.
    - Test Case #7: This test case is created to test client behavior when the server stops suddenly or becomes unreachable.
    - Test Case #8: This test case is created to test client behaviors when it tries to connect to offline server


# Please just run this command to get all test cases done at once

- cargo test -- --test-threads=1
