import socket
import time

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
server_address = ('127.0.0.1', 4210)

angle = -180.0
direction = 1.0  
sweep_speed = 6.0 # Increased speed so the turret spins faster!

print("📡 Starting 360° LiDAR Sweep Simulation (-180° to +180°)...")

while True:
    # We still send 0.0 for base_x, base_y, and heading because the 
    # Rust State Machine completely overrides them with its own physical state!
    payload = f"0.0, 0.0, 0.0, {angle:.1f}, 200.0".encode('utf-8')
    sock.sendto(payload, server_address)

    # Mechanical sweep logic
    angle += (direction * sweep_speed)
    
    # Sweep from -180 to +180
    if angle >= 180.0:
        direction = -1.0
    elif angle <= -180.0:
        direction = 1.0

    # 20Hz update rate
    time.sleep(0.05)
