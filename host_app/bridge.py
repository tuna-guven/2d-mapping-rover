import serial
import socket
import time

# Ayarlarını kendi bilgisayarına göre yap
COM_PORT = 'COM9' # Mac/Linux için '/dev/ttyUSB0' gibi olabilir
BAUD_RATE = 115200
UDP_IP = "127.0.0.1"
UDP_PORT = 4210

# Bağlantıları başlat
try:
    ser = serial.Serial(COM_PORT, BAUD_RATE, timeout=1)
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    print(f"✅ Köprü kuruldu: {COM_PORT} dinleniyor, UDP {UDP_PORT} portuna aktarılıyor...")
except Exception as e:
    print(f"Bağlantı hatası: {e}")
    exit()

# Sonsuz döngüde USB'den oku, UDP'ye at
while True:
    try:
        if ser.in_waiting > 0:
            # Arduino'dan gelen metni oku
            line = ser.readline().decode('utf-8').strip()
            
            # Eğer beklediğimiz virgüllü formattaysa Rust'a yolla
            if line and line.count(',') == 4:
                sock.sendto(line.encode('utf-8'), (UDP_IP, UDP_PORT))
                print(f"🚀 İletildi: {line}")
    except KeyboardInterrupt:
        print("Köprü kapatılıyor.")
        ser.close()
        break
    except Exception as e:
        print(f"Hata: {e}")