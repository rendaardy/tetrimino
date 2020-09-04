use std::thread;
use std::time::{Duration, SystemTime};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

mod score;
mod shape;
mod tetrimino;

use crate::tetrimino::Tetris;

const NB_HIGHSOCRES: usize = 5;
const TETRIS_HEIGHT: usize = 40;

fn create_texture_rect<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    r: u8,
    g: u8,
    b: u8,
    width: u32,
    height: u32,
) -> Option<Texture<'a>> {
    if let Ok(mut square_texture) = texture_creator.create_texture_target(None, width, height) {
        canvas
            .with_texture_canvas(&mut square_texture, |texture| {
                texture.set_draw_color(Color::RGB(r, g, b));
                texture.clear();
            })
            .expect("Failed to color a texture");
        Some(square_texture)
    } else {
        None
    }
}

fn create_texture_from_text<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    font: &Font,
    text: &str,
    r: u8,
    g: u8,
    b: u8,
) -> Option<Texture<'a>> {
    if let Ok(surface) = font.render(text).blended(Color::RGB(r, g, b)) {
        texture_creator.create_texture_from_surface(&surface).ok()
    } else {
        None
    }
}

fn get_rect_from_text(text: &str, x: i32, y: i32) -> Option<Rect> {
    Some(Rect::new(x, y, text.len() as u32 * 10, 30))
}

fn handle_events(
    tetris: &mut Tetris,
    quit: &mut bool,
    timer: &mut SystemTime,
    event_pump: &mut EventPump,
) -> bool {
    let mut make_permanent = false;
    if let Some(ref mut piece) = tetris.current_piece {
        let mut tmp_x = piece.x;
        let mut tmp_y = piece.y;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    *quit = true;
                    break;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    *timer = SystemTime::now();
                    tmp_y += 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    tmp_x += 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    tmp_x -= 1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    piece.rotate(&tetris.game_map);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    let x = piece.x;
                    let mut y = piece.y;
                    while piece.change_position(&tetris.game_map, x, y + 1) == true {
                        y += 1;
                    }
                    make_permanent = true;
                }
                _ => {}
            }
        }

        if !make_permanent {
            if piece.change_position(&tetris.game_map, tmp_x, tmp_y) == false && tmp_y != piece.y {
                make_permanent = true;
            }
        }
    }

    if make_permanent {
        tetris.make_permanent();
        *timer = SystemTime::now();
    }

    make_permanent
}

fn update_vec(v: &mut Vec<u32>, value: u32) -> bool {
    if v.len() < NB_HIGHSOCRES {
        v.push(value);
        v.sort();
        return true;
    }

    for entry in v.iter_mut() {
        if value > *entry {
            *entry = value;
            return true;
        }
    }

    false
}

fn print_game_information(tetris: &Tetris) {
    let mut new_highest_score = true;
    let mut new_highest_lines_sent = true;

    if let Some((mut highscores, mut lines_sent)) = score::load_highscores_and_lines() {
        new_highest_score = update_vec(&mut highscores, tetris.score);
        new_highest_lines_sent = update_vec(&mut lines_sent, tetris.nb_lines);

        if new_highest_score || new_highest_lines_sent {
            score::save_highscores_and_lines(&highscores, &lines_sent);
        }
    } else {
        score::save_highscores_and_lines(&[tetris.score], &[tetris.nb_lines]);
    }

    println!("Game over...");
    println!(
        "Score: {}{}",
        tetris.score,
        if new_highest_score {
            " [NEW HIGHSCORE] "
        } else {
            ""
        }
    );
    println!(
        "Number of lines: {}{}",
        tetris.nb_lines,
        if new_highest_lines_sent {
            " [NEW HIGHSCORE] "
        } else {
            ""
        }
    );
    println!("Current level: {}", tetris.current_level);
}

fn display_game_information<'a>(
    tetris: &Tetris,
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    font: &Font,
    start_x_pos: i32,
) {
    let score_text = format!("Score: {}", tetris.score);
    let lines_set_text = format!("Lines sent: {}", tetris.nb_lines);
    let level_text = format!("Level: {}", tetris.current_level);
    let score = create_texture_from_text(&texture_creator, &font, &score_text, 255, 255, 255)
        .expect("Cannot render text");
    let lines_sent =
        create_texture_from_text(&texture_creator, &font, &lines_set_text, 255, 255, 255)
            .expect("Cannot render text");
    let level = create_texture_from_text(&texture_creator, &font, &level_text, 255, 255, 255)
        .expect("Cannot render text");

    canvas
        .copy(
            &score,
            None,
            get_rect_from_text(&score_text, start_x_pos, 90),
        )
        .expect("Couldn't copy text");
    canvas
        .copy(
            &lines_sent,
            None,
            get_rect_from_text(&lines_set_text, start_x_pos, 125),
        )
        .expect("Couldn't copy text");
    canvas
        .copy(
            &level,
            None,
            get_rect_from_text(&level_text, start_x_pos, 160),
        )
        .expect("Couldn't copy text");
}

pub fn main() {
    let sdl_context = sdl2::init().expect("SDL initialization failed");
    let video_subsystem = sdl_context
        .video()
        .expect("Couldn't get SDL video subsystem");

    let ttf_context = sdl2::ttf::init().expect("SDL TTF initialization failed");
    let font = ttf_context
        .load_font("assets/catamaran_regular.ttf", 128)
        .expect("Couldn't load the font");

    let width = 600;
    let height = 800;
    let mut tetris = Tetris::new();
    let mut timer = SystemTime::now();
    let mut event_pump = sdl_context
        .event_pump()
        .expect("Failed to get SDL event pump");

    let grid_x = 20;
    let grid_y = (height - TETRIS_HEIGHT as u32 * 16) as i32 / 2;

    let window = video_subsystem
        .window("Tetris", width, height)
        .position_centered()
        .build()
        .expect("Failed to create window");

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .expect("Failed to convert window into canvas");

    let texture_creator = canvas.texture_creator();
    let grid = create_texture_rect(
        &mut canvas,
        &texture_creator,
        0,
        0,
        0,
        TETRIS_HEIGHT as u32 * 10,
        TETRIS_HEIGHT as u32 * 16,
    )
    .expect("Failed to create a texture");
    let border = create_texture_rect(
        &mut canvas,
        &texture_creator,
        255,
        255,
        255,
        TETRIS_HEIGHT as u32 * 10 + 20,
        TETRIS_HEIGHT as u32 * 16 + 20,
    )
    .expect("Failed to create a texture");

    macro_rules! texture {
        ($r:expr, $g:expr, $b:expr) => {
            create_texture_rect(
                &mut canvas,
                &texture_creator,
                $r,
                $g,
                $b,
                TETRIS_HEIGHT as u32,
                TETRIS_HEIGHT as u32,
            )
            .unwrap()
        };
    }

    let textures = [
        texture!(255, 69, 69),
        texture!(255, 220, 69),
        texture!(237, 150, 37),
        texture!(171, 99, 237),
        texture!(77, 149, 239),
        texture!(39, 218, 225),
        texture!(45, 216, 47),
    ];

    loop {
        if tetrimino::is_time_over(&tetris, &timer) {
            let mut make_permanent = false;
            if let Some(ref mut piece) = tetris.current_piece {
                let x = piece.x;
                let y = piece.y + 1;
                make_permanent = !piece.change_position(&tetris.game_map, x, y);
            }

            if make_permanent {
                tetris.make_permanent();
            }

            timer = SystemTime::now();
        }

        // We need to draw the tetrimino grid in here
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.clear();

        display_game_information(
            &tetris,
            &mut canvas,
            &texture_creator,
            &font,
            width as i32 - grid_x - 130,
        );

        canvas
            .copy(
                &border,
                None,
                Rect::new(
                    10,
                    (height - TETRIS_HEIGHT as u32 * 16) as i32 / 2 - 10,
                    TETRIS_HEIGHT as u32 * 10 + 20,
                    TETRIS_HEIGHT as u32 * 16 + 20,
                ),
            )
            .expect("Render failed");

        canvas
            .copy(
                &grid,
                None,
                Rect::new(
                    20,
                    (height - TETRIS_HEIGHT as u32 * 16) as i32 / 2,
                    TETRIS_HEIGHT as u32 * 10,
                    TETRIS_HEIGHT as u32 * 16,
                ),
            )
            .expect("Render failed");

        if tetris.current_piece.is_none() {
            let current_piece = tetris.create_new_tetrimino();
            if !current_piece.test_current_position(&tetris.game_map) {
                print_game_information(&tetris);
                break;
            }
            tetris.current_piece = Some(current_piece);
        }

        let mut quit = false;
        if !handle_events(&mut tetris, &mut quit, &mut timer, &mut event_pump) {
            if let Some(ref mut piece) = tetris.current_piece {
                // We need to draw our current tetrimono in here
                for (line_nb, line) in piece.states[piece.current_state as usize]
                    .iter()
                    .enumerate()
                {
                    for (case_nb, case) in line.iter().enumerate() {
                        if *case == 0 {
                            continue;
                        }

                        canvas
                            .copy(
                                &textures[*case as usize - 1],
                                None,
                                Rect::new(
                                    grid_x
                                        + (piece.x + case_nb as isize) as i32
                                            * TETRIS_HEIGHT as i32,
                                    grid_y + (piece.y + line_nb) as i32 * TETRIS_HEIGHT as i32,
                                    TETRIS_HEIGHT as u32,
                                    TETRIS_HEIGHT as u32,
                                ),
                            )
                            .expect("Couldn't copy texture into window");
                    }
                }
            }
        }

        if quit {
            print_game_information(&tetris);
            break;
        }

        // We need to draw the game map in here
        for (line_nb, line) in tetris.game_map.iter().enumerate() {
            for (case_nb, case) in line.iter().enumerate() {
                if *case == 0 {
                    continue;
                }

                canvas
                    .copy(
                        &textures[*case as usize - 1],
                        None,
                        Rect::new(
                            grid_x + case_nb as i32 * TETRIS_HEIGHT as i32,
                            grid_y + line_nb as i32 * TETRIS_HEIGHT as i32,
                            TETRIS_HEIGHT as u32,
                            TETRIS_HEIGHT as u32,
                        ),
                    )
                    .expect("Couldn't copy texture into window");
            }
        }

        canvas.present();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
