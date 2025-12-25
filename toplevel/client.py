import socket
import input as ipt

path = "./net_cfg"

cfgs = open(path).read().splitlines()
assert (len(cfgs) >= 2)

addr, port = cfgs[0], cfgs[1]
print(addr, port)
socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
socket.connect((addr, int(port)))

while True:
    ipt.clear_stdin()

    command = input("$ ")
    if command == "":
        continue

    socket.send(command.encode(encoding="utf-8"))
    back = int.from_bytes(socket.recv(4096), byteorder='big')

    if back == 1:
        print("Syntax error...")
    if back == 3:
        print("Unimplemented command...")
    if back == 4:  # EXIT signal code
        break


print("[Client shutdown...]")
