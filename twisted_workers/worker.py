#!/usr/bin/python

import threading, struct
import twisted.internet
from twisted.internet import reactor
from twisted.protocols.basic import NetstringReceiver
import computation

class WorkerThread(threading.Thread):
	def __init__(self, engine, version_number, post_work_callback):
		threading.Thread.__init__(self)
		self.engine = engine
		self.version_number = version_number
		self.post_work_callback = post_work_callback
		self.stop_event = threading.Event()

	def stop(self):
		self.stop_event.set()

	def run(self):
		while not self.stop_event.is_set():
			result = self.engine.compute()
			self.post_work_callback(self.version_number, result)

class WorkerProtocol(NetstringReceiver):
	worker_thread = None

	def stop_work(self):
		if self.worker_thread is None:
			return
		self.worker_thread.stop()
		self.worker_thread.join()
		self.worker_thread = None

	def start_work(self):
		assert self.worker_thread is None
		self.worker_thread = WorkerThread(self.engine, self.version_number, self.post_work)
		self.worker_thread.start()

	def post_work(self, version_number, result):
		if version_number != self.version_number:
			print("Discarding stale work for %i, when %i is current." % (version_number, self.version_number))
			return
		message = b"P" + struct.pack("<I", version_number) + result
		self.sendString(message)

	def connectionMade(self):
		self.engine = computation.ComputationEngine()
		self.sendString(b"R")

	def stringReceived(self, s):
		command_type = s[:1]
		if command_type == b"d": # command: die
			reactor.stop()
		elif command_type == b"c": # command: configure
			self.stop_work()
			self.version_number, = struct.unpack("<I", s[1:5])
			configuration_blob = s[5:]
			self.engine.configure(configuration_blob)
		elif command_type == b"w": # command: work
			self.start_work()
		elif command_type == b"s": # command: stop
			self.stop_work()
		else:
			raise ValueError("Protocol failure: %r" % (s,))

class Factory(twisted.internet.protocol.ClientFactory):
	protocol = WorkerProtocol

def main():
	factory = Factory()
	reactor.connectTCP("localhost", 50017, factory)
	reactor.run()

if __name__ == "__main__":
	main()

