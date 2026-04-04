use opencv::{
    core, face, imgproc, objdetect, prelude::*, videoio
};
// Removed 'types' as we'll use core::Vector directly
use std::{thread, time::Duration};

fn main() {
    println!("🚀 Blink Enrollment: Time to learn your face Buddy!");

    let ir_camera_path = std::fs::read_to_string("/etc/blink/device")
        .unwrap_or_else(|_| "/dev/video0".to_string())
        .trim()
        .to_string();

    println!("Using camera: {}", ir_camera_path);
    let mut cam = videoio::VideoCapture::from_file(&ir_camera_path, videoio::CAP_V4L2).unwrap();

    let cascade_path = "/usr/share/opencv4/haarcascades/haarcascade_frontalface_alt2.xml";
    let mut face_detector = objdetect::CascadeClassifier::new(cascade_path).unwrap();

    // FIX: Using the generic core::Vector for Matrices
    let mut face_images = core::Vector::<Mat>::new();
    let mut labels = core::Vector::<i32>::new();

    println!("📸 Look directly into the IR camera. Capturing 15 frames...");

    let mut count = 0;
    let mut no_face_count = 0;
    while count < 15 {
        let mut frame = Mat::default();
        if cam.read(&mut frame).unwrap() {
            if frame.empty() {
                println!("❌ Camera returned an empty frame! Are you sure {} is the correct capture device?", ir_camera_path);
                println!("(Hint: Linux often creates two nodes per camera. If /dev/video3 failed, try /dev/video2)");
                break;
            }

            let mut gray = Mat::default();
            imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap_or(());

            let mut equalized = Mat::default();
            imgproc::equalize_hist(&gray, &mut equalized).unwrap_or(());

            let mut faces = core::Vector::<core::Rect>::new();
            face_detector.detect_multi_scale(
                &equalized, &mut faces, 1.1, 3, 0, 
                core::Size::new(100, 100), core::Size::new(0, 0)
            ).unwrap_or(());

            if faces.len() > 0 {
                let face_rect = faces.get(0).unwrap();
                let face_roi = Mat::roi(&equalized, face_rect).unwrap();
                
                // Ensure we clone the data properly for the vector
                face_images.push(face_roi.try_clone().unwrap());
                labels.push(1); // 1 = James/Amor

                count += 1;
                no_face_count = 0;
                println!("\n📸 Captured frame {}/15...", count);
                
                thread::sleep(Duration::from_millis(300)); 
            } else {
                no_face_count += 1;
                if no_face_count % 10 == 0 {
                    println!("⏳ Scanning... no face detected yet. Ensure your face is lit and you're using the right camera.");
                }
            }
        } else {
            println!("❌ Failed to grab frame from camera.");
            break;
        }
    }

    cam.release().unwrap_or(());
    println!("🧠 15 frames captured! Training the LBPH AI model...");

    let mut recognizer = face::LBPHFaceRecognizer::create(1, 8, 8, 8, 100.0).unwrap();
    
    // Train using the vector of Mats and vector of IDs
    recognizer.train(&face_images, &labels).unwrap();

    let model_path = "/etc/blink/blink_model.yml";
    
    // Explicitly using the trait to avoid 'multiple applicable items' error
    opencv::prelude::FaceRecognizerTraitConst::write(&recognizer, model_path).unwrap();
    
    println!("✅ Model successfully trained and saved to {}!", model_path);
}