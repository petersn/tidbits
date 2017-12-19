#!/usr/bin/python2

import math, os
from Crypto.PublicKey import RSA

# Load up the secret message.
message = open("secret/secret_message").read().strip()
message = long(message.encode("hex"), 16)

# Generate three key-pairs, and encrypt the message to each of them.
for i in (1, 2, 3):
	# Generate a 1024-bit RSA private key.
	os.system("openssl genrsa -3 -out secret/private%i.key 1024" % i)

	# Produce the corresponding public keys.
	os.system("openssl rsa -in secret/private%i.key -pubout -out data/public%i.key" % (i, i))

	# Read in the public key, and encrypt the message under it.
	key = RSA.importKey(open("data/public%i.key" % i))
	ciphertext, = key.encrypt(message, None)

	# Write out the encrypted message.
	with open("data/ciphertext%i" % i, "w") as f:
		print >>f, ciphertext

