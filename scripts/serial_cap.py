import serial, sys

PORT = "/dev/cu.usbmodem1101"
BAUD = 115200

if __name__ == "__main__":
    dur = int(sys.argv[1]) if len(sys.argv) > 1 else 120
    with serial.Serial(PORT, BAUD, timeout=1) as ser:
        with open("/tmp/ultramcu-touch.log", "wb") as f:
            for _ in range(dur):
                line = ser.readline()
                if line:
                    f.write(line)
                    f.flush()
                    print(line.decode(errors="replace"), end="")
