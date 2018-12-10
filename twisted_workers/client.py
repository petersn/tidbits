#!/usr/bin/python

import xmlrpc.client

class Connection:
	def __init__(self, uri):
		self.proxy = xmlrpc.client.ServerProxy(uri)

	def launch_task(self, configuration_blob, count):
		return self.proxy.launch_task(configuration_blob, count)

if __name__ == "__main__":
	connection = Connection("http://localhost:50018/")
	results = connection.launch_task(b"123", 20)
	print("Got back:", [int(r.data) for r in results])
	print("Total of %i results." % (len(results),))

