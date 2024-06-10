// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: MIT

#![feature(decl_macro, coroutines, coroutine_trait)]
#![deny(
  warnings,
  nonstandard_style,
  unused,
  future_incompatible,
  rust_2018_idioms,
  unsafe_code
)]
#![deny(clippy::all, clippy::nursery, clippy::pedantic)]
#![recursion_limit = "128"]

mod ppm;

use {crate::ppm::PPMParser, image::DynamicImage, std::process::exit};

#[allow(unused)]
fn get_image(parser: &mut PPMParser, index: usize) -> DynamicImage {
  let frame = parser.get_frame_pixels(index);
  let colours = parser.get_frame_palette(index);
  let mut img = Vec::new();
  let mut img_encoder = image::codecs::bmp::BmpEncoder::new(&mut img);

  img_encoder.encode_with_palette(
    &frame.into_iter().flatten().collect::<Vec<u8>>(),
    256,
    192,
    image::ColorType::L8,
    Some(&[
      [colours[0].0, colours[0].1, colours[0].2],
      [colours[1].0, colours[1].1, colours[1].2],
      [colours[2].0, colours[2].1, colours[2].2],
    ]),
  );

  image::load_from_memory(&img).unwrap()
}

fn main() {
  human_panic::setup_panic!(
    human_panic::Metadata::new(
      env!("CARGO_PKG_NAME"),
      env!("CARGO_PKG_VERSION")
    )
    .authors(env!("CARGO_PKG_AUTHORS"))
    .homepage(env!("CARGO_PKG_HOMEPAGE"))
  );

  let args = std::env::args().collect::<Vec<_>>();

  if args.len() < 4 {
    println!(
      "{}, version {}(1)-{}-({})-{}\n\
      usage:  {} <in> <index option> <out>\n\
      index options:\n\
             \tgif\n\
             \tthumb\n\
             \tdump\n\
             \tinteger(u16)\n\n\
             {0} home page: <https://github.com/Usugata/{0}>",
      env!("CARGO_PKG_NAME"),
      env!("CARGO_PKG_VERSION"),
      env!("PROFILE"),
      env!("TARGET"),
      env!("GIT_COMMIT_HASH"),
      args[0],
    );
    exit(1);
  }

  let path = &args[1];
  let index = &args[2];
  let out_path = &args[3];
  let mut parser = PPMParser::new_from_file(path);
  parser.load();
  let frame_count = usize::from(parser.get_frame_count());

  match index.as_str() {
    "gif" => {
      let frame_duration = (1.0 / parser.get_framerate()) * 100.0;
      let frames = (0..parser.get_frame_count())
        .map(|i| get_image(&mut parser, i as usize))
        .collect::<Vec<DynamicImage>>();
      let mut file_out = std::fs::File::create(out_path).unwrap();
      let mut gif_encoder = image::codecs::gif::GifEncoder::new_with_speed(
        &mut file_out,
        #[allow(clippy::cast_possible_truncation)]
        {
          frame_duration as i32
        },
      );
      for frame in frames {
        gif_encoder
          .encode_frame(image::Frame::new(frame.into_rgba8()))
          .unwrap();
      }
      gif_encoder.set_repeat(image::codecs::gif::Repeat::Infinite).unwrap();
    }
    "thumb" => {
      let thumb_index = parser.get_thumb_index() as usize;
      get_image(&mut parser, thumb_index).save(out_path).unwrap();
    }
    "dump" => parser.dump_to_json(out_path),
    _ => {
      if !(0..frame_count).contains(&index.parse::<usize>().unwrap()) {
        println!(
          "invalid frame index({}), image has {}(0..{}) frames",
          index,
          frame_count,
          frame_count - 1,
        );
        exit(1);
      }

      get_image(&mut parser, index.parse::<usize>().unwrap())
        .save(out_path)
        .unwrap();
    }
  }

  println!("converted {path}({index}) to {out_path}");
}
