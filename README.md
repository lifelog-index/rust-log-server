## Overview

A simple log server written in Rust. For research purposes.

## Install Rust

```bash
curl https://sh.rustup.rs -sSf | sh
```

## Usage

Edit the `.env` file to set the server address and port.

Start the server using `cargo run`

Send a post request to the server with the log message in the body.

Example:

```bash
curl -X POST http://localhost:8080 -d '{
    "timestamp": "2021-01-01T00:00:00Z",
    "system_name": "example_system",
    "username": "test_user",
    "service_name": "example_service",
    "interaction_type": "test_interaction",
    "value": {"key": "A very long string here that exceeds 5MB to test artifact separation"}
}'
```

Example output directory:

```
example_system/
├── test_user/
│   ├── log_1.log
│   ├── log_2.log
│   ├── ...
│   └── artifacts/
│       └── artifact_####.txt
```


For more information, see the stress test script `stress_test.py` written in Python.

