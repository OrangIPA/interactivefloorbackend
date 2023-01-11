use opencv::{
    core::{self, absdiff, Point, Size_, BORDER_CONSTANT, BORDER_DEFAULT, CV_8U},
    highgui::{self, wait_key},
    imgproc::{
        self, morphology_default_border_value, CHAIN_APPROX_SIMPLE, COLOR_RGB2GRAY, LINE_8,
        RETR_EXTERNAL, THRESH_BINARY,
    },
    prelude::*,
    videoio, Result,
};

use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

// #[allow(unused_imports)]
// use autopilot::{key, mouse};

fn main() -> Result<()> {
    // Create camera object and initialize window
    let mut cam = videoio::VideoCapture::new(1, videoio::CAP_ANY)?;
    highgui::named_window("woo", highgui::WINDOW_AUTOSIZE)?;

    // Variable declaration
    let mut frame = Mat::default();
    let mut previous_frame: Option<Mat> = None;
    let mut grayscaled_frame = Mat::default();
    let mut blurred_image = Mat::default();
    let mut dilated_frame = Mat::default();
    let mut thresh_frame = Mat::default();
    let mut contours: core::Vector<core::Vector<Point>> = core::Vector::new();

    let c: Arc<Mutex<Vec<(i32, i32)>>> = Arc::new(Mutex::new(vec![]));
    // let mut chill: i32 = 0;
    // #[allow(unused)]let mut play = false;

    // Get camera size
    // cam.read(&mut frame)?;
    // let frame_size = frame.mat_size().to_vec();
    // #[allow(unused)]let (cam_width, cam_height) = (frame_size.get(0).unwrap(), frame_size.get(1).unwrap());

    // highgui::named_window("screen size", WND_PROP_FULLSCREEN)?;
    // highgui::set_window_property("screen size", WND_PROP_FULLSCREEN, WINDOW_FULLSCREEN as f64)?;
    // let (screen_width, screen_height) = (get_window_image_rect("screen size")?.width, get_window_image_rect("screen size")?.height);

    let c_clone = Arc::clone(&c);
    thread::spawn(move || {
        let c = c_clone;
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            handle_connection(stream, Arc::clone(&c));
        }
    });

    loop {
        // Insert the camera output to variable frame
        cam.read(&mut frame)?;

        // Grayscale and blur the image
        imgproc::cvt_color(&frame, &mut grayscaled_frame, COLOR_RGB2GRAY, 0)?;
        imgproc::gaussian_blur(
            &grayscaled_frame,
            &mut blurred_image,
            Size_::new(5, 5),
            0.,
            0.,
            BORDER_DEFAULT,
        )?;

        // First iteration; previous frame contains nothing
        if let None = previous_frame {
            previous_frame = Some(blurred_image.clone());
            continue;
        }
        let prev_frame = previous_frame.clone().unwrap();

        // Calculate the difference between current and previous frame
        let mut diff_frame = Mat::default();
        absdiff(&prev_frame, &blurred_image, &mut diff_frame)?;
        previous_frame = Some(blurred_image.clone());

        // Dilute the image a bit
        let kernel = Mat::ones(35, 35, CV_8U)?;
        imgproc::dilate(
            &diff_frame,
            &mut dilated_frame,
            &kernel,
            opencv::core::Point_::new(-1, -1),
            1,
            BORDER_CONSTANT,
            morphology_default_border_value()?,
        )?;

        // Only take different area that are above the threshold
        imgproc::threshold(&dilated_frame, &mut thresh_frame, 20., 255., THRESH_BINARY)?;

        // Find and draw contours
        imgproc::find_contours(
            &thresh_frame,
            &mut contours,
            RETR_EXTERNAL,
            CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;
        // imgproc::draw_contours(&mut frame, &contours, -1, VecN::new(0., 255., 0., 255.), 2, LINE_AA, &Mat::default(), 255, Point::new(0, 0))?;

        let mut coor = (*c).lock().unwrap();
        *coor = vec![];
        for contour in contours.clone() {
            if imgproc::contour_area(&contour, false)? < 10000. {
                continue;
            } // Too small; continue
            let bounding_rect = imgproc::bounding_rect(&contour)?;

            let (x, y) = (bounding_rect.x, bounding_rect.y);
            // #[allow(unused_variables)]
            // let loc = (bounding_rect.x, bounding_rect.y);
            imgproc::rectangle(
                &mut frame,
                bounding_rect,
                core::Scalar::new(0., 255., 0., 255.),
                2,
                LINE_8,
                0,
            )?;
            
            coor.push((x, y));

            // Click section
            // if chill > 10 {
            //     if play {
            //         mouse::move_to(autopilot::geometry::Point {
            //             x: (loc.0 as f64),
            //             y: (loc.1 as f64),
            //         })
            //         .unwrap();
            //         mouse::click(mouse::Button::Left, None);
            //     }
            //     chill = 0
            // } else {
            //     chill += 1;
            // }
            // End click section
        }

        highgui::imshow("woo", &frame)?;
        // highgui::imshow("woo", &thresh_frame)?;
        if wait_key(30).unwrap() == 27 {
            break;
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, c: Arc<Mutex<Vec<(i32, i32)>>>) {
    // let buf_reader = BufReader::new(&mut stream);
    // let request_line = buf_reader.lines().next().unwrap().unwrap();

    let status_line = "HTTP/1.1 200 OK";


    let mut contents = "[".to_owned();
    let c = (*c).lock().unwrap();
    for (i, c) in c.clone().into_iter().enumerate() {
        if i == 0 {
            contents = format!("{contents}[{}, {}]", c.0, c.1);
            continue;
        }
        contents = format!("{contents}, [{}, {}]", c.0, c.1);

    }
    contents = format!("{contents}]");
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type:application/json\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
