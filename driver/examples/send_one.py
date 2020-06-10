#!/usr/bin/env python
# coding: utf-8

"""
This example shows how sending a single message works using the python-can
package with CANtact.
"""

from __future__ import print_function

import can
import cantact


def send_one():

    bus = can.interface.Bus(
        bustype="cantact", channel="0", bitrate=500000, monitor=False
    )

    msg = can.Message(
        arbitration_id=0xC0FFEE, data=[0, 25, 0, 1, 3, 1, 4, 1], is_extended_id=True
    )

    try:
        bus.send(msg)
        print("Message sent on {}".format(bus.channel_info))
    except can.CanError:
        print("Message NOT sent")


if __name__ == "__main__":
    send_one()
