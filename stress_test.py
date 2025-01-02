import os
import json
import random
import string
import datetime
import requests
from tqdm import tqdm
from dotenv import load_dotenv

load_dotenv()

# Function to generate random strings
def generate_random_string(size: int) -> str:
    return ''.join(random.choices(string.digits, k=size))

# Function to generate a large random file of 6MB
def generate_file(file_path: str, size_mb: int = 6):
    size_bytes = size_mb * 1024 * 1024  # Convert size to bytes
    with open(file_path, 'w') as f:
        f.write(generate_random_string(size_bytes))

def generate_large_file(file_path: str):
    generate_file(file_path, 6)

def send_log_to_server(url: str, log_data: dict):
    try:
        response = requests.post(url, json=log_data)
        if response.status_code == 200:
            print(f"Log sent successfully: {response.json()}")
        else:
            print(f"Failed to send log: {response.status_code}, {response.text}")
    except requests.exceptions.RequestException as e:
        print(f"Error sending log: {e}")

# Generate a random large file
large_file_path = "large_file.txt"
generate_large_file(large_file_path)

# Create log data
log_data = {
    "timestamp": "",
    "system_name": "test_system",
    "username": "test_user",
    "service_name": "test_service",
    "interaction_type": "file_upload",
    "value": {"file": ""},
}

# Get the log server URL from environment variables
HOST = os.getenv("HOST", "localhost")
PORT = os.getenv("PORT", "3030")

log_server_url = f"http://{HOST}:{PORT}/log"

for i in tqdm(range(10000)):
    now = datetime.datetime.now()
    timestamp = now.strftime("%Y-%m-%dT%H:%M:%S")
    system_name = random.choice(["system1"])
    username = random.choice(["user1", "user2", "user3"])
    log_data["timestamp"] = timestamp
    log_data["system_name"] = system_name
    log_data["username"] = username
    if i % 10 == 0:
        log_data["value"]["file"] = open(large_file_path, 'r').read()
    else:
        # small file
        log_data["value"]["file"] = generate_random_string(1000)
    send_log_to_server(log_server_url, log_data)

# clean up 
os.remove(large_file_path)
