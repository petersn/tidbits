#!/usr/bin/python3

import flask
import flask_login
import flask_sqlalchemy

# Initialize our Flask app.
app = flask.Flask("example-app")
login_manager = flask_login.LoginManager()
login_manager.init_app(app)

# Connect the database.
app.config["SQLALCHEMY_DATABASE_URI"] = "sqlite:///data/database.db"
db = flask_sqlalchemy.SQLAlchemy(app)

# Create database data model.
class User(db.Model, flask_login.UserMixin):
	__tablename__ = "users"
	id            = db.Column("id", db.Integer, primary_key=True)
	name          = db.Column("name", db.String(200), unique=True, nullable=False)
	password_hash = db.Column("password_hash", db.String(256))
	creation_time = db.Column("creation_time", db.DateTime)

	def __repr__(self):
		return "<User(%i) %r>" % (self.id, self.name)

@app.route("/")
def index():
	return flask.render_template("index.html")

if __name__ == "__main__":
	app.run(debug=True)

