import socket
import time

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
server_address = ('127.0.0.1', 4210)

angle = -60.0
direction = 1.0  # 1 for increasing angle, -1 for decreasing
sweep_speed = 1.0 # Degrees per tick

print("📡 Starting Radar Sweep Simulation (-60° to +60°)...")

while True:
    # base_x=0, base_y=0, heading=0, scan_angle=angle, max_dist=200
    # max_dist is 200cm so the ray travels far enough to hit our blue hourglass
    payload = f"0.0, 0.0, 0.0, {angle:.1f}, 200.0".encode('utf-8')
    sock.sendto(payload, server_address)

    # Mechanical sweep logic
    angle += (direction * sweep_speed)
    if angle >= 60.0:
        direction = -1.0
    elif angle <= -60.0:
        direction = 1.0

    # 30Hz update rate
    time.sleep(0.05)
