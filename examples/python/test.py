import cantact

i = cantact.Interface()
i.set_bitrate(0, 500000)
i.start(0)
while True:
    for b in i.recv()['data']:
        print("%02X" % b, end = " ")
    print("")