import socket
import time

# UDP soketi oluşturuyoruz
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
server_address = ('127.0.0.1', 4210) # Rust sunucumuzun adresi ve portu

# Göndereceğimiz sahte veri formatı: base_x, base_y, heading, scan_angle, scan_dist
# Örnek: Araç (0,0) konumunda, 0 derece yöne bakıyor. Sensör 45 derece açıyla 100 cm ötede bir engel gördü.
mock_data = b"0.0, 0.0, 0.0, 45.0, 100.0"

print(f"Rover simüle ediliyor. Veri gönderiliyor: {mock_data.decode()}")

# Veriyi 3 kere peş peşe gönderelim ki grid güncellensin
for i in range(3):
    sock.sendto(mock_data, server_address)
    time.sleep(0.1)

print("Veri gönderimi tamamlandı.")