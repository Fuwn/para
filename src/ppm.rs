// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: MIT

#![allow(clippy::cast_sign_loss)]

use std::{
  collections::HashMap,
  fs,
  io::{Cursor, Read},
  ops::Generator,
};

use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{DateTime, NaiveDateTime, Utc};

lazy_static::lazy_static! {
  // Flipnote speed -> frames per second
  static ref FRAMERATES: HashMap<u8, f64> = {
    let mut hashmap = HashMap::new();

    hashmap.insert(1, 0.5);
    hashmap.insert(2, 1.0);
    hashmap.insert(3, 2.0);
    hashmap.insert(4, 4.0);
    hashmap.insert(5, 6.0);
    hashmap.insert(6, 12.0);
    hashmap.insert(7, 20.0);
    hashmap.insert(8, 30.0);

    hashmap
  };

  // Thumbnail bitmap RGB colours
  static ref THUMBNAIL_PALETTE: &'static [(u64, u64, u64)] = &[
    (0xFF, 0xFF, 0xFF),
    (0x52, 0x52, 0x52),
    (0xFF, 0xFF, 0xFF),
    (0x9C, 0x9C, 0x9C),
    (0xFF, 0x48, 0x44),
    (0xC8, 0x51, 0x4F),
    (0xFF, 0xAD, 0xAC),
    (0x00, 0xFF, 0x00),
    (0x48, 0x40, 0xFF),
    (0x51, 0x4F, 0xB8),
    (0xAD, 0xAB, 0xFF),
    (0x00, 0xFF, 0x00),
    (0xB6, 0x57, 0xB7),
    (0x00, 0xFF, 0x00),
    (0x00, 0xFF, 0x00),
    (0x00, 0xFF, 0x00),
  ];

  // Frame RGB colours
  static ref BLACK: (u8, u8, u8) = (0x0E, 0x0E, 0x0E);
  static ref WHITE: (u8, u8, u8) = (0xFF, 0xFF, 0xFF);
  static ref BLUE: (u8, u8, u8) = (0x0A, 0x39, 0xFF);
  static ref RED: (u8, u8, u8) = (0xFF, 0x2A, 0x2A);
}

macro read_n_to_as_utf8_from_stream($n:expr, $from:ident) {
  String::from_utf8({
    let mut buffer = vec![0; $n];

    $from.stream.read_exact(&mut buffer).unwrap();

    buffer
  })
  .unwrap()
}

macro read_n_of_size_from_to_vec($n:expr, $from:tt, $size:ty) {{
  let mut buffer = vec![0 as $size; $n];

  $from.stream.read_exact(&mut buffer).unwrap();

  buffer
}}

fn strip_null(string: &str) -> String { string.replace(char::from(0), "") }

fn read_n_to_vec(stream: &mut Cursor<Vec<u8>>, n: usize) -> Vec<u8> {
  let mut buffer = vec![0; n];

  stream.read_exact(&mut buffer).unwrap();

  buffer
}

fn vec_u8_to_string(vec: &[u8]) -> String {
  vec.iter().rev().map(|m| format!("{:02X}", m)).collect()
}

pub struct PPMParser {
  stream:              Cursor<Vec<u8>>,
  layers:              Vec<Vec<Vec<u8>>>,
  prev_layers:         Vec<Vec<Vec<u8>>>,
  prev_frame_index:    usize,
  animation_data_size: u32,
  sound_data_size:     u32,
  frame_count:         u16,
  lock:                u16,
  thumb_index:         u16,
  root_author_name:    String,
  parent_author_name:  String,
  current_author_name: String,
  parent_author_id:    String,
  current_author_id:   String,
  parent_filename:     String,
  current_filename:    String,
  root_author_id:      String,
  partial_filename:    String,
  timestamp:           DateTime<Utc>,
  layer_1_visible:     bool,
  layer_2_visible:     bool,
  loop_:               bool,
  frame_speed:         u8,
  bgm_speed:           u8,
  framerate:           f64,
  bgm_framerate:       f64,
  offset_table:        Vec<u32>,
}
impl PPMParser {
  #[allow(unused)]
  pub fn new(stream: Vec<u8>) -> Self {
    Self {
      stream: Cursor::new(stream),
      ..Self::default()
    }
  }

  pub fn new_from_file(file: &str) -> Self {
    Self {
      stream: Cursor::new(std::fs::read(file).unwrap()),
      ..Self::default()
    }
  }

  pub fn load(&mut self) {
    self.read_header();
    self.read_meta();
    self.read_animation_header();
    self.read_sound_header();
    self.layers = vec![vec![vec![0; 256]; 192]; 2];
    self.prev_layers = vec![vec![vec![0; 256]; 192]; 2];
    self.prev_frame_index = isize::MAX as usize; // -1
  }

  /// Decode header
  ///
  /// <https://github.com/pbsds/hatena-server/wiki/PPM-format#file-header>
  fn read_header(&mut self) {
    self.stream.set_position(0);

    let _magic = read_n_to_as_utf8_from_stream!(4, self);
    let animation_data_size = self.stream.read_u32::<LittleEndian>().unwrap();
    let sound_data_size = self.stream.read_u32::<LittleEndian>().unwrap();
    let frame_count = self.stream.read_u16::<LittleEndian>().unwrap();
    let _version = self.stream.read_u16::<LittleEndian>().unwrap();

    self.animation_data_size = animation_data_size;
    self.sound_data_size = sound_data_size;
    self.frame_count = frame_count + 1;
  }

  fn read_filename(&mut self) -> String {
    // Parent and current filenames are stored as:
    //
    // - three bytes representing the last six digits of the console's MAC address
    // - thirteen-character `String`
    // - `u16` edit counter
    let mac = read_n_to_vec(&mut self.stream, 3);
    let ident = read_n_to_vec(&mut self.stream, 13)
      .into_iter()
      .map(|c| c as char)
      .collect::<String>();
    let edits = self.stream.read_u16::<LittleEndian>().unwrap();

    // Filenames are formatted as
    // <three-byte MAC as hexadecimal>_<thirteen-character string>_<edit counter as
    // three-digit number>
    //
    // Example: F78DA8_14768882B56B8_030
    format!(
      "{}_{}_{:#03}",
      mac
        .into_iter()
        .map(|m| format!("{:02X}", m))
        .collect::<String>(),
      String::from_utf8(ident.as_bytes().to_vec()).unwrap(),
      edits,
    )
  }

  /// Decode metadata
  ///
  /// <https://github.com/pbsds/hatena-server/wiki/PPM-format#file-header>
  fn read_meta(&mut self) {
    self.stream.set_position(0x10);

    self.lock = self.stream.read_u16::<LittleEndian>().unwrap();
    self.thumb_index = self.stream.read_u16::<LittleEndian>().unwrap();
    self.root_author_name = strip_null(&read_n_to_as_utf8_from_stream!(22, self));
    self.parent_author_name = strip_null(&read_n_to_as_utf8_from_stream!(22, self));
    self.current_author_name = strip_null(&read_n_to_as_utf8_from_stream!(22, self));
    self.parent_author_id = vec_u8_to_string(read_n_to_vec(&mut self.stream, 8).as_mut_slice());
    self.current_author_id = vec_u8_to_string(read_n_to_vec(&mut self.stream, 8).as_mut_slice());
    self.parent_filename = self.read_filename();
    self.current_filename = self.read_filename();
    self.root_author_id = vec_u8_to_string(read_n_to_vec(&mut self.stream, 8).as_mut_slice());
    self.partial_filename = vec_u8_to_string(read_n_to_vec(&mut self.stream, 8).as_slice()); // Not really useful for anything

    // Timestamp is stored as the number of seconds since 2000, January, 1st
    let timestamp = self.stream.read_u32::<LittleEndian>().unwrap();
    self.timestamp = DateTime::from_utc(
      // We add 946684800 to convert this to a more common Unix timestamp,
      // which starts on 1970, January, 1st
      NaiveDateTime::from_timestamp(i64::from(timestamp) + 946_684_800, 0),
      Utc,
    );
  }

  #[allow(unused)]
  fn read_thumbnail(&mut self) -> Vec<Vec<u64>> {
    self.stream.set_position(0xA0);

    let mut bitmap = vec![vec![0; 64]; 48];

    for tile_index in 0..48 {
      let tile_x = tile_index % 8 * 8;
      let tile_y = tile_index / 8 * 8;

      for line in 0..8 {
        // [This](https://linuxtut.com/en/ff1ac20b39137f1ccdb9/) can be used,
        // but let's do it in Rust.
        for pixel in (0..8).step_by(2) {
          let byte = self.stream.read_uint::<LittleEndian>(1).unwrap();
          let x = tile_x + pixel;
          let y = tile_y + line;

          bitmap[y][x] = byte & 0x0F;
          bitmap[y][x + 1] = (byte >> 4) & 0x0F;
        }
      }
    }

    bitmap
  }

  fn read_animation_header(&mut self) {
    self.stream.set_position(0x06A0);

    let table_size = self.stream.read_u16::<LittleEndian>().unwrap();
    let _unknown = self.stream.read_u16::<LittleEndian>().unwrap();
    let flags = self.stream.read_u32::<LittleEndian>().unwrap();

    // Unpack animation flags
    self.layer_1_visible = (flags >> 11) & 0x01 != 0;
    self.layer_2_visible = (flags >> 10) & 0x01 != 0;
    self.loop_ = (flags >> 1) & 0x01 != 0;

    // Read offset table into an array
    let offset_table = {
      let from_buffer = read_n_to_vec(&mut self.stream, table_size.into());
      let mut buffer = Vec::with_capacity((table_size / 4).into());

      // I'm very glad that I got this working. It took way longer than it
      // should have...
      //
      // 2022. 02. 25. 03:58., Fuwn
      for index in (0..usize::from(table_size)).step_by(4) {
        buffer.push(
          (u32::from(from_buffer[index]))
            | (u32::from(from_buffer[index + 1]) << 8)
            | (u32::from(from_buffer[index + 2]) << 16)
            | (u32::from(from_buffer[index + 3]) << 24),
        );
      }

      buffer
    };
    self.offset_table = offset_table
      .into_iter()
      .map(|m| m + 0x06A0 + 8 + u32::from(table_size))
      .collect();
  }

  fn read_sound_header(&mut self) {
    // offset = frame data offset + frame data length + sound effect flags
    //
    // <https://github.com/pbsds/hatena-server/wiki/PPM-format#sound-data-section>
    let mut offset = 0x06A0 + self.animation_data_size + u32::from(self.frame_count);
    if offset % 2 != 0 {
      // Account for multiple-of-four padding
      offset += 4 - (offset % 4);
    }

    self.stream.set_position(u64::from(offset));

    let _bgm_size = self.stream.read_u32::<LittleEndian>().unwrap();
    let _se1_size = self.stream.read_u32::<LittleEndian>().unwrap();
    let _se2_size = self.stream.read_u32::<LittleEndian>().unwrap();
    let _se3_size = self.stream.read_u32::<LittleEndian>().unwrap();
    let frame_speed = self.stream.read_u8().unwrap();
    let bgm_speed = self.stream.read_u8().unwrap();

    self.frame_speed = 8 - frame_speed;
    self.bgm_speed = 8 - bgm_speed;
    self.framerate = *FRAMERATES.get(&self.frame_speed).unwrap();
    self.bgm_framerate = *FRAMERATES.get(&self.bgm_speed).unwrap();
  }

  fn frame_is_new(&mut self, index: usize) -> bool {
    self
      .stream
      .set_position(u64::from(*self.offset_table.get(index).unwrap()));

    self.stream.read_uint::<LittleEndian>(1).unwrap() >> 7 & 0x1 != 0
  }

  fn read_line_types(line_types: Vec<u8>) -> impl Generator<Yield = (usize, u8), Return = ()> {
    move || {
      for index in 0..192 {
        let line_type = line_types.get(index / 4).unwrap() >> ((index % 4) * 2) & 0x03;
        yield (index, line_type);
      }
    }
  }

  fn read_frame(&mut self, index: usize) -> &Vec<Vec<Vec<u8>>> {
    // Decode the previous frames if needed
    if index != 0 && self.prev_frame_index != index - 1 && !self.frame_is_new(index) {
      self.read_frame(index - 1);
    }

    // Copy the current layer buffers to the previous ones
    self.prev_layers = self.layers.clone();
    self.prev_frame_index = index;
    // Clear the current layer buffers by resetting them to zero
    self.layers.fill(vec![vec![0u8; 256]; 192]);

    // Seek to the frame offset so we can start reading
    self
      .stream
      .set_position(u64::from(*self.offset_table.get(index).unwrap()));

    // Unpack frame header flags
    let header = self.stream.read_uint::<LittleEndian>(1).unwrap();
    let is_new_frame = (header >> 7) & 0x01 != 0;
    let is_translated = (header >> 5) & 0x03 != 0;
    // If the frame is translated, we need to unpack the x and y values
    let translation_x = if is_translated {
      self.stream.read_i8().unwrap()
    } else {
      0
    };
    let translation_y = if is_translated {
      self.stream.read_i8().unwrap()
    } else {
      0
    };
    // Read line encoding bytes
    let line_types = vec![
      read_n_of_size_from_to_vec!(48, self, u8),
      read_n_of_size_from_to_vec!(48, self, u8),
    ];

    // Loop through layers
    #[allow(clippy::needless_range_loop)]
    for layer in 0..2 {
      let bitmap = &mut self.layers[layer];

      {
        let mut generator = Self::read_line_types(line_types[layer].clone());
        while let std::ops::GeneratorState::Yielded((line, line_type)) =
          std::pin::Pin::new(&mut generator).resume(())
        {
          let mut pixel = 0;

          // No data stored for this line
          if line_type == 0 {
            // pass;
          } else if line_type == 1 || line_type == 2 {
            // Compressed line
            // If `line_type == 2`, the line starts off with all the pixels set to one
            if line_type == 2 {
              for i in 0..256 {
                bitmap[line][i] = 1;
              }
            }

            // Unpack chunk usage
            let mut chunk_usage = self.stream.read_u32::<byteorder::BigEndian>().unwrap();

            // Unpack pixel chunks
            while pixel < 256 {
              if chunk_usage & 0x8000_0000 == 0 {
                pixel += 8;
              } else {
                let chunk = self.stream.read_uint::<LittleEndian>(1).unwrap();

                for bit in 0..8 {
                  bitmap[line][pixel] = (chunk >> bit & 0x1) as u8;
                  pixel += 1;
                }
              }

              chunk_usage <<= 1;
            }
          // Raw line
          } else if line_type == 3 {
            // Unpack pixel chunks
            while pixel < 256 {
              let chunk = self.stream.read_uint::<LittleEndian>(1).unwrap();

              for bit in 0..8 {
                bitmap[line][pixel] = (chunk >> bit & 0x1) as u8;
                pixel += 1;
              }
            }
          }
        }
      }
    }

    // Frame diffing
    //
    // If the current frame is based on the previous one, merge them by XOR-ing
    // their pixels. This is a big performance bottleneck...
    if !is_new_frame {
      // Loop through lines
      for y in 0..192 {
        // Skip to next line if this one falls off the top edge of the screen
        // if y - (translation_y as usize) < 0 {
        //   continue;
        // }
        // Stop once the bottom screen edge has been reached
        if y - translation_y as usize >= 192 {
          break;
        }

        for x in 0..256 {
          // Skip to the next pixel if this one falls off the left edge of the screen
          // if x - (translation_x as usize) < 0 {
          //   continue;
          // }
          // Stop diffing this line once the right screen edge has been reached
          if x - translation_x as usize >= 256 {
            break;
          }

          // Diff pixels with a binary XOR
          self.layers[0][y][x] ^=
            self.prev_layers[0][y - translation_y as usize][x - translation_x as usize];
          self.layers[1][y][x] ^=
            self.prev_layers[1][y - translation_y as usize][x - translation_x as usize];
        }
      }
    }

    &self.layers
  }

  pub fn get_frame_palette(&mut self, index: usize) -> Vec<(u8, u8, u8)> {
    self.stream.set_position(self.offset_table[index].into());

    let header = self.stream.read_uint::<LittleEndian>(1).unwrap();
    let paper_colour = header & 0x1;
    let pen = vec![
      None,
      Some(if paper_colour == 1 { *BLACK } else { *WHITE }),
      Some(*RED),
      Some(*BLUE),
    ];

    vec![
      if paper_colour == 1 { *WHITE } else { *BLACK },
      pen.get(((header >> 1) & 0x3) as usize).unwrap().unwrap(), // Layer one colour
      pen.get(((header >> 3) & 0x3) as usize).unwrap().unwrap(), // Layer two colour
    ]
  }

  pub fn get_frame_pixels(&mut self, index: usize) -> Vec<Vec<u8>> {
    let layers = self.read_frame(index);
    let mut pixels = vec![vec![0u8; 256]; 192];

    #[allow(clippy::needless_range_loop)]
    for y in 0..192 {
      for x in 0..256 {
        if layers[0][y][x] > 0 {
          pixels[y][x] = 1;
        } else if layers[1][y][x] > 0 {
          pixels[y][x] = 2;
        }
      }
    }

    pixels
  }

  pub const fn get_frame_count(&self) -> u16 { self.frame_count }

  pub const fn get_thumb_index(&self) -> u16 { self.thumb_index }

  pub const fn get_framerate(&self) -> f64 { self.framerate }

  pub fn dump_to_json(&self, filename: &str) {
    let writer = std::io::BufWriter::new(fs::File::create(filename).unwrap());
    serde_json::to_writer_pretty(
      writer,
      &serde_json::json!({
        "animation_data_size": self.animation_data_size,
        "sound_data_size": self.sound_data_size,
        "frame_count": self.frame_count,
        "lock": self.lock,
        "thumb_index": self.thumb_index,
        "root_author_name": self.root_author_name,
        "parent_author_name": self.parent_author_name,
        "current_author_name": self.current_author_name,
        "root_author_id": self.root_author_id,
        "parent_author_id": self.parent_author_id,
        "current_author_id": self.current_author_id,
        "parent_filename": self.parent_filename,
        "current_filename": self.current_filename,
        "partial_filename": self.partial_filename,
        "timestamp": self.timestamp.to_string(),
        "layer_1_visible": self.layer_1_visible,
        "layer_2_visible": self.layer_2_visible,
        "loop": self.loop_,
        "frame_speed": self.frame_speed,
        "bgm_speed": self.bgm_speed,
        "framerate": self.framerate,
        "bgm_framerate": self.bgm_framerate,
      }),
    )
    .unwrap();
  }
}
impl Default for PPMParser {
  fn default() -> Self {
    Self {
      stream:              Cursor::default(),
      layers:              Vec::new(),
      prev_layers:         Vec::new(),
      prev_frame_index:    Default::default(),
      animation_data_size: Default::default(),
      sound_data_size:     Default::default(),
      frame_count:         Default::default(),
      lock:                Default::default(),
      thumb_index:         Default::default(),
      root_author_name:    String::default(),
      parent_author_name:  String::default(),
      current_author_name: String::default(),
      parent_author_id:    String::default(),
      current_author_id:   String::default(),
      parent_filename:     String::default(),
      current_filename:    String::default(),
      root_author_id:      String::default(),
      partial_filename:    String::default(),
      timestamp:           DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
      layer_1_visible:     Default::default(),
      layer_2_visible:     Default::default(),
      loop_:               Default::default(),
      frame_speed:         Default::default(),
      bgm_speed:           Default::default(),
      framerate:           Default::default(),
      bgm_framerate:       Default::default(),
      offset_table:        Vec::default(),
    }
  }
}
