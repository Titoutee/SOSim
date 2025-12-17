import socket
import input as ipt
import cmd

path = "./net_cfg"

cfgs = open(path).read().splitlines()
assert (len(cfgs) >= 2)

addr, port = cfgs[0], cfgs[1]
print(addr, port)
socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
socket.connect((addr, int(port)))

while True:
    ipt.clear_stdin()

    command = input("$ ").strip()  # command sent as plain text to server
    socket.send(command.encode())
    back = int.from_bytes(socket.recv(4096), byteorder='big')

    print(back)
    if back == 4:  # EXIT signal code
        break
    elif back == 1:
        print("Syntax error...")

print("[Client shutdown...]")
