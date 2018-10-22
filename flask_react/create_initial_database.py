#!/usr/bin/python3

import datetime
import app

if __name__ == "__main__":
	print("Creating database.")
	app.db.create_all()

	# Create an admin user.
	app.db.session.add(app.User(
		id=1,
		name="root",
		password_hash="???",
		creation_time=datetime.datetime.now(),
	))
	app.db.session.commit()

	print("Users:", app.User.query.all())

