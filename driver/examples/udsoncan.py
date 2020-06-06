import can
import isotp
import udsoncan
from udsoncan.connections import PythonIsoTpConnection
from udsoncan.client import Client
from udsoncan.exceptions import *
from udsoncan.services import *

udsoncan.setup_logging()

bus = can.interface.Bus(bustype='cantact', channel='0', bitrate=500000)
addr = isotp.Address(addressing_mode=isotp.AddressingMode.Normal_11bits, 
                     txid=0x123,
                     rxid=0x456
                     )
tp = isotp.CanStack(bus, address=addr)
conn = PythonIsoTpConnection(tp)
client = Client(conn)

conn.open()
client.ecu_reset(ECUReset.ResetType.hardReset)
print("done")
conn.close()
