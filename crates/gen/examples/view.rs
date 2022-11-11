use itertools::izip;
use sdf_gen::*;
use std::{env, fs::File};

fn main() {
  let Some(filename) = env::args().nth(1) else { panic!("No filename given") };
  eprintln!("{filename:?}");

  let decoder = png::Decoder::new(File::open(filename).unwrap());
  let mut reader = decoder.read_info().unwrap();
  let mut buf = vec![0; reader.output_buffer_size()];
  let info = reader.next_frame(&mut buf).unwrap();

  let bytes = &buf[..info.buffer_size()];

  let sdf_width = 40;
  let sdf_height = 40;

  let mut image = Image::new("view_image.png", [1000, 1000]);
  for y in 0..image.height {
    for x in 0..image.width {
      // normalised coordinates
      let x_norm = x as f32 / image.width as f32;
      let y_norm = y as f32 / image.height as f32;

      // points in sdf coordinate system
      let x_sdf_p = x_norm * (sdf_width - 1) as f32;
      let y_sdf_p = y_norm * (sdf_height - 1) as f32;

      let sample_image = |x, y| -> [u8; 3] {
        let offset = (y * sdf_height + x) * 3;
        [bytes[offset], bytes[offset + 1], bytes[offset + 2]]
      };

      // sample from points, bilinear
      let pixel = {
        let x1 = (x_sdf_p - 0.5).floor();
        let y1 = (y_sdf_p - 0.5).floor();
        let x2 = x1 + 1.;
        let y2 = y1 + 1.;
        let wx = x_sdf_p - x1 - 0.5;
        let wy = y_sdf_p - y1 - 0.5;

        let t1 = sample_image(x1 as usize, y1 as usize).map(|v| (1. - wx) * (1. - wy) * v as f32);
        let t2 = sample_image(x2 as usize, y1 as usize).map(|v| wx * (1. - wy) * v as f32);
        let t3 = sample_image(x1 as usize, y2 as usize).map(|v| (1. - wx) * wy * v as f32);
        let t4 = sample_image(x2 as usize, y2 as usize).map(|v| wx * wy * v as f32);

        let result: Vec<f32> = izip!(t1, t2, t3, t4)
          .map(|(v1, v2, v3, v4)| v1 + v2 + v3 + v4)
          .collect();

        [result[0] as u8, result[1] as u8, result[2] as u8]
      };

      // find the median value
      let val = if (pixel[0] <= pixel[1] && pixel[1] <= pixel[2])
        || (pixel[2] <= pixel[1] && pixel[1] <= pixel[0])
      {
        pixel[1]
      } else if (pixel[0] <= pixel[2] && pixel[2] <= pixel[1])
        || (pixel[1] <= pixel[2] && pixel[2] <= pixel[0])
      {
        pixel[2]
      } else {
        pixel[0]
      };

      // colour the output based on a simple threshold
      let mut new_val = 0;
      if val > 125 {
        new_val = 255;
      }
      // add an outline effect
      let mut r_val = new_val;
      if val > 100 {
        r_val = 255;
      }

      image.set_pixel([x, y], [r_val, new_val, new_val]);
    }
  }
  image.flush();
}
