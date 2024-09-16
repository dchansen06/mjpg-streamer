/*
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
*/

use clap::command;
use clap::value_parser;
use clap::Arg;
use clap::ArgAction;
use opencv::{core::Mat, core::Vector, imgcodecs, prelude::VideoCaptureTrait, videoio};
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

fn collect_buffer(camera: &mut videoio::VideoCapture, frame: &mut Mat, buffer: &mut Vector<u8>) {
	camera.read(frame).expect("Failed to capture frame");
	buffer.clear();
	imgcodecs::imencode(".jpg", frame, buffer, &Vector::new()).expect("Failed to fill buffer");
}

fn main() {
	let matches = command!("OctoPrint-Camera")
		.about("Sets up a MJPG stream at /stream and /mjpg as well as a jpg at anything else")
		.arg(Arg::new("port").short('p').long("port").help("Sets the port").action(ArgAction::Set).required(false).value_parser(value_parser!(u16)))
		.arg(Arg::new("width").short('w').long("width").help("Sets the width").action(ArgAction::Set).required(false).value_parser(value_parser!(f64)))
		.arg(Arg::new("height").short('v').long("height").help("Sets the height").action(ArgAction::Set).required(false).value_parser(value_parser!(f64)))
		.arg(Arg::new("video-id").short('i').long("id").help("Identifies the /dev/video#").action(ArgAction::Set).required(false).value_parser(value_parser!(i32)),)
		.get_matches();

	let port: u16 = *matches.get_one::<u16>("port").unwrap_or(&8080);
	let width: f64 = *matches.get_one::<f64>("width").unwrap_or(&320.0);
	let height: f64 = *matches.get_one::<f64>("height").unwrap_or(&240.0);
	let video: i32 = *matches.get_one::<i32>("video-id").unwrap_or(&0);

	println!("Reading port: {}", port);
	println!("Attempting: {}x{}", width, height);
	println!("Trying device: {}", video);

	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect(&format!("Failed to get 0.0.0.0:{}", port));
	let camera = Arc::new(Mutex::new(videoio::VideoCapture::new(video, videoio::CAP_ANY).expect("Failed to get video capture")));

	camera.lock().unwrap().set(videoio::CAP_PROP_FRAME_WIDTH, width).expect("Failed to set width");
	camera.lock().unwrap().set(videoio::CAP_PROP_FRAME_HEIGHT, height).expect("Failed to set height");

	let frame = Arc::new(Mutex::new(Mat::default()));
	let buffer = Arc::new(Mutex::new(Vector::new()));

	for stream in listener.incoming() {
		let camera = Arc::clone(&camera);
		let frame = Arc::clone(&frame);
		let buffer = Arc::clone(&buffer);

		thread::spawn(move || {
			let mut stream = stream.expect("Failed to accept connection");
			let client = stream.peer_addr().expect("Failed to continue connection");
			println!("Opening {}", client);
			let mut header_get = String::new();
			BufReader::new(stream.try_clone().unwrap()).read_line(&mut header_get).expect("Failed to parse header");

			if header_get.contains("stream") || header_get.contains("mjpg") {
				let response = format!("HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n");
				stream.write_all(response.as_bytes()).expect("Failed to write response to stream");

				loop {
					collect_buffer(&mut camera.lock().unwrap(), &mut frame.lock().unwrap(), &mut buffer.lock().unwrap());

					let mut closing_operations = || -> Result<(), std::io::Error> {
						stream.write_all(format!("--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n", buffer.lock().unwrap().len()).as_bytes())?;
						stream.write_all(buffer.lock().unwrap().as_slice())?;
						stream.write_all(b"\r\n")?;
						stream.flush()?;
						Ok(())
					};

					if let Err(_) = closing_operations() {
						break;
					}
				}
			} else {
				collect_buffer(&mut camera.lock().unwrap(), &mut frame.lock().unwrap(), &mut buffer.lock().unwrap());

				let mut closing_operations = || -> Result<(), std::io::Error> {
					stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length {}\r\n\r\n", buffer.lock().unwrap().len()).as_bytes())?;
					stream.write_all(buffer.lock().unwrap().as_slice())?;
					stream.write_all(b"\r\n")?;
					stream.flush()?;
					Ok(())
				};

				let _ = closing_operations();
			}

			println!("Closing {}", client);
		});
	}
}
