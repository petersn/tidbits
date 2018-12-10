#!/usr/bin/python

import struct
import twisted.internet
from twisted.internet import reactor
from twisted.protocols.basic import NetstringReceiver
from twisted.internet.defer import inlineCallbacks, returnValue
from twisted.web import xmlrpc, server
from twisted.internet.task import deferLater

worker_pool = set()

class ManagerProtocol(NetstringReceiver):
	def connectionMade(self):
		self.version_number = 1000

	def connectionLost(self, status):
		print("Worker disconnected:", self)
		if self in worker_pool:
			worker_pool.remove(self)

	def stringReceived(self, s):
		command_type = s[:1]
		if command_type == b"R": # command: READY
			print("Worker connected:", self)
			worker_pool.add(self)
		elif command_type == b"P": # command: POST
			version_number, = struct.unpack("<I", s[1:5])
			result = s[5:]
			if version_number != self.version_number:
				print("Discarding stale work for %i, when %i is current." % (version_number, self.version_number))
				return
			self.work_target.append(result)
		else:
			raise ValueError("Protocol failure: %r" % (s,))

	def launch_task(self, configuration_blob, work_target):
		# Set the destination for results to be accumulated in.
		self.work_target = work_target
		# Increment the version number, to avoid using stale data that's incoming.
		self.version_number += 1
		# Configure the client, and tell it to start working!
		message = b"c" + struct.pack("<I", self.version_number) + configuration_blob.data
		self.sendString(message)
		self.sendString(b"w")

	def stop_task(self):
		self.sendString(b"s")

class Factory(twisted.internet.protocol.ServerFactory):
	protocol = ManagerProtocol

class FrontEnd(xmlrpc.XMLRPC):
	@inlineCallbacks		
	def xmlrpc_launch_task(self, configuration_blob, count):
		print("Launching %i responses to: %r" % (count, configuration_blob))

		# Copy to avoid mutation during waiting loop.
		workers = set(worker_pool)

		results = []
		for worker in workers:
			worker.launch_task(configuration_blob, results)

		# Horrific busy-wait here!
		# FIXME: Replace with proper event-driven wakeup.
		while len(results) < count:
			yield deferLater(reactor, 0.1, lambda: None)

		for worker in workers:
			worker.stop_task()

		return results

def main():
	factory = Factory()
	reactor.listenTCP(50017, factory)
	reactor.listenTCP(50018, server.Site(FrontEnd()))
	reactor.run()

if __name__ == "__main__":
	main()

