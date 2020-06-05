#!/usr/bin/env python
# coding: utf-8

"""
Python CANtact dump example.

A simple example of dumping frames from the bus via the cantact API.
Note, most users will want to use the python-can package instead 
of direct access!
"""

import cantact

# create the interface
intf = cantact.Interface()

# set the CAN bitrate
intf.set_bitrate(0, 500000)

# enable channel 0
intf.set_enabled(0, True)

# start the interface
intf.start()

while True:
    try:
        # wait for frame with 10 ms timeout
        f = intf.recv(10)
        if f != None:
            # received frame
            print(f)
    except KeyboardInterrupt:
        # ctrl-c pressed, close the interface
        intf.stop()
        break
