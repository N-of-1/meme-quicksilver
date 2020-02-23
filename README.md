<p align="center">
  <img src="images/nof1-logo.png">
</p>

# meme-quicksilver

master
[![Master branch build status](https://github.com/N-of-1/meme-quicksilver/workflows/Rust/badge.svg?branch=master)](https://github.com/N-of-1/meme-quicksilver/actions) &emsp; dev
[![Test branch build status](https://github.com/N-of-1/meme-quicksilver/workflows/Rust/badge.svg?branch=dev)](https://github.com/N-of-1/meme-quicksilver/actions)

UI for meme machine demo using the quicksilver gaming library

To setup for web builds, install Apache locally and fix the WASM MIME type, then:
`cargo install cargo-web ./deploy`

## log file

All values recieved from the Muse headset are written unmodified to a log file

These files are in ./log subdirectory below the directory where the application is being run. For performance an stability is recommended to create this on an external hard drive or SSD rather than a MicroSD card.

To add an event to the log file
´´´
info!("message that might be parsed");
´´´

## database setup

Install Postgresql locally
In Postgres command line client, create a user

..

In Postgres command line client, create meme\*database

..

In your project directory, type the following. This file will \_not\* be in the git repo
´´´
echo DATABASE_URL=postgres://username:password@localhost/meme_database > .env
´´´

In your project directory, one time configuration of Diesel database library using the .env
´´´
diesel setup
´´´

The following was done to create ´/migrations/../up.sql´ and ´../down.sql´. There are used to create and remove tables in the database on each computer to match the schema set in the code.
´´´
diesel migration generate create_posts
´´´
