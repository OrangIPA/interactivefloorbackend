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

use std::fs;

fn main() -> Result<()> {
    // Create camera object and initialize window
    let cam_index = fs::read_to_string("cam-index.txt")
        .unwrap()
        .parse::<i32>()
        .unwrap();
    let mut cam = videoio::VideoCapture::new(cam_index, videoio::CAP_ANY)?;
    highgui::named_window("woo", highgui::WINDOW_AUTOSIZE)?;

    // Variable declaration
    let mut frame = Mat::default();
    let mut previous_frame: Option<Mat> = None;
    let mut grayscaled_frame = Mat::default();
    let mut blurred_image = Mat::default();
    let mut dilated_frame = Mat::default();
    let mut thresh_frame = Mat::default();
    let mut contours: core::Vector<core::Vector<Point>> = core::Vector::new();
    let mut c: Vec<(f32, f32)>;

    // Get camera size
    cam.read(&mut frame)?;
    let frame_size = frame.mat_size().to_vec();
    let (cam_width, cam_height) = (
        frame_size.get(0).unwrap().to_owned() as f32,
        frame_size.get(1).unwrap().to_owned() as f32,
    );
    loop {
        c = vec![];
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

        for contour in contours.clone() {
            if imgproc::contour_area(&contour, false)? < 10000. {
                continue;
            } // Too small; continue
            let bounding_rect = imgproc::bounding_rect(&contour)?;

            #[allow(unused_variables)]
            let (x, y) = (
                (bounding_rect.x + (bounding_rect.height / 2)) as f32 / cam_height,
                (bounding_rect.y + (bounding_rect.width / 2)) as f32 / cam_width,
            );

            c.push((x, y));

            imgproc::rectangle(
                &mut frame,
                bounding_rect,
                core::Scalar::new(0., 255., 0., 255.),
                2,
                LINE_8,
                0,
            )?;
        }

        // let mut contents = "[".to_owned();
        // for (i, c) in c.clone().into_iter().enumerate() {
        //     if i == 0 {
        //         contents = format!("{contents}[{}, {}]\n", c.0, c.1);
        //         continue;
        //     }
        //     contents = format!("{contents}, [{}, {}]\n", c.0, c.1);
        // }
        // contents = format!("{contents}]");
        let mut contents = "".to_owned();
        for c in c.into_iter() {
            contents = format!("{contents}{} {}\n", c.0, c.1);
        }

        fs::write("yahahahayyuukk.txt", contents).unwrap_or_else(|e| {
            eprintln!("error: {e}");
        });

        highgui::imshow("woo", &frame)?;
        // highgui::imshow("woo", &thresh_frame)?;
        if wait_key(30).unwrap() == 27 {
            break;
        }
    }
    Ok(())
}
