// Todo: Implement multithreading

use opencv::{core::Mat, core::Vector, imgcodecs, prelude::VideoCaptureTrait, prelude::VideoCaptureTraitConst, videoio};

use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;

fn collect_buffer(camera: &mut videoio::VideoCapture, frame: &mut Mat, buffer: &mut Vector<u8>) {
	camera.read(frame).expect("Failed to capture frame");
	buffer.clear();
	imgcodecs::imencode(".jpg", frame, buffer, &Vector::new()).expect("Failed to fill buffer");
}

fn main() {
	let args: Vec<String> = env::args().collect();

	let port = args[1].parse::<u16>().unwrap_or(8080);
	let width = args[2].parse::<f64>().unwrap_or(320.0);
	let height = args[3].parse::<f64>().unwrap_or(240.0);

	let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect(&format!("Failed to get 0.0.0.0:{}", port));
	let mut camera = videoio::VideoCapture::new(0, videoio::CAP_ANY).expect("Failed to get video capture");

	camera.set(videoio::CAP_PROP_FRAME_WIDTH, width).expect("Failed to set width");
	camera.set(videoio::CAP_PROP_FRAME_HEIGHT, height).expect("Failed to set height");

	println!("Camera FPS: {}", camera.get(videoio::CAP_PROP_FPS).expect("Failed to get fps"));

	let mut frame = Mat::default();
	let mut buffer = Vector::new();

	loop {
		let (mut stream, client) = listener.accept().expect("Failed to accept connection");
		println!("New viewer {:#?}", client);

		let mut header_get = String::new();
		BufReader::new(stream.try_clone().unwrap()).read_line(&mut header_get).expect("Failed to parse header");

		if header_get.contains("stream") || header_get.contains("mjpg") {
			let response = format!("HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n");
			stream.write_all(response.as_bytes()).expect("Failed to write response to stream");

			loop {
				collect_buffer(&mut camera, &mut frame, &mut buffer);

				let mut closing_operations = || -> Result<(), std::io::Error> {
					stream.write_all(format!("--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n", buffer.len()).as_bytes())?;
					stream.write_all(buffer.as_slice())?;
					stream.write_all(b"\r\n")?;
					stream.flush()?;
					Ok(())
				};

				if let Err(_) = closing_operations() {
					println!("Viewer {:#?} leaving", client);
					break;
				}
			}
		} else {
			collect_buffer(&mut camera, &mut frame, &mut buffer);

			let mut closing_operations = || -> Result<(), std::io::Error> {
				stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length {}\r\n\r\n", buffer.len()).as_bytes())?;
				stream.write_all(buffer.as_slice())?;
				stream.write_all(b"\r\n")?;
				stream.flush()?;
				Ok(())
			};

			if let Err(_) = closing_operations() {
				println!("Viewer {:#?} leaving", client);
			}
		}
	}
}
