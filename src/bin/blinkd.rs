use opencv::{core, face, imgproc, objdetect, prelude::*, videoio};
use std::os::unix::net::UnixListener;
use std::io::{Read, Write};
use std::fs;
use std::os::unix::fs::PermissionsExt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/var/run/blink.sock";
    let model_path = "/etc/blink/blink_model.yml";
    let cascade_path = "/usr/share/opencv4/haarcascades/haarcascade_frontalface_alt2.xml";

    // 1. Pre-load the recognizer to make auth instant
    let mut recognizer = face::LBPHFaceRecognizer::create(1, 8, 8, 8, 100.0)?;
    if let Err(e) = face::FaceRecognizerTrait::read(&mut recognizer, model_path) {
        eprintln!("❌ Daemon: Failed to load model: {:?}. Run enroll first!", e);
        return Ok(());
    }

    // 2. Setup the socket
    let _ = fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o666))?;

    println!("🚀 Blink Daemon: Online and listening on {}...", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1];
                if stream.read(&mut buffer).is_ok() {
                    println!("📸 Auth requested! Checking face...");
                    
                    // Run the actual recognition logic
                    let success = perform_recognition(&mut recognizer, cascade_path);
                    
                    if success {
                        println!("✅ Match found! Sending success to PAM.");
                        let _ = stream.write_all(&[1]);
                    } else {
                        println!("❌ No match. Sending fail to PAM.");
                        let _ = stream.write_all(&[0]);
                    }
                }
            }
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }
    Ok(())
}

fn perform_recognition(recognizer: &mut core::Ptr<face::LBPHFaceRecognizer>, cascade_path: &str) -> bool {
    let ir_camera_path = std::fs::read_to_string("/etc/blink/device")
        .unwrap_or_else(|_| "/dev/video0".to_string())
        .trim()
        .to_string();
    
    // Open camera for just a moment
    let mut cam = match videoio::VideoCapture::from_file(&ir_camera_path, videoio::CAP_V4L2) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut frame = Mat::default();
    let mut success = false;

    // Capture 5 frames to give the IR sensor time to adjust exposure
    for _ in 0..5 {
        if let Ok(true) = cam.read(&mut frame) {
            let mut gray = Mat::default();
            let _ = imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0);
            
            let mut equalized = Mat::default();
            let _ = imgproc::equalize_hist(&gray, &mut equalized);

            if let Ok(mut face_detector) = objdetect::CascadeClassifier::new(cascade_path) {
                let mut faces = core::Vector::<core::Rect>::new();
                let _ = face_detector.detect_multi_scale(&equalized, &mut faces, 1.1, 3, 0, core::Size::new(100, 100), core::Size::new(0, 0));

                if faces.len() > 0 {
                    let face_roi = Mat::roi(&equalized, faces.get(0).unwrap()).unwrap();
                    let mut label = -1;
                    let mut confidence = 0.0;
                    
                    let _ = recognizer.predict(&face_roi, &mut label, &mut confidence);

                    // Threshold check
                    if label == 1 && confidence < 75.0 {
                        success = true;
                        break; // Found you!
                    }
                }
            }
        }
    }

    let _ = cam.release();
    success
}